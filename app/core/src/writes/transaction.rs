use super::*;
use policy::Entry;
use std::io::{Read, Seek};
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};

fn path_key(path: &NativePath) -> Result<Vec<u8>> {
    Ok(path.relative()?.as_os_str().as_encoded_bytes().to_vec())
}
fn selected(request: &WriteRequest) -> Vec<SelectedPath> {
    match request {
        WriteRequest::Commit { selected, .. } => selected.clone(),
        WriteRequest::Stage { paths, .. } => paths
            .iter()
            .map(|path| SelectedPath {
                path: path.clone(),
                source: ContentSource::WorkingTree,
            })
            .collect(),
        WriteRequest::Unstage { paths, .. } => paths
            .iter()
            .map(|path| SelectedPath {
                path: path.clone(),
                source: ContentSource::Index,
            })
            .collect(),
    }
}
fn hook_policy(request: &WriteRequest) -> HookPolicy {
    match request {
        WriteRequest::Commit { hooks, .. }
        | WriteRequest::Stage { hooks, .. }
        | WriteRequest::Unstage { hooks, .. } => *hooks,
    }
}
fn scratch_repo(dir: &Path) -> PathBuf {
    dir.join("repository.git")
}
fn object_dir(dir: &Path) -> PathBuf {
    scratch_repo(dir).join("objects")
}
fn apply(
    repo: &Path,
    index: &Path,
    path: &NativePath,
    entry: Option<&Entry>,
    format: &ObjectFormat,
    options: &ReadOptions,
) -> Result<()> {
    let relative = path_key(path)?;
    let zero = "0".repeat(if *format == ObjectFormat::Sha1 {
        40
    } else {
        64
    });
    let mut record = if let Some(entry) = entry {
        format!("{} {}\t", entry.mode, entry.oid).into_bytes()
    } else {
        format!("0 {zero}\t").into_bytes()
    };
    record.extend(relative);
    record.push(0);
    adapter::run(
        repo,
        &["update-index", "-z", "--index-info"],
        Some(index),
        None,
        Some(record),
        None,
        options,
    )?;
    Ok(())
}
fn safe_path(meta: &Metadata, path: &NativePath) -> Result<PathBuf> {
    let root = meta.root.to_path()?;
    let relative = path.relative()?;
    let mut parent = root.clone();
    let parts = relative.components().collect::<Vec<_>>();
    for part in parts.iter().take(parts.len() - 1) {
        parent.push(part.as_os_str());
        match fs::symlink_metadata(&parent) {
            Ok(m) if m.is_symlink() || !m.is_dir() => {
                return Err(fail(
                    ErrorKind::InvalidPath,
                    "Symlinked/non-directory ancestor",
                ))
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => break,
            Err(error) => return Err(error.into()),
        }
    }
    Ok(root.join(relative))
}
enum CapturedPayload {
    Bytes(Vec<u8>),
    File(File),
}
type CapturedContent = (String, Option<(String, CapturedPayload)>);
fn open_regular(path: &Path) -> Result<File> {
    let file = fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(path)?;
    if !file.metadata()?.is_file() {
        return Err(unsupported("Selected file type changed"));
    }
    Ok(file)
}
fn capture_regular(
    mut file: File,
    mode: &str,
    snapshot_dir: Option<&Path>,
    options: &ReadOptions,
) -> Result<CapturedContent> {
    use sha2::{Digest, Sha256};
    // Only materialization needs a copy. Unlink it before writing any content
    // so cancellation, errors and process exit all release the scratch space.
    let mut snapshot = if let Some(dir) = snapshot_dir {
        let path = dir.join("capture.tmp");
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&path)?;
        fs::remove_file(path)?;
        Some(file)
    } else {
        None
    };
    let mut hash = Sha256::new();
    hash.update(mode.as_bytes());
    let started = Instant::now();
    let mut buffer = [0; 65536];
    loop {
        options.check(started)?;
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hash.update(&buffer[..count]);
        if let Some(snapshot) = &mut snapshot {
            snapshot.write_all(&buffer[..count])?;
        }
    }
    if let Some(snapshot) = &mut snapshot {
        snapshot.rewind()?;
    }
    Ok((
        format!("{:x}", hash.finalize()),
        snapshot.map(|f| (mode.to_owned(), CapturedPayload::File(f))),
    ))
}
fn capture(
    meta: &Metadata,
    path: &NativePath,
    old: Option<&Entry>,
    config: &EffectiveConfig,
    snapshot_dir: Option<&Path>,
    options: &ReadOptions,
) -> Result<CapturedContent> {
    let target = safe_path(meta, path)?;
    let content = match fs::symlink_metadata(&target) {
        Ok(m) if m.is_symlink() => Some((
            "120000".to_owned(),
            fs::read_link(&target)?
                .as_os_str()
                .as_encoded_bytes()
                .to_vec(),
        )),
        Ok(m) if m.is_file() => {
            let file = open_regular(&target)?;
            let mode = if !config.file_mode {
                old.filter(|e| e.mode.starts_with("100"))
                    .map(|e| e.mode.as_str())
                    .unwrap_or("100644")
            } else if file.metadata()?.mode() & 0o111 != 0 {
                "100755"
            } else {
                "100644"
            };
            return capture_regular(file, mode, snapshot_dir, options);
        }
        Ok(m) if m.is_dir() && old.is_some_and(|e| e.mode == "160000") => {
            let child = inspection::snapshot(&target)?;
            let head = child
                .head
                .ok_or_else(|| unsupported("Submodule HEAD is unborn"))?;
            Some(("160000".into(), head.into_bytes()))
        }
        Ok(_) => return Err(unsupported("Selected file type is unsupported")),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => return Err(error.into()),
    };
    let hash = if let Some((mode, bytes)) = &content {
        digest(&[mode.as_bytes(), bytes].concat())
    } else {
        digest(b"missing")
    };
    Ok((
        hash,
        content.map(|(mode, bytes)| (mode, CapturedPayload::Bytes(bytes))),
    ))
}
fn attributes(meta: &Metadata, paths: &[SelectedPath], options: &ReadOptions) -> Result<()> {
    let mut input = Vec::new();
    for selected in paths {
        input.extend(path_key(&selected.path)?);
        input.push(0);
    }
    let bytes = adapter::run(
        &meta.root.to_path()?,
        &["check-attr", "-z", "--all", "--stdin"],
        None,
        None,
        Some(input),
        None,
        options,
    )?;
    let fields = bytes
        .split(|b| *b == 0)
        .filter(|f| !f.is_empty())
        .collect::<Vec<_>>();
    for triple in fields.chunks(3) {
        if triple.len() != 3 {
            return Err(unsupported("Invalid attribute output"));
        }
        if [
            b"filter".as_slice(),
            b"text",
            b"eol",
            b"working-tree-encoding",
            b"ident",
        ]
        .contains(&triple[1])
            && triple[2] != b"unset"
            && triple[2] != b"unspecified"
        {
            return Err(unsupported("Selected content conversion is unsupported"));
        }
    }
    Ok(())
}
fn tree(repo: &Path, index: &Path, options: &ReadOptions) -> Result<String> {
    adapter::text(adapter::run(
        repo,
        &["write-tree"],
        Some(index),
        None,
        None,
        None,
        options,
    )?)
}
fn c_quote(bytes: &[u8]) -> Vec<u8> {
    let mut out = vec![b'"'];
    for b in bytes {
        match b {
            b'"' | b'\\' => {
                out.push(b'\\');
                out.push(*b);
            }
            32..=126 => out.push(*b),
            _ => out.extend(format!("\\{b:03o}").as_bytes()),
        }
    }
    out.extend_from_slice(b"\"\n");
    out
}
fn candidate_entries(_repo: &Path, index: &Path, format: &ObjectFormat) -> Result<Vec<Entry>> {
    policy::index_entries(&inspection::file_bytes(index, 16 * 1024 * 1024)?, format)
}
impl ControlledWrites {
    pub fn propose(
        &mut self,
        workspace: &WorkspaceId,
        worktree: &WorktreeId,
        request: WriteRequest,
    ) -> Result<Proposal> {
        self.prepare_operation(workspace, worktree, request, true)
    }
    #[cfg(feature = "application-services")]
    pub(crate) fn prepare_direct_operation(
        &mut self,
        workspace: &WorkspaceId,
        worktree: &WorktreeId,
        request: WriteRequest,
        cancellation: inspection::Cancellation,
    ) -> Result<Proposal> {
        let previous = std::mem::replace(&mut self.options.process.cancellation, cancellation);
        let result = self.prepare_operation(workspace, worktree, request, false);
        self.options.process.cancellation = previous;
        result
    }
    fn prepare_operation(
        &mut self,
        workspace: &WorkspaceId,
        worktree: &WorktreeId,
        request: WriteRequest,
        include_patch: bool,
    ) -> Result<Proposal> {
        let id = Uuid::new_v4().to_string();
        let result =
            self.prepare_operation_inner(workspace, worktree, request, include_patch, id.clone());
        if result.is_err() {
            let operations = self.workspace.data_directory().join("operations");
            temporary::cleanup(self.workspace.connection(), &operations, &id)?;
        }
        result
    }
    fn prepare_operation_inner(
        &mut self,
        workspace: &WorkspaceId,
        worktree: &WorktreeId,
        mut request: WriteRequest,
        include_patch: bool,
        id: String,
    ) -> Result<Proposal> {
        let entry = self.entry(workspace, worktree)?;
        let meta = entry.metadata;
        self.ensure_fetch_import_resolved(&meta.common_binding, None)?;
        self.ensure_upstream_resolved(&meta.common_binding, None)?;
        let (original, entries, branch, head) = policy::preflight(&meta, &self.options)?;
        let preference = match &request {
            WriteRequest::Commit { signing, .. } => *signing,
            _ => SigningPreference::Unsigned,
        };
        let config = policy::effective(&meta, preference, &self.options)?;
        self.trusted(worktree, &meta, &config)?;
        let hooks = hook_policy(&request);
        policy::validate_hook_policy(&config, hooks)?;
        let mut paths = selected(&request);
        if paths.is_empty() || paths.len() > 10_000 {
            return Err(fail(
                ErrorKind::InvalidRequest,
                "Path selection must contain 1..10000 paths",
            ));
        }
        let mut keys = Vec::new();
        for path in &mut paths {
            let relative = path.path.relative()?;
            path.path = NativePath::from_path(&relative);
            let key = path_key(&path.path)?;
            if keys.iter().any(|k: &Vec<u8>| k.eq_ignore_ascii_case(&key)) {
                return Err(unsupported("Duplicate or case-alias selections"));
            }
            if entries
                .iter()
                .any(|e| path_key(&e.path).is_ok_and(|p| p != key && p.eq_ignore_ascii_case(&key)))
            {
                return Err(unsupported("Selection aliases an existing index path"));
            }
            keys.push(key);
        }
        attributes(&meta, &paths, &self.options.process)?;
        let message = if let WriteRequest::Commit { message, .. } = &mut request {
            if message.trim().is_empty() || message.len() > 512 * 1024 || message.contains('\0') {
                return Err(fail(
                    ErrorKind::InvalidRequest,
                    "A nonempty NUL-free message of at most 512 KiB is required",
                ));
            }
            if !message.ends_with('\n') {
                message.push('\n');
            }
            message.as_bytes().to_vec()
        } else {
            Vec::new()
        };
        let dir = self.directory(&id)?;
        use std::os::unix::fs::DirBuilderExt;
        fs::DirBuilder::new().mode(0o700).create(&dir)?;
        if let Err(error) = temporary::remember(
            self.workspace.connection(),
            dir.parent().unwrap(),
            &id,
            &dir,
            &fs::symlink_metadata(&dir)?,
        ) {
            fs::remove_dir(&dir)?;
            return Err(error);
        }
        store::sync_dir(dir.parent().unwrap())?;
        let repo = scratch_repo(&dir);
        let format = if meta.object_format == ObjectFormat::Sha1 {
            "sha1"
        } else {
            "sha256"
        };
        adapter::run(
            &dir,
            &[
                "-c",
                "init.templateDir=",
                "init",
                "--bare",
                "-q",
                &format!("--object-format={format}"),
                repo.to_str()
                    .ok_or_else(|| unsupported("Operation directory must be UTF-8"))?,
            ],
            None,
            None,
            None,
            None,
            &self.options.process,
        )?;
        let alternate = meta.common_dir.to_path()?.join("objects");
        store::write_new(
            &object_dir(&dir).join("info/alternates"),
            &c_quote(alternate.as_os_str().as_encoded_bytes()),
        )?;
        let selected_index = dir.join("selected.index");
        let reconciled = dir.join("reconciled.index");
        let original_path = dir.join("original.index");
        store::write_new(&original_path, &original)?;
        if original.is_empty() {
            adapter::run(
                &repo,
                &["read-tree", "--empty"],
                Some(&reconciled),
                None,
                None,
                None,
                &self.options.process,
            )?;
        } else {
            store::write_new(&reconciled, &original)?;
        }
        let original_tree = tree(&repo, &reconciled, &self.options.process)?;
        if matches!(&request, WriteRequest::Commit { .. }) {
            if let Some(head) = &head {
                adapter::run(
                    &repo,
                    &["read-tree", head],
                    Some(&selected_index),
                    None,
                    None,
                    None,
                    &self.options.process,
                )?;
            } else {
                adapter::run(
                    &repo,
                    &["read-tree", "--empty"],
                    Some(&selected_index),
                    None,
                    None,
                    None,
                    &self.options.process,
                )?;
            }
        } else {
            store::write_new(
                &selected_index,
                &inspection::file_bytes(&reconciled, 16 * 1024 * 1024)?,
            )?;
        }
        let head_index = dir.join("head.index");
        if let Some(head) = &head {
            adapter::run(
                &repo,
                &["read-tree", head],
                Some(&head_index),
                None,
                None,
                None,
                &self.options.process,
            )?;
        } else {
            adapter::run(
                &repo,
                &["read-tree", "--empty"],
                Some(&head_index),
                None,
                None,
                None,
                &self.options.process,
            )?;
        }
        let head_entries = candidate_entries(&repo, &head_index, &meta.object_format)?;
        let mut fingerprints = Vec::new();
        for selected in &paths {
            let old = entries.iter().find(|e| e.path == selected.path);
            let tracked = head_entries.iter().find(|e| e.path == selected.path);
            // Index-only gitlinks may be uninitialized. Their index entry is the authority.
            let captured = if selected.source == ContentSource::Index
                && old.is_some_and(|e| e.mode == "160000")
            {
                None
            } else {
                Some(capture(
                    &meta,
                    &selected.path,
                    old.or(tracked),
                    &config,
                    (selected.source == ContentSource::WorkingTree).then_some(dir.as_path()),
                    &self.options.process,
                )?)
            };
            if let Some((fingerprint, _)) = &captured {
                fingerprints.push((selected.path.clone(), fingerprint.clone()));
            }
            let new = if matches!(&request, WriteRequest::Unstage { .. }) {
                tracked.cloned()
            } else if selected.source == ContentSource::Index {
                if old.is_none() && tracked.is_none() {
                    return Err(unsupported("Index selection is neither staged nor tracked"));
                }
                old.cloned()
            } else if let Some((mode, bytes)) = captured.and_then(|(_, content)| content) {
                let oid = match bytes {
                    CapturedPayload::File(file) => {
                        adapter::hash_file(&repo, file, &self.options.process)?
                    }
                    CapturedPayload::Bytes(bytes) if mode == "160000" => {
                        String::from_utf8(bytes).map_err(|_| unsupported("Invalid gitlink OID"))?
                    }
                    CapturedPayload::Bytes(bytes) => adapter::text(adapter::run(
                        &repo,
                        &["hash-object", "-w", "--stdin"],
                        None,
                        None,
                        Some(bytes),
                        None,
                        &self.options.process,
                    )?)?,
                };
                Some(Entry {
                    path: selected.path.clone(),
                    mode,
                    oid,
                })
            } else {
                None
            };
            apply(
                &repo,
                &selected_index,
                &selected.path,
                new.as_ref(),
                &meta.object_format,
                &self.options.process,
            )?;
        }
        let approved_tree = tree(&repo, &selected_index, &self.options.process)?;
        let base_tree = if matches!(&request, WriteRequest::Commit { .. }) {
            tree(&repo, &head_index, &self.options.process)?
        } else {
            original_tree
        };
        if approved_tree == base_tree {
            return Err(fail(
                ErrorKind::InvalidRequest,
                "Selection produces no change",
            ));
        }
        let selected_entries = candidate_entries(&repo, &selected_index, &meta.object_format)?;
        for path in &paths {
            apply(
                &repo,
                &reconciled,
                &path.path,
                selected_entries.iter().find(|e| e.path == path.path),
                &meta.object_format,
                &self.options.process,
            )?;
        }
        // The patch is a review display artifact. Direct operations bind and
        // revalidate the exact trees/files without generating an unused full diff.
        let patch = if include_patch {
            adapter::run(
                &repo,
                &[
                    "diff",
                    "--no-ext-diff",
                    "--no-textconv",
                    "--no-color",
                    "--binary",
                    "--submodule=short",
                    &base_tree,
                    &approved_tree,
                    "--",
                ],
                None,
                None,
                None,
                None,
                &self.options.process,
            )?
        } else {
            Vec::new()
        };
        let mut proposal = Proposal {
            operation_id: id,
            workspace_id: workspace.clone(),
            repository_id: entry.repository_id,
            worktree_id: worktree.clone(),
            request,
            branch,
            expected_head: head,
            approved_tree,
            message,
            author_name: config.name.clone(),
            author_email: config.email.clone(),
            author_date: config.date.clone(),
            hooks,
            hook_manifest: config.hooks.clone(),
            signing: config.signing.clone(),
            created_at_ms: now_ms(),
            review_digest: String::new(),
            patch,
        };
        proposal.review_digest = digest(&serde_json::to_vec(&proposal)?);
        let op = Operation {
            proposal: proposal.clone(),
            metadata: meta.clone(),
            binding: policy::binding(&meta)?,
            config,
            original_index_hash: digest(&original),
            original_entries: entries,
            selected_fingerprints: fingerprints,
            candidate_hash: digest(&inspection::file_bytes(&reconciled, 16 * 1024 * 1024)?),
            selected_index_hash: digest(&inspection::file_bytes(
                &selected_index,
                16 * 1024 * 1024,
            )?),
            new_head: None,
            phase: Phase::Proposed,
            published: false,
            reconciled: false,
            lock: None,
            head_lock: None,
            head_log_before: None,
            post_attempted: Vec::new(),
            post_results: Vec::new(),
            reason: None,
        };
        self.revalidate(&op, false)?;
        self.persist(&op)?;
        Ok(proposal)
    }
    fn revalidate(&mut self, op: &Operation, allow_lock: bool) -> Result<()> {
        self.entry(&op.proposal.workspace_id, &op.proposal.worktree_id)?;
        let root = op.metadata.root.to_path()?;
        let current = inspection::repository_metadata_with_options(&root, &self.options.process)?;
        if policy::binding(&current)? != op.binding {
            return Err(fail(
                ErrorKind::UntrustedRepository,
                "Identity/ownership changed after review",
            ));
        }
        let preference = match &op.proposal.request {
            WriteRequest::Commit { signing, .. } => *signing,
            _ => SigningPreference::Unsigned,
        };
        let config = policy::effective(&op.metadata, preference, &self.options)?;
        self.trusted(&op.proposal.worktree_id, &op.metadata, &config)?;
        if config.config_hash != op.config.config_hash
            || config.hooks != op.config.hooks
            || config.signing != op.config.signing
        {
            return Err(fail(ErrorKind::StaleReview, "Execution policy changed"));
        }
        policy::preflight_locked(&op.metadata, &self.options, allow_lock)?;
        if allow_lock
            && (!owned_head(op)?
                || !owned_lock(op, &op.metadata.git_dir.to_path()?.join("index.lock"))?)
        {
            return Err(fail(
                ErrorKind::NeedsManualReconciliation,
                "Native locks changed",
            ));
        }
        let branch = adapter::text(adapter::run(
            &root,
            &["symbolic-ref", "-q", "HEAD"],
            None,
            None,
            None,
            None,
            &self.options.process,
        )?)?;
        if branch != op.proposal.branch {
            return Err(fail(ErrorKind::StaleReview, "Checked-out branch changed"));
        }
        let output = adapter::git(
            &root,
            &inspection::strings(&["rev-parse", "--verify", "--quiet", "HEAD"]),
            None,
            None,
            None,
            None,
            &self.options.process,
        )?;
        let head = if output.status.success() {
            Some(adapter::text(output.stdout)?)
        } else if output.status.code() == Some(1) {
            None
        } else {
            return Err(fail(ErrorKind::StaleReview, "HEAD unavailable"));
        };
        if head != op.proposal.expected_head
            || digest(&policy::original_index(&op.metadata)?) != op.original_index_hash
        {
            return Err(fail(ErrorKind::StaleReview, "HEAD or index changed"));
        }
        for (path, expected) in &op.selected_fingerprints {
            let old = op.original_entries.iter().find(|e| &e.path == path);
            if capture(
                &op.metadata,
                path,
                old,
                &op.config,
                None,
                &self.options.process,
            )?
            .0 != *expected
            {
                return Err(fail(
                    ErrorKind::StaleReview,
                    "Selected working content changed",
                ));
            }
        }
        let dir = self.directory(&op.proposal.operation_id)?;
        for (name, hash) in [
            ("selected.index", &op.selected_index_hash),
            ("reconciled.index", &op.candidate_hash),
        ] {
            if digest(&inspection::file_bytes(&dir.join(name), 16 * 1024 * 1024)?) != *hash {
                return Err(fail(
                    ErrorKind::NeedsManualReconciliation,
                    "Operation artifact integrity mismatch",
                ));
            }
        }
        Ok(())
    }
}

fn owned_lock(op: &Operation, path: &Path) -> Result<bool> {
    let Some(receipt) = &op.lock else {
        return Ok(false);
    };
    if !path.exists() {
        return Ok(false);
    }
    let bytes = inspection::file_bytes(path, 16 * 1024 * 1024)?;
    Ok(inspection::fs_identity(path)? == receipt.identity && digest(&bytes) == receipt.hash)
}
fn observation(meta: &Metadata, config: &EffectiveConfig, options: &ReadOptions) -> Result<String> {
    let status = inspection::snapshot(&meta.root.to_path()?)?;
    let mut values = Vec::new();
    for change in status.changes {
        let fingerprint = capture(meta, &change.path, None, config, None, options)
            .map(|c| c.0)
            .unwrap_or_else(|_| "unavailable".into());
        values.push((change.path, fingerprint));
    }
    Ok(digest(&serde_json::to_vec(&values)?))
}
fn hook_command(
    meta: &Metadata,
    config: &EffectiveConfig,
    index: &Path,
    hook: &Hook,
    objects: Option<&Path>,
    args: &[std::ffi::OsString],
    options: &ReadOptions,
) -> Result<std::process::Output> {
    if adapter::program_fingerprint(&hook.resolved.to_path()?)? != hook.fingerprint {
        return Err(fail(ErrorKind::StaleReview, "Hook executable changed"));
    }
    let mut command = adapter::clean(&hook.resolved.to_path()?);
    command
        .current_dir(meta.root.to_path()?)
        .args(args)
        .env("GIT_INDEX_FILE", index)
        .env("GIT_DIR", meta.git_dir.to_path()?)
        .env("GIT_COMMON_DIR", meta.common_dir.to_path()?)
        .env("GIT_WORK_TREE", meta.root.to_path()?)
        .env("GIT_EDITOR", ":")
        .env("GIT_OPTIONAL_LOCKS", "0")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_COUNT", "4")
        .env("GIT_CONFIG_KEY_0", "core.hooksPath")
        .env("GIT_CONFIG_VALUE_0", "")
        .env("GIT_CONFIG_KEY_1", "core.fsmonitor")
        .env("GIT_CONFIG_VALUE_1", "false")
        .env("GIT_CONFIG_KEY_2", "maintenance.auto")
        .env("GIT_CONFIG_VALUE_2", "false")
        .env("GIT_CONFIG_KEY_3", "gc.auto")
        .env("GIT_CONFIG_VALUE_3", "0")
        .env("GIT_AUTHOR_NAME", &config.name)
        .env("GIT_AUTHOR_EMAIL", &config.email)
        .env("GIT_COMMITTER_NAME", &config.name)
        .env("GIT_COMMITTER_EMAIL", &config.email);
    if let Some(objects) = objects {
        use std::os::unix::ffi::OsStrExt;
        let alternate_path = meta.common_dir.to_path()?.join("objects");
        let mut alternate = c_quote(alternate_path.as_os_str().as_bytes());
        alternate.pop(); // Environment syntax uses a C-quoted list entry, without LF.
        command.env("GIT_OBJECT_DIRECTORY", objects).env(
            "GIT_ALTERNATE_OBJECT_DIRECTORIES",
            std::ffi::OsStr::from_bytes(&alternate),
        );
    }
    process::bounded_input(command, options, None)
}
pub(super) fn promote_loose_objects(
    db: &rusqlite::Connection,
    operations: &Path,
    source: &Path,
    meta: &Metadata,
    options: &ReadOptions,
    max_object_bytes: Option<usize>,
) -> Result<()> {
    let started = Instant::now();
    let destination = meta.common_dir.to_path()?.join("objects");
    let root = fs::symlink_metadata(&destination)?;
    if root.is_symlink() || !root.is_dir() {
        return Err(unsupported("Unsafe object storage"));
    }
    for entry in fs::read_dir(source)? {
        options.check(started)?;
        let entry = entry?;
        let name = entry.file_name();
        let name = name
            .to_str()
            .ok_or_else(|| unsupported("Invalid object directory"))?;
        if name.len() != 2 || !name.bytes().all(|b| b.is_ascii_hexdigit()) {
            continue;
        }
        let target = destination.join(name);
        if !target.exists() {
            fs::create_dir(&target)?;
            store::sync_dir(&destination)?;
        }
        let m = fs::symlink_metadata(&target)?;
        if m.is_symlink() || !m.is_dir() {
            return Err(unsupported("Unsafe object fanout"));
        }
        for file in fs::read_dir(entry.path())? {
            options.check(started)?;
            let file = file?;
            let name = file.file_name();
            let name = name
                .to_str()
                .ok_or_else(|| unsupported("Invalid object name"))?;
            let len = if meta.object_format == ObjectFormat::Sha1 {
                38
            } else {
                62
            };
            if name.len() != len || !name.bytes().all(|b| b.is_ascii_hexdigit()) {
                return Err(unsupported("Unexpected object artifact"));
            }
            let path = target.join(name);
            if path.exists() {
                let m = fs::symlink_metadata(&path)?;
                if m.is_symlink() || !m.is_file() {
                    return Err(unsupported("Unsafe existing object"));
                }
                continue;
            }
            let id = Uuid::new_v4().to_string();
            let temporary = target.join(format!(".gitfleet-object-{id}"));
            let mut input = open_regular(&file.path())?;
            let mut output = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(&temporary)?;
            if let Err(error) =
                temporary::remember(db, operations, &id, &temporary, &output.metadata()?)
            {
                fs::remove_file(&temporary)?;
                return Err(error);
            }
            let result = (|| -> Result<()> {
                let mut copied = 0u64;
                let mut buffer = [0; 65536];
                loop {
                    options.check(started)?;
                    let count = input.read(&mut buffer)?;
                    if count == 0 {
                        break;
                    }
                    copied += count as u64;
                    if max_object_bytes.is_some_and(|limit| copied > limit as u64) {
                        return Err(fail(
                            ErrorKind::OutputLimit,
                            "Imported object exceeds transfer limit",
                        ));
                    }
                    #[cfg(test)]
                    if copied > buffer.len() as u64 {
                        promotion_cleanup_tests::inject("write", options)?;
                    }
                    output.write_all(&buffer[..count])?;
                    #[cfg(test)]
                    promotion_cleanup_tests::inject("cancel", options)?;
                }
                #[cfg(test)]
                promotion_cleanup_tests::inject("sync", options)?;
                store::sync(&output)?;
                store::sync_dir(&target)?;
                // Publish an immutable object only after its complete bytes are durable.
                match fs::hard_link(&temporary, &path) {
                    Ok(()) => {}
                    Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
                    Err(error) => return Err(error.into()),
                }
                store::sync_dir(&target)?;
                #[cfg(test)]
                promotion_cleanup_tests::inject("linked", options)?;
                Ok(())
            })();
            temporary::cleanup(db, operations, &id)?;
            result?;
        }
    }
    Ok(())
}
fn validate_objects(
    repo: &Path,
    objects: Option<&Path>,
    index: &Path,
    head: Option<&str>,
    options: &ReadOptions,
) -> Result<()> {
    let mut args = vec![
        "fsck",
        "--strict",
        "--no-reflogs",
        "--no-dangling",
        "--cache",
    ];
    if let Some(head) = head {
        args.push(head);
    }
    adapter::run(repo, &args, Some(index), objects, None, None, options)?;
    Ok(())
}
fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}
fn signing_config(op: &Operation, dir: &Path) -> Result<EffectiveConfig> {
    let mut config = op.config.clone();
    if let Some(signing) = &mut config.signing {
        if signing.format == "openpgp" {
            let program = signing
                .program
                .as_ref()
                .ok_or_else(|| fail(ErrorKind::SigningFailed, "Missing signing program"))?
                .to_path()?;
            let text = format!(
                "#!/bin/sh\nexec {} --no-options --no-auto-key-retrieve \"$@\"\n",
                shell_quote(
                    program
                        .to_str()
                        .ok_or_else(|| unsupported("Non-UTF-8 signing program"))?
                )
            );
            let wrapper = dir.join("openpgp-program");
            if !wrapper.exists() {
                store::write_new(&wrapper, text.as_bytes())?;
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(&wrapper, fs::Permissions::from_mode(0o700))?;
                store::sync(&File::open(&wrapper)?)?;
            }
            if inspection::file_bytes(&wrapper, 65536)? != text.as_bytes() {
                return Err(fail(
                    ErrorKind::SigningFailed,
                    "Signing adapter integrity mismatch",
                ));
            }
            signing.program = Some(NativePath::from_path(&wrapper));
        }
    }
    Ok(config)
}
fn verify_commit(
    op: &Operation,
    repo: &Path,
    config: &EffectiveConfig,
    dir: &Path,
    options: &ReadOptions,
) -> Result<()> {
    let oid = op
        .new_head
        .as_ref()
        .ok_or_else(|| fail(ErrorKind::InvalidData, "Missing created commit"))?;
    let bytes = adapter::run(
        repo,
        &["cat-file", "commit", oid],
        None,
        None,
        None,
        None,
        options,
    )?;
    let split = bytes
        .windows(2)
        .position(|b| b == b"\n\n")
        .ok_or_else(|| fail(ErrorKind::InvalidData, "Malformed commit"))?;
    if bytes[split + 2..] != op.proposal.message {
        return Err(fail(
            ErrorKind::StaleReview,
            "Commit message differs byte-for-byte from approval",
        ));
    }
    let headers = bytes[..split].split(|b| *b == b'\n').collect::<Vec<_>>();
    let parents = headers
        .iter()
        .filter_map(|line| line.strip_prefix(b"parent "))
        .collect::<Vec<_>>();
    let expected = op
        .proposal
        .expected_head
        .as_ref()
        .map(|head| head.as_bytes())
        .into_iter()
        .collect::<Vec<_>>();
    let identity = format!(
        "{} <{}> {}",
        op.config.name, op.config.email, op.config.date
    );
    if !headers.contains(&format!("tree {}", op.proposal.approved_tree).as_bytes())
        || parents != expected
        || !headers.contains(&format!("author {identity}").as_bytes())
        || !headers.contains(&format!("committer {identity}").as_bytes())
    {
        return Err(fail(
            ErrorKind::StaleReview,
            "Commit tree/parents/identity differ from approval",
        ));
    }
    if let Some(signing) = &op.config.signing {
        let mut args = inspection::strings(&["verify-commit", "--raw", oid]);
        if signing.format == "ssh" {
            if config.email.bytes().any(|b| b.is_ascii_whitespace()) {
                return Err(fail(ErrorKind::SigningFailed, "Invalid SSH principal"));
            }
            let public = signing
                .public_key
                .as_ref()
                .ok_or_else(|| fail(ErrorKind::SigningFailed, "Missing approved SSH public key"))?;
            let path = dir.join("allowed-signers");
            let mut expected = format!("{} ", config.email).into_bytes();
            expected.extend(public);
            if !path.exists() {
                store::write_new(&path, &expected)?;
            }
            if inspection::file_bytes(&path, 65536)? != expected {
                return Err(fail(
                    ErrorKind::SigningFailed,
                    "Verifier policy was modified",
                ));
            }
            args.splice(
                0..0,
                inspection::strings(&[
                    "-c",
                    &format!("gpg.ssh.allowedSignersFile={}", path.display()),
                ]),
            );
        }
        let output = adapter::git(repo, &args, None, None, None, Some(config), options)?;
        let mut diagnostic = output.stderr;
        diagnostic.extend(output.stdout);
        let text = String::from_utf8_lossy(&diagnostic);
        let valid = if signing.format == "openpgp" {
            text.lines()
                .filter_map(|line| line.strip_prefix("[GNUPG:] VALIDSIG "))
                .any(|line| {
                    let fields = line.split_whitespace().collect::<Vec<_>>();
                    fields
                        .first()
                        .is_some_and(|value| value.eq_ignore_ascii_case(&signing.fingerprint))
                        || fields
                            .get(9)
                            .is_some_and(|value| value.eq_ignore_ascii_case(&signing.fingerprint))
                })
        } else {
            text.contains(&signing.fingerprint)
        };
        if !output.status.success() || !valid {
            return Err(fail(
                ErrorKind::SigningFailed,
                "Signature did not verify against the approved key",
            ));
        }
    } else if headers
        .iter()
        .any(|line| line.starts_with(b"gpgsig ") || line.starts_with(b"gpgsig-sha256 "))
    {
        return Err(fail(ErrorKind::SigningFailed, "Unexpected signature"));
    }
    Ok(())
}
impl ControlledWrites {
    pub fn execute(&mut self, approval: &Approval) -> Result<Outcome> {
        let mut op = self.load(&approval.operation)?;
        let row: (Option<String>, Option<String>) = self.workspace.connection().query_row(
            "SELECT approval_hash,approval_session FROM write_operations WHERE id=?1",
            [&approval.operation],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;
        if approval.session != self.session
            || row.1.as_deref() != Some(self.session.as_str())
            || row.0.as_deref() != Some(digest(approval.token.as_bytes()).as_str())
        {
            return Err(fail(
                ErrorKind::StaleReview,
                "Approval does not belong to this session",
            ));
        }
        if op.phase != Phase::Approved {
            return Ok(op.outcome());
        }
        if approval.issued.elapsed() > self.options.approval_lifetime {
            op.phase = Phase::NeedsReauthorization;
            op.reason = Some("Approval expired".into());
            self.persist(&op)?;
            return Err(fail(ErrorKind::StaleReview, "Approval expired"));
        }
        let _common = crate::locking::ExclusiveLock::acquire(
            File::open(op.metadata.common_dir.to_path()?)?,
            "Shared repository is write-locked",
        )?;
        self.ensure_fetch_import_resolved(&op.metadata.common_binding, None)?;
        self.ensure_upstream_resolved(&op.metadata.common_binding, None)?;
        let result = self.execute_inner(&mut op);
        match result {
            Ok(()) => Ok(op.outcome()),
            Err(error) => {
                let pending = matches!(
                    op.phase,
                    Phase::PublicationPending
                        | Phase::ReferenceUpdated
                        | Phase::IndexRenamed
                        | Phase::HeadLogWritten
                        | Phase::Published
                        | Phase::IndexReconciled
                        | Phase::PostProcessing
                        | Phase::PublishedNeedsReconciliation
                );
                if pending {
                    op.phase = if op.published {
                        Phase::PublishedNeedsReconciliation
                    } else {
                        Phase::UncertainOutcome
                    };
                } else {
                    op.phase = if error.kind == ErrorKind::Cancelled {
                        Phase::CancelledBeforePublication
                    } else if matches!(
                        error.kind,
                        ErrorKind::StaleReview | ErrorKind::UntrustedRepository
                    ) {
                        Phase::NeedsReauthorization
                    } else {
                        Phase::Blocked
                    };
                    let lock = op.metadata.git_dir.to_path()?.join("index.lock");
                    if owned_lock(&op, &lock)? {
                        fs::remove_file(lock)?;
                        store::sync_dir(&op.metadata.git_dir.to_path()?)?;
                    }
                    release_head(&op)?;
                }
                op.reason = Some(format!("{:?}: {}", error.kind, error));
                self.persist(&op)?;
                Ok(op.outcome())
            }
        }
    }
    fn execute_inner(&mut self, op: &mut Operation) -> Result<()> {
        self.revalidate(op, false)?;
        self.checkpoint(op, Phase::Preparing)?;
        let dir = self.directory(&op.proposal.operation_id)?;
        let repo = scratch_repo(&dir);
        let index = dir.join("selected.index");
        let message = dir.join("message.txt");
        store::write_new(&message, &op.proposal.message)?;
        if op.proposal.hooks == HookPolicy::Execute
            && matches!(&op.proposal.request, WriteRequest::Commit { .. })
        {
            self.checkpoint(op, Phase::HooksRunning)?;
            for name in ["pre-commit", "prepare-commit-msg", "commit-msg"] {
                if let Some(hook) = op
                    .config
                    .hooks
                    .iter()
                    .find(|h| h.name == name && h.executable)
                    .cloned()
                {
                    let before = observation(&op.metadata, &op.config, &self.options.process)?;
                    let args = match name {
                        "prepare-commit-msg" => {
                            vec![message.as_os_str().to_owned(), "message".into()]
                        }
                        "commit-msg" => vec![message.as_os_str().to_owned()],
                        _ => Vec::new(),
                    };
                    let output = hook_command(
                        &op.metadata,
                        &op.config,
                        &index,
                        &hook,
                        Some(&object_dir(&dir)),
                        &args,
                        &self.options.process,
                    )?;
                    if before != observation(&op.metadata, &op.config, &self.options.process)? {
                        op.post_results.push(format!("{name}: working-tree side effects observed; not added to approved content"));
                        self.persist(op)?;
                    }
                    if !output.status.success() {
                        return Err(fail(
                            ErrorKind::HookRejected,
                            "Preparatory hook rejected the operation",
                        ));
                    }
                    if tree(&repo, &index, &self.options.process)? != op.proposal.approved_tree
                        || inspection::file_bytes(&message, 16 * 1024 * 1024)?
                            != op.proposal.message
                    {
                        return Err(fail(
                            ErrorKind::StaleReview,
                            "Hook changed approved tree or message",
                        ));
                    }
                    // Metadata-only scratch refreshes are allowed after semantic validation.
                    op.selected_index_hash =
                        digest(&inspection::file_bytes(&index, 16 * 1024 * 1024)?);
                    self.revalidate(op, false)?;
                }
            }
        }
        self.revalidate(op, false)?;
        self.checkpoint(op, Phase::Revalidated)?;
        let config = signing_config(op, &dir)?;
        if matches!(&op.proposal.request, WriteRequest::Commit { .. }) {
            let mut args = vec!["commit-tree", op.proposal.approved_tree.as_str()];
            if let Some(head) = &op.proposal.expected_head {
                args.extend(["-p", head]);
            }
            if op.config.signing.is_some() {
                args.push("-S");
            }
            let output = adapter::git(
                &repo,
                &inspection::strings(&args),
                None,
                None,
                Some(op.proposal.message.clone()),
                Some(&config),
                &self.options.process,
            )?;
            if !output.status.success() {
                return Err(fail(
                    ErrorKind::SigningFailed,
                    "Commit creation/signing failed; unsigned fallback is forbidden",
                ));
            }
            op.new_head = Some(adapter::text(output.stdout)?);
            verify_commit(op, &repo, &config, &dir, &self.options.process)?;
        }
        validate_objects(
            &repo,
            None,
            &dir.join("reconciled.index"),
            op.new_head.as_deref(),
            &self.options.process,
        )?;
        self.checkpoint(op, Phase::ObjectReady)?;
        self.revalidate(op, false)?;
        let lock = op.metadata.git_dir.to_path()?.join("index.lock");
        let bytes = inspection::file_bytes(&dir.join("reconciled.index"), 16 * 1024 * 1024)?;
        if digest(&bytes) != op.candidate_hash {
            return Err(fail(
                ErrorKind::NeedsManualReconciliation,
                "Candidate index integrity mismatch",
            ));
        }
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&lock)
            .map_err(|_| fail(ErrorKind::Busy, "Index is locked"))?;
        op.lock = Some(Receipt {
            identity: inspection::fs_identity(&lock)?,
            hash: digest(&bytes),
        });
        // Persist ownership before filling the lock; a partial write will fail ownership checks.
        self.persist(op)?;
        file.write_all(&bytes)?;
        store::sync(&file)?;
        drop(file);
        store::sync_dir(&op.metadata.git_dir.to_path()?)?;
        let head_lock = op.metadata.git_dir.to_path()?.join("HEAD.lock");
        let head_bytes =
            inspection::file_bytes(&op.metadata.git_dir.to_path()?.join("HEAD"), 65536)?;
        if head_bytes != format!("ref: {}\n", op.proposal.branch).as_bytes() {
            return Err(fail(
                ErrorKind::StaleReview,
                "HEAD changed before lock acquisition",
            ));
        }
        let mut head_file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&head_lock)
            .map_err(|_| fail(ErrorKind::Busy, "HEAD is locked"))?;
        op.head_lock = Some(Receipt {
            identity: inspection::fs_identity(&head_lock)?,
            hash: digest(&head_bytes),
        });
        self.persist(op)?;
        head_file.write_all(&head_bytes)?;
        store::sync(&head_file)?;
        drop(head_file);
        store::sync_dir(&op.metadata.git_dir.to_path()?)?;
        if inspection::file_bytes(&op.metadata.git_dir.to_path()?.join("HEAD"), 65536)?
            != head_bytes
        {
            return Err(fail(
                ErrorKind::StaleReview,
                "HEAD changed during lock acquisition",
            ));
        }
        let head_log = op.metadata.git_dir.to_path()?.join("logs/HEAD");
        op.head_log_before = Some(digest(&if head_log.exists() {
            inspection::file_bytes(&head_log, 16 * 1024 * 1024)?
        } else {
            Vec::new()
        }));
        self.checkpoint(op, Phase::IndexLockWritten)?;
        self.revalidate(op, true)?;
        let operations = self.workspace.data_directory().join("operations");
        promote_loose_objects(
            self.workspace.connection(),
            &operations,
            &object_dir(&dir),
            &op.metadata,
            &self.options.process,
            None,
        )?;
        // Validate the promoted objects through the isolated reference namespace.
        // Staging must not enumerate unrelated refs (including Finder metadata).
        validate_objects(
            &repo,
            Some(&op.metadata.common_dir.to_path()?.join("objects")),
            &dir.join("reconciled.index"),
            op.new_head.as_deref(),
            &self.options.process,
        )?;
        if op.new_head.is_some() {
            verify_commit(
                op,
                &op.metadata.root.to_path()?,
                &config,
                &dir,
                &self.options.process,
            )?;
        }
        self.revalidate(op, true)?;
        self.checkpoint(op, Phase::PublicationPending)?;
        if let Some(new) = &op.new_head {
            let zero = "0".repeat(if op.metadata.object_format == ObjectFormat::Sha1 {
                40
            } else {
                64
            });
            let old = op.proposal.expected_head.as_deref().unwrap_or(&zero);
            // A detached operation context avoids Git's implicit HEAD update,
            // while the real HEAD is held by its native lock and verified above.
            let commondir = repo.join("commondir");
            store::write_new(
                &commondir,
                &[
                    op.metadata
                        .common_dir
                        .to_path()?
                        .as_os_str()
                        .as_encoded_bytes(),
                    b"\n",
                ]
                .concat(),
            )?;
            let temporary_head = repo.join("HEAD.next");
            store::write_new(&temporary_head, format!("{new}\n").as_bytes())?;
            fs::rename(temporary_head, repo.join("HEAD"))?;
            store::sync_dir(&repo)?;
            let mut command = adapter::clean(Path::new("git"));
            command
                .current_dir(&root_for(op)?)
                .env("GIT_DIR", &repo)
                .env("GIT_COMMON_DIR", op.metadata.common_dir.to_path()?)
                .env("GIT_CONFIG_NOSYSTEM", "1")
                .env("GIT_CONFIG_GLOBAL", "/dev/null")
                .env("GIT_COMMITTER_NAME", &op.config.name)
                .env("GIT_COMMITTER_EMAIL", &op.config.email)
                .env("GIT_COMMITTER_DATE", &op.config.date)
                .args([
                    "-c",
                    "core.hooksPath=",
                    "-c",
                    "core.fsmonitor=false",
                    "-c",
                    "maintenance.auto=false",
                    "-c",
                    "core.fsync=all",
                    "-c",
                    "core.fsyncMethod=fsync",
                    "update-ref",
                    "--create-reflog",
                    "-m",
                    &format!("GitFleet {}", op.proposal.operation_id),
                    &op.proposal.branch,
                    new,
                    old,
                ]);
            let output = process::bounded_input(command, &self.options.process, None)?;
            if !output.status.success() {
                return Err(fail(ErrorKind::UncertainOutcome,"Ref transaction failed or its outcome is uncertain; recovery must inspect actual state"));
            }
            self.checkpoint(op, Phase::ReferenceUpdated)?;
            op.published = true;
            self.sync_reference(op)?;
            self.checkpoint(op, Phase::Published)?;
            self.head_reflog(op)?;
            self.checkpoint(op, Phase::HeadLogWritten)?;
        }
        self.finish_index(op)?;
        self.checkpoint(op, Phase::IndexRenamed)?;
        release_head(op)?;
        self.checkpoint(op, Phase::IndexReconciled)?;
        self.post_process(op)?;
        if matches!(
            op.phase,
            Phase::PublishedWithPostProcessingFailure | Phase::NeedsManualReconciliation
        ) {
            return Ok(());
        }
        self.checkpoint(op, Phase::Complete)
    }
    fn sync_reference(&self, op: &Operation) -> Result<()> {
        let common = op.metadata.common_dir.to_path()?;
        for path in [
            common.join(&op.proposal.branch),
            common.join("logs").join(&op.proposal.branch),
            op.metadata.git_dir.to_path()?.join("logs/HEAD"),
        ] {
            if path.exists() {
                let file = OpenOptions::new()
                    .read(true)
                    .custom_flags(libc::O_NOFOLLOW)
                    .open(&path)?;
                store::sync(&file)?;
                let mut directory = path.parent().expect("ref parent");
                loop {
                    store::sync_dir(directory)?;
                    if directory == common || directory == op.metadata.git_dir.to_path()? {
                        break;
                    }
                    directory = directory
                        .parent()
                        .ok_or_else(|| unsupported("Invalid ref path"))?;
                }
            }
        }
        Ok(())
    }
    fn finish_index(&mut self, op: &mut Operation) -> Result<()> {
        let dir = op.metadata.git_dir.to_path()?;
        let lock = dir.join("index.lock");
        if !owned_lock(op, &lock)?
            || digest(&policy::original_index(&op.metadata)?) != op.original_index_hash
        {
            return Err(fail(
                ErrorKind::NeedsManualReconciliation,
                "Index changed or lock ownership cannot be established",
            ));
        }
        if let Some(new) = &op.new_head {
            let actual = adapter::text(adapter::run(
                &op.metadata.root.to_path()?,
                &["rev-parse", &op.proposal.branch],
                None,
                None,
                None,
                None,
                &self.options.process,
            )?)?;
            if &actual != new {
                return Err(fail(
                    ErrorKind::NeedsManualReconciliation,
                    "Branch moved after publication",
                ));
            }
        }
        fs::rename(&lock, dir.join("index"))?;
        store::sync(&File::open(dir.join("index"))?)?;
        store::sync_dir(&dir)?;
        op.reconciled = true;
        Ok(())
    }
    fn post_process(&mut self, op: &mut Operation) -> Result<()> {
        if op.proposal.hooks == HookPolicy::BypassAll {
            return Ok(());
        }
        let mut names = vec!["post-index-change"];
        if op.new_head.is_some() {
            names.push("post-commit");
        }
        for name in names {
            if let Some(hook) = op
                .config
                .hooks
                .iter()
                .find(|h| h.executable && h.name == name)
                .cloned()
            {
                op.post_attempted.push(name.to_owned());
                self.checkpoint(op, Phase::PostProcessing)?;
                let before = observation(&op.metadata, &op.config, &self.options.process)?;
                let args = if name == "post-index-change" {
                    inspection::strings(&["0", "0"])
                } else {
                    Vec::new()
                };
                let output = hook_command(
                    &op.metadata,
                    &op.config,
                    &op.metadata.git_dir.to_path()?.join("index"),
                    &hook,
                    None,
                    &args,
                    &self.options.process,
                );
                match output {
                    Ok(output) => {
                        op.post_results
                            .push(format!("{name}: exit={:?}", output.status.code()));
                        if !output.status.success() {
                            op.phase = Phase::PublishedWithPostProcessingFailure;
                            op.reason = Some(format!("{name} failed after index/ref publication"));
                            self.persist(op)?;
                            return Ok(());
                        }
                    }
                    Err(error) => {
                        op.phase = Phase::PublishedWithPostProcessingFailure;
                        op.reason = Some(format!("{name}: {:?} after publication", error.kind));
                        self.persist(op)?;
                        return Ok(());
                    }
                }
                if before != observation(&op.metadata, &op.config, &self.options.process)? {
                    op.post_results
                        .push(format!("{name}: working-tree side effects observed"));
                }
                if digest(&policy::original_index(&op.metadata)?) != op.candidate_hash {
                    op.phase = Phase::NeedsManualReconciliation;
                    op.reason = Some("Post-hook changed the real index".into());
                    self.persist(op)?;
                    return Ok(());
                }
                if let Some(new) = &op.new_head {
                    let actual = adapter::text(adapter::run(
                        &op.metadata.root.to_path()?,
                        &["rev-parse", &op.proposal.branch],
                        None,
                        None,
                        None,
                        None,
                        &self.options.process,
                    )?)?;
                    if &actual != new {
                        op.phase = Phase::NeedsManualReconciliation;
                        op.reason = Some("Post-hook changed the published branch".into());
                        self.persist(op)?;
                        return Ok(());
                    }
                }
            }
        }
        Ok(())
    }
}

impl ControlledWrites {
    // Reauthorization may inspect a crashed operation without deleting its proven locks.
    pub(super) fn preflight_for_trust(&mut self, meta: &Metadata) -> Result<()> {
        let git = meta.git_dir.to_path()?;
        let index = git.join("index.lock");
        let head = git.join("HEAD.lock");
        let mut owned = false;
        if index.exists() || head.exists() {
            let mut statement = self
                .workspace
                .connection()
                .prepare("SELECT payload FROM write_operations")?;
            let rows = statement.query_map([], |row| row.get::<_, String>(0))?;
            for row in rows {
                let op: Operation = serde_json::from_str(&row?)?;
                if op.metadata.git_dir == meta.git_dir
                    && op.binding == policy::binding(meta)?
                    && (!index.exists() || owned_lock(&op, &index)?)
                    && (!head.exists() || owned_head(&op)?)
                {
                    owned = true;
                    break;
                }
            }
        }
        policy::preflight_locked(meta, &self.options, owned)?;
        Ok(())
    }
    fn recovery_view(&mut self, id: &str) -> Result<RecoveryReview> {
        let op = self.load(id)?;
        if matches!(
            op.phase,
            Phase::Complete | Phase::PublishedWithPostProcessingFailure
        ) {
            return Ok(RecoveryReview{operation_id:id.into(),outcome:op.outcome(),action:"historical_complete".into(),explanation:"Already finalized; later external Git changes do not rewrite historical results".into(),fingerprint:digest(&serde_json::to_vec(&op)?) });
        }
        let current = inspection::repository_metadata(&op.metadata.root.to_path()?)?;
        let binding = policy::binding(&current)?;
        let root = op.metadata.root.to_path()?;
        let branch = adapter::text(adapter::run(
            &root,
            &["symbolic-ref", "-q", "HEAD"],
            None,
            None,
            None,
            None,
            &self.options.process,
        )?)?;
        let output = adapter::git(
            &root,
            &inspection::strings(&["rev-parse", "--verify", "--quiet", &op.proposal.branch]),
            None,
            None,
            None,
            None,
            &self.options.process,
        )?;
        let actual = if output.status.success() {
            Some(adapter::text(output.stdout)?)
        } else if output.status.code() == Some(1) {
            None
        } else {
            return Err(fail(
                ErrorKind::NeedsManualReconciliation,
                "Reference unavailable during recovery",
            ));
        };
        let index_hash = digest(&policy::original_index(&op.metadata)?);
        let lock = op.metadata.git_dir.to_path()?.join("index.lock");
        let owned = owned_lock(&op, &lock).unwrap_or(false);
        let lock_state = if lock.exists() {
            Some((
                inspection::fs_identity(&lock).unwrap_or_default(),
                inspection::file_bytes(&lock, 16 * 1024 * 1024)
                    .map(|b| digest(&b))
                    .unwrap_or_default(),
            ))
        } else {
            None
        };
        policy::safe_metadata_paths(&op.metadata)?;
        let head_path = op.metadata.git_dir.to_path()?.join("HEAD.lock");
        let head_owned = owned_head(&op)?;
        let head_state = if head_path.exists() {
            Some((
                inspection::fs_identity(&head_path)?,
                digest(&inspection::file_bytes(&head_path, 65536)?),
            ))
        } else {
            None
        };
        let log_path = op
            .metadata
            .common_dir
            .to_path()?
            .join("logs")
            .join(&op.proposal.branch);
        let log = if log_path.exists() {
            inspection::file_bytes(&log_path, 16 * 1024 * 1024)?
        } else {
            Vec::new()
        };
        let head_log_path = op.metadata.git_dir.to_path()?.join("logs/HEAD");
        let head_log_hash = digest(&if head_log_path.exists() {
            inspection::file_bytes(&head_log_path, 16 * 1024 * 1024)?
        } else {
            Vec::new()
        });
        let evidence = op.new_head.as_ref().is_some_and(|new| {
            log.split(|b| *b == b'\n').any(|line| {
                line.starts_with(
                    format!(
                        "{} {new} ",
                        op.proposal
                            .expected_head
                            .clone()
                            .unwrap_or_else(|| "0".repeat(new.len()))
                    )
                    .as_bytes(),
                ) && line.ends_with(format!("\tGitFleet {}", op.proposal.operation_id).as_bytes())
            })
        });
        let action = if binding != op.binding
            || branch != op.proposal.branch
            || (head_path.exists() && !head_owned)
        {
            "manual"
        } else if op.new_head.is_none() {
            if index_hash == op.candidate_hash && !lock.exists() {
                "recognize_index"
            } else if index_hash == op.original_index_hash && (!lock.exists() || owned) {
                "abort_unpublished"
            } else {
                "manual"
            }
        } else if actual == op.new_head && (evidence || op.published) {
            if index_hash == op.candidate_hash && !lock.exists() {
                "recognize_commit"
            } else if index_hash == op.original_index_hash && owned {
                "reconcile_index"
            } else {
                "manual"
            }
        } else if actual == op.proposal.expected_head
            && !evidence
            && !op.published
            && index_hash == op.original_index_hash
            && (!lock.exists() || owned)
        {
            "abort_unpublished"
        } else {
            "manual"
        };
        let explanation=match action{"abort_unpublished"=>"No publication evidence; abort and remove only a proven owned lock. Objects may remain unreferenced.","reconcile_index"=>"Commit is published; promote only the proven owned index. No commit or hook will be replayed.","recognize_commit"|"recognize_index"=>"Publication/reconciliation is already present. Record it without rerunning commits or hooks.",_=>"Unexpected identity/ref/index/lock state; manual reconciliation is required. No repository writes are permitted."}.to_owned();
        let fingerprint = digest(&serde_json::to_vec(&(
            &op,
            &binding,
            &branch,
            &actual,
            &index_hash,
            &lock_state,
            &head_state,
            &head_log_hash,
            digest(&log),
            evidence,
        ))?);
        Ok(RecoveryReview {
            operation_id: id.into(),
            outcome: op.outcome(),
            action: action.into(),
            explanation,
            fingerprint,
        })
    }
    pub fn inspect_recovery(&mut self, id: &str) -> Result<RecoveryReview> {
        let review = self.recovery_view(id)?;
        self.recovery_reviews.insert(id.into(), review.clone());
        Ok(review)
    }
    pub fn recover(&mut self, review: &RecoveryReview) -> Result<Outcome> {
        let expected = self
            .recovery_reviews
            .remove(&review.operation_id)
            .ok_or_else(|| {
                fail(
                    ErrorKind::StaleReview,
                    "Recovery review is not owned by this session",
                )
            })?;
        let current = self.recovery_view(&review.operation_id)?;
        if serde_json::to_vec(review)? != serde_json::to_vec(&expected)?
            || current.fingerprint != review.fingerprint
        {
            return Err(fail(
                ErrorKind::StaleReview,
                "Recovery state/review changed",
            ));
        }
        let mut op = self.load(&review.operation_id)?;
        if review.action == "historical_complete" {
            return Ok(op.outcome());
        }
        if review.action == "manual" {
            op.phase = Phase::NeedsManualReconciliation;
            op.reason = Some(review.explanation.clone());
            self.persist(&op)?;
            return Ok(op.outcome());
        }
        let _common = crate::locking::ExclusiveLock::acquire(
            File::open(op.metadata.common_dir.to_path()?)?,
            "Shared repository is locked",
        )?;
        self.ensure_fetch_import_resolved(&op.metadata.common_binding, None)?;
        self.ensure_upstream_resolved(&op.metadata.common_binding, None)?;
        // Recheck after the cross-process lock, before any managed-repository mutation.
        if self.recovery_view(&review.operation_id)?.fingerprint != review.fingerprint {
            return Err(fail(
                ErrorKind::StaleReview,
                "Recovery changed while acquiring its lock",
            ));
        }
        self.entry(&op.proposal.workspace_id, &op.proposal.worktree_id)?;
        let preference = match &op.proposal.request {
            WriteRequest::Commit { signing, .. } => *signing,
            _ => SigningPreference::Unsigned,
        };
        let config = policy::effective(&op.metadata, preference, &self.options)?;
        self.trusted(&op.proposal.worktree_id, &op.metadata, &config)?;
        if review.action == "abort_unpublished" {
            let lock = op.metadata.git_dir.to_path()?.join("index.lock");
            if lock.exists() {
                if !owned_lock(&op, &lock)? {
                    return Err(fail(
                        ErrorKind::NeedsManualReconciliation,
                        "Foreign index lock",
                    ));
                }
                fs::remove_file(lock)?;
                store::sync_dir(&op.metadata.git_dir.to_path()?)?;
            }
            release_head(&op)?;
            op.phase = Phase::CancelledBeforePublication;
            op.reason =
                Some("Aborted without replay; unreferenced objects/artifacts retained".into());
            self.persist(&op)?;
            return Ok(op.outcome());
        }
        let dir = self.directory(&op.proposal.operation_id)?;
        if digest(&inspection::file_bytes(
            &dir.join("reconciled.index"),
            16 * 1024 * 1024,
        )?) != op.candidate_hash
        {
            return Err(fail(
                ErrorKind::NeedsManualReconciliation,
                "Recovery artifact is corrupt",
            ));
        }
        if op.new_head.is_some() {
            let signing = signing_config(&op, &dir)?;
            verify_commit(
                &op,
                &op.metadata.root.to_path()?,
                &signing,
                &dir,
                &self.options.process,
            )?;
            op.published = true;
            self.sync_reference(&op)?;
            self.head_reflog(&op)?;
        }
        if review.action == "reconcile_index" {
            self.finish_index(&mut op)?;
        } else {
            op.reconciled = true;
        }
        release_head(&op)?;
        let post_uncertain = !op.post_attempted.is_empty()
            && matches!(op.phase, Phase::PostProcessing | Phase::UncertainOutcome);
        op.post_results.push(
            "Post-processing skipped by explicit recovery; no hooks or commit creation replayed"
                .into(),
        );
        op.phase = if post_uncertain {
            Phase::UncertainOutcome
        } else {
            Phase::Complete
        };
        op.reason = if post_uncertain {
            Some("Post-hook attempt outcome is uncertain".into())
        } else {
            None
        };
        self.persist(&op)?;
        Ok(op.outcome())
    }
}

fn root_for(op: &Operation) -> Result<PathBuf> {
    op.metadata.root.to_path()
}
fn owned_head(op: &Operation) -> Result<bool> {
    let Some(receipt) = &op.head_lock else {
        return Ok(false);
    };
    let path = op.metadata.git_dir.to_path()?.join("HEAD.lock");
    if !path.exists() {
        return Ok(false);
    }
    Ok(inspection::fs_identity(&path)? == receipt.identity
        && digest(&inspection::file_bytes(&path, 65536)?) == receipt.hash)
}
fn release_head(op: &Operation) -> Result<()> {
    let path = op.metadata.git_dir.to_path()?.join("HEAD.lock");
    if op.head_lock.is_none() {
        return Ok(());
    }
    if !path.exists() {
        return Ok(());
    }
    if !owned_head(op)? {
        return Err(fail(
            ErrorKind::NeedsManualReconciliation,
            "Cannot establish HEAD lock ownership",
        ));
    }
    fs::remove_file(path)?;
    store::sync_dir(&op.metadata.git_dir.to_path()?)
}
impl ControlledWrites {
    fn head_reflog(&mut self, op: &Operation) -> Result<()> {
        let new = op
            .new_head
            .as_ref()
            .ok_or_else(|| unsupported("Missing publication OID"))?;
        let old = op
            .proposal
            .expected_head
            .clone()
            .unwrap_or_else(|| "0".repeat(new.len()));
        let line = format!(
            "{old} {new} {} <{}> {}\tGitFleet {}\n",
            op.config.name, op.config.email, op.config.date, op.proposal.operation_id
        );
        let directory = op.metadata.git_dir.to_path()?.join("logs");
        if directory.exists() && fs::symlink_metadata(&directory)?.is_symlink() {
            return Err(unsupported("Unsafe reflog directory"));
        }
        let path = directory.join("HEAD");
        let previous = if path.exists() {
            inspection::file_bytes(&path, 16 * 1024 * 1024)?
        } else {
            Vec::new()
        };
        if previous.ends_with(line.as_bytes()) {
            return Ok(());
        }
        if !owned_head(op)? {
            return Err(fail(
                ErrorKind::NeedsManualReconciliation,
                "HEAD reflog requires the owned native HEAD lock",
            ));
        }
        if op.head_log_before.as_deref() != Some(digest(&previous).as_str()) {
            return Err(fail(
                ErrorKind::NeedsManualReconciliation,
                "HEAD reflog changed or has a partial write",
            ));
        }
        if !directory.exists() {
            fs::create_dir(&directory)?;
            store::sync_dir(&op.metadata.git_dir.to_path()?)?;
        }
        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW)
            .open(&path)?;
        file.write_all(line.as_bytes())?;
        store::sync(&file)?;
        store::sync_dir(&directory)?;
        Ok(())
    }
}

#[cfg(test)]
mod promotion_cleanup_tests {
    use super::*;
    thread_local! { static FAULT: std::cell::Cell<&'static str> = const {std::cell::Cell::new("")}; }
    pub(super) fn inject(point: &str, options: &ReadOptions) -> Result<()> {
        if FAULT.get() == point {
            match point {
                "cancel" => options.cancellation.cancel(),
                "linked" => std::process::exit(71),
                _ => return Err(std::io::Error::from_raw_os_error(libc::ENOSPC).into()),
            }
        }
        Ok(())
    }

    #[test]
    fn interrupted_copy_preserves_foreign_fanout_file() {
        const CHILD: &str = "GITFLEET_F13_PROMOTION_CHILD";
        if let Some(root) = std::env::var_os(CHILD) {
            let root = PathBuf::from(root);
            let meta = inspection::repository_metadata(&root).unwrap();
            let db = rusqlite::Connection::open(root.join("test.db")).unwrap();
            FAULT.set("linked");
            promote_loose_objects(
                &db,
                &root,
                &root.join("source"),
                &meta,
                &Default::default(),
                None,
            )
            .unwrap();
            panic!("crash point missed");
        }
        let root = std::env::temp_dir().join(format!("promotion-{}", Uuid::new_v4()));
        fs::create_dir(&root).unwrap();
        assert!(std::process::Command::new("git")
            .args(["init", "-q"])
            .arg(&root)
            .status()
            .unwrap()
            .success());
        let meta = inspection::repository_metadata(&root).unwrap();
        let source = root.join("source");
        let fanout = root.join(".git/objects/ab");
        fs::create_dir_all(source.join("ab")).unwrap();
        fs::create_dir(&fanout).unwrap();
        let name = "1".repeat(38);
        fs::write(source.join("ab").join(&name), vec![42; 131072]).unwrap();
        let foreign = fanout.join(".gitfleet-object-foreign");
        fs::write(&foreign, b"keep").unwrap();
        let db = rusqlite::Connection::open(root.join("test.db")).unwrap();
        db.execute_batch("CREATE TABLE state(key TEXT PRIMARY KEY,value TEXT)")
            .unwrap();
        assert_eq!(
            promote_loose_objects(&db, &root, &source, &meta, &Default::default(), Some(65536))
                .unwrap_err()
                .kind,
            ErrorKind::OutputLimit
        );
        for point in ["write", "sync", "cancel"] {
            FAULT.set(point);
            assert!(
                promote_loose_objects(&db, &root, &source, &meta, &Default::default(), None)
                    .is_err()
            );
            assert_eq!(fs::read_dir(&fanout).unwrap().count(), 1);
        }
        FAULT.set("");
        let status = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact","writes::transaction::promotion_cleanup_tests::interrupted_copy_preserves_foreign_fanout_file"])
            .env(CHILD,&root).status().unwrap();
        assert_eq!(status.code(), Some(71));
        assert_eq!(fs::read_dir(&fanout).unwrap().count(), 3);
        temporary::recover(&db, &root).unwrap();
        assert_eq!(fs::read_dir(&fanout).unwrap().count(), 2);
        assert_eq!(
            fs::read(fanout.join(&name)).unwrap(),
            fs::read(source.join("ab").join(&name)).unwrap()
        );
        promote_loose_objects(&db, &root, &source, &meta, &Default::default(), None).unwrap();
        assert_eq!(fs::read(&foreign).unwrap(), b"keep");
        // A foreign file can replace the exact recorded name before restart.
        let id = Uuid::new_v4().to_string();
        let temporary = fanout.join(format!(".gitfleet-object-{id}"));
        let original = File::create_new(&temporary).unwrap();
        temporary::remember(&db, &root, &id, &temporary, &original.metadata().unwrap()).unwrap();
        fs::remove_file(&temporary).unwrap();
        fs::write(&temporary, b"replacement").unwrap();
        temporary::recover(&db, &root).unwrap();
        assert_eq!(fs::read(&temporary).unwrap(), b"replacement");
        fs::remove_dir_all(root).unwrap();
    }
}
