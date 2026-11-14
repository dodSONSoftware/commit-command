//! Offline object import and one exact tracking-ref transaction have fresh native authority.
use super::*;
use crate::remote::{fetch::*, observation::ObservationPlan};
use std::io::Read;
use std::os::unix::fs::OpenOptionsExt;

#[derive(Serialize, Deserialize)]
struct StoredImport {
    version: u32,
    session: String,
    value: FetchImport,
    receipt: Option<Receipt>,
}
fn stale() -> Error {
    fail(ErrorKind::StaleReview, "Fetch import review is stale")
}
fn unresolved(phase: FetchImportPhase) -> bool {
    matches!(
        phase,
        FetchImportPhase::ImportingObjects
            | FetchImportPhase::PublicationPending
            | FetchImportPhase::ReferencePrepared
            | FetchImportPhase::ReferenceUpdated
            | FetchImportPhase::Uncertain
    )
}
fn safe_ref_path(plan: &FetchImportPlan, log: bool) -> Result<PathBuf> {
    let mut path = plan.transport.metadata.common_dir.to_path()?;
    let relative = if log {
        format!("logs/{}", plan.tracking_ref)
    } else {
        plan.tracking_ref.clone()
    };
    for part in relative.split('/') {
        if part.is_empty() || part == "." || part == ".." {
            return Err(stale());
        }
        if path.is_dir() {
            for entry in fs::read_dir(&path)? {
                let name = entry?.file_name();
                if name
                    .to_str()
                    .is_some_and(|name| name != part && name.eq_ignore_ascii_case(part))
                {
                    return Err(unsupported("Case-aliased ref storage is unsupported"));
                }
            }
        }
        path.push(part);
        match fs::symlink_metadata(&path) {
            Ok(meta) if meta.is_symlink() || (!meta.is_dir() && !meta.is_file()) => {
                return Err(unsupported(
                    "Symlinked or special ref storage is unsupported",
                ))
            }
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.into()),
        }
    }
    Ok(path)
}
fn lock_path(plan: &FetchImportPlan) -> Result<PathBuf> {
    let mut path = safe_ref_path(plan, false)?.into_os_string();
    path.push(".lock");
    Ok(path.into())
}
fn offline_command(
    plan: &ObservationPlan,
    cwd: &Path,
    home: &Path,
) -> Result<std::process::Command> {
    let mut command = remote::command(plan, cwd, home)?;
    command.env("GIT_ALLOW_PROTOCOL", "");
    Ok(command)
}
fn remaining(options: &ReadOptions, started: Instant) -> Result<ReadOptions> {
    options.check(started)?;
    let mut copy = options.clone();
    copy.timeout = copy.timeout.saturating_sub(started.elapsed());
    Ok(copy)
}
impl ControlledWrites {
    fn persist_import(&mut self, stored: &StoredImport) -> Result<()> {
        let raw = serde_json::to_string(stored)?;
        if raw.len() > 1024 * 1024 {
            return Err(fail(
                ErrorKind::OutputLimit,
                "Fetch import record exceeds 1 MiB",
            ));
        }
        self.workspace.connection().execute(
            "INSERT OR REPLACE INTO state(key,value) VALUES(?1,?2)",
            params![
                format!("fetch-import:{}", stored.value.plan.transport.id),
                raw
            ],
        )?;
        Ok(())
    }
    fn import(&mut self, id: &str) -> Result<StoredImport> {
        Uuid::parse_str(id).map_err(|_| stale())?;
        let raw: Option<String> = self
            .workspace
            .connection()
            .query_row(
                "SELECT value FROM state WHERE key=?1",
                [format!("fetch-import:{id}")],
                |r| r.get(0),
            )
            .optional()?;
        let raw = raw.ok_or_else(stale)?;
        if raw.len() > 1024 * 1024 {
            return Err(fail(
                ErrorKind::OutputLimit,
                "Fetch import record exceeds 1 MiB",
            ));
        }
        let stored: StoredImport = serde_json::from_str(&raw)?;
        if stored.version != 1 {
            return Err(fail(
                ErrorKind::NewerSchema,
                "Unsupported fetch import record",
            ));
        }
        if stored.value.plan.transport.id != id {
            return Err(stale());
        }
        Ok(stored)
    }
    fn ensure_local_writes_resolved(&mut self, common: &str) -> Result<()> {
        let pending:bool=self.workspace.connection().query_row("SELECT EXISTS(SELECT 1 FROM write_operations WHERE json_extract(payload,'$.metadata.common_binding')=?1 AND json_extract(payload,'$.phase') NOT IN ('complete','blocked','published_with_post_processing_failure','cancelled_before_publication','proposed','approved','needs_reauthorization'))",[common],|r|r.get(0))?;
        if pending {
            return Err(fail(
                ErrorKind::Busy,
                "Resolve interrupted local writes before importing a captured branch",
            ));
        }
        Ok(())
    }
    fn bundle_snapshot(&self, capture: &FetchCapture) -> Result<Vec<u8>> {
        let artifact = capture.artifact.as_ref().ok_or_else(stale)?;
        let mut path = self.workspace.data_directory().join("network");
        for append in [None, Some(capture.plan.transport.id.as_str())] {
            if let Some(part) = append {
                path.push(part);
            }
            if !fs::symlink_metadata(&path)?.file_type().is_dir() {
                return Err(unsupported("Unsafe captured bundle directory"));
            }
        }
        path.push("capture.bundle");
        let mut file = OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW)
            .open(&path)?;
        let metadata = file.metadata()?;
        if !metadata.is_file()
            || metadata.len() != artifact.bundle_bytes
            || metadata.len() > fetch::OBJECT_BYTES
        {
            return Err(stale());
        }
        let mut bytes = Vec::new();
        Read::by_ref(&mut file)
            .take(fetch::OBJECT_BYTES + 1)
            .read_to_end(&mut bytes)?;
        if bytes.len() as u64 != artifact.bundle_bytes || digest(&bytes) != artifact.bundle_digest {
            return Err(stale());
        }
        Ok(bytes)
    }
    fn read_tracking(
        &self,
        plan: &FetchImportPlan,
        options: &ReadOptions,
    ) -> Result<(Option<crate::domain::ObjectId>, Option<String>)> {
        let current = inspection::repository_metadata_with_options(
            &plan.transport.metadata.root.to_path()?,
            options,
        )?;
        let expected = &plan.transport.metadata;
        if current.root_binding != expected.root_binding
            || current.git_binding != expected.git_binding
            || current.common_binding != expected.common_binding
        {
            return Err(stale());
        }
        policy::safe_metadata_paths(expected)?;
        safe_ref_path(plan, false)?;
        safe_ref_path(plan, true)?;
        let home = self.workspace.data_directory();
        let root = plan.transport.metadata.root.to_path()?;
        let mut command = offline_command(&plan.transport, &root, home)?;
        command.args(["symbolic-ref", "-q", &plan.tracking_ref]);
        let result = process::bounded_input(command, options, None)?;
        let symbolic = match result.status.code() {
            Some(0) => Some(inspection::output_text(result.stdout)?),
            Some(1) => None,
            _ => return Err(stale()),
        };
        let mut command = offline_command(&plan.transport, &root, home)?;
        command.args(["show-ref", "--verify", "--hash", &plan.tracking_ref]);
        let result = process::bounded_input(command, options, None)?;
        let oid = if result.status.success() {
            Some(crate::domain::ObjectId::parse(
                &inspection::output_text(result.stdout)?,
                &plan.transport.metadata.object_format,
            )?)
        } else if matches!(result.status.code(), Some(1) | Some(128)) && result.stdout.is_empty() {
            // show-ref reports missing and malformed refs alike: prove absence separately.
            let mut command = offline_command(&plan.transport, &root, home)?;
            command.args(["show-ref", "--exists", &plan.tracking_ref]);
            if process::bounded_input(command, options, None)?
                .status
                .code()
                != Some(2)
            {
                return Err(stale());
            }
            None
        } else {
            return Err(stale());
        };
        Ok((oid, symbolic))
    }
    pub(crate) fn propose_import(
        &mut self,
        workspace: &WorkspaceId,
        entry: &crate::workspace::StoredWorktree,
        capture_id: &str,
        options: &ReadOptions,
    ) -> Result<FetchImportPlan> {
        self.ensure_fetch_import_resolved(&entry.metadata.common_binding, None)?;
        self.ensure_upstream_resolved(&entry.metadata.common_binding, None)?;
        self.ensure_local_writes_resolved(&entry.metadata.common_binding)?;
        let capture = self.completed_capture(capture_id, workspace, &entry.id)?;
        self.bundle_snapshot(&capture)?;
        let previous = std::mem::replace(&mut self.options.process, options.clone());
        let result = self.prepare_transport_plan(
            workspace,
            entry,
            &capture.plan.transport.remote,
            &capture.plan.transport.reference,
            options,
        );
        self.options.process = previous;
        let transport = result?;
        if transport.config_digest != capture.plan.transport.config_digest
            || transport.generation != capture.plan.transport.generation
        {
            return Err(stale());
        }
        let branch = transport
            .reference
            .strip_prefix("refs/heads/")
            .ok_or_else(stale)?;
        if branch == "HEAD" || !branch.is_ascii() || !transport.remote.is_ascii() {
            return Err(unsupported(
                "Fetch publication currently requires an ASCII remote and ASCII branch other than HEAD",
            ));
        }
        let tracking_ref = format!("refs/remotes/{}/{branch}", transport.remote);
        let artifact = capture.artifact.as_ref().ok_or_else(stale)?;
        let mut plan = FetchImportPlan {
            transport,
            capture_id: capture_id.into(),
            expected_oid: capture.plan.expected_oid,
            bundle_digest: artifact.bundle_digest.clone(),
            bundle_bytes: artifact.bundle_bytes,
            tracking_ref,
            expected_old: None,
        };
        let mut command = offline_command(
            &plan.transport,
            &entry.metadata.root.to_path()?,
            self.workspace.data_directory(),
        )?;
        command.args(["check-ref-format", &plan.tracking_ref]);
        if !process::bounded_input(command, options, None)?
            .status
            .success()
        {
            return Err(stale());
        }
        let (old, symbolic) = self.read_tracking(&plan, options)?;
        if symbolic.is_some() {
            return Err(unsupported("Symbolic tracking refs cannot be replaced"));
        }
        if old.as_ref() == Some(&plan.expected_oid) {
            return Err(fail(
                ErrorKind::InvalidRequest,
                "Tracking ref already matches this capture",
            ));
        }
        plan.expected_old = old;
        self.persist_import(&StoredImport {
            version: 1,
            session: self.session.clone(),
            receipt: None,
            value: FetchImport {
                plan: plan.clone(),
                phase: FetchImportPhase::Proposed,
                objects_imported: false,
                reference_status: FetchReferenceStatus::NotAttempted,
                issue: None,
            },
        })?;
        Ok(plan)
    }
    pub(crate) fn cancel_import(&mut self, id: &str) -> Result<()> {
        let mut stored = self.import(id)?;
        if stored.value.phase == FetchImportPhase::Proposed {
            stored.value.phase = FetchImportPhase::Cancelled;
            self.persist_import(&stored)?;
        }
        Ok(())
    }
    #[cfg(feature = "write-test-support")]
    fn import_test_checkpoint(&mut self, value: &FetchImport) -> Result<()> {
        if !self
            .import_fault
            .as_ref()
            .is_some_and(|f| f.0 == value.phase)
        {
            return Ok(());
        }
        let root = value.plan.transport.metadata.root.to_path()?;
        if !root.starts_with(fs::canonicalize(std::env::temp_dir())?)
            || !root.join(".gitfleet-m2-fixture").is_file()
        {
            return Err(fail(
                ErrorKind::InvalidRequest,
                "Fault injection requires a disposable fixture",
            ));
        }
        match self.import_fault.take().expect("checked fault").1 {
            TestFaultKind::Sigkill => {
                unsafe {
                    libc::kill(libc::getpid(), libc::SIGKILL);
                };
                loop {
                    std::thread::park();
                }
            }
            _ => Err(fail(ErrorKind::Storage, "Injected import storage failure")),
        }
    }
    #[cfg(feature = "write-test-support")]
    pub(crate) fn set_import_fault(&mut self, phase: FetchImportPhase, kind: TestFaultKind) {
        self.import_fault = Some((phase, kind));
    }
    fn import_checkpoint(
        &mut self,
        stored: &mut StoredImport,
        phase: FetchImportPhase,
    ) -> Result<()> {
        stored.value.phase = phase;
        self.persist_import(stored)?;
        #[cfg(feature = "write-test-support")]
        self.import_test_checkpoint(&stored.value)?;
        Ok(())
    }
    pub(crate) fn execute_import(
        &mut self,
        plan: &FetchImportPlan,
        cancellation: inspection::Cancellation,
    ) -> Result<FetchImport> {
        let previous = std::mem::replace(&mut self.options.process.cancellation, cancellation);
        let result = self.execute_import_inner(plan);
        self.options.process.cancellation = previous;
        result
    }
    fn execute_import_inner(&mut self, plan: &FetchImportPlan) -> Result<FetchImport> {
        let mut stored = self.import(&plan.transport.id)?;
        if stored.session != self.session
            || stored.value.phase != FetchImportPhase::Proposed
            || serde_json::to_vec(&stored.value.plan)? != serde_json::to_vec(plan)?
        {
            return Err(stale());
        }
        let _common = crate::locking::ExclusiveLock::acquire(
            File::open(plan.transport.metadata.common_dir.to_path()?)?,
            "Shared repository is write-locked",
        )?;
        self.ensure_fetch_import_resolved(
            &plan.transport.metadata.common_binding,
            Some(&plan.transport.id),
        )?;
        self.ensure_upstream_resolved(&plan.transport.metadata.common_binding, None)?;
        self.ensure_local_writes_resolved(&plan.transport.metadata.common_binding)?;
        let options = self.options.process.clone();
        let started = Instant::now();
        let result = (|| {
            self.revalidate_transport(&plan.transport, &options)?;
            if self.read_tracking(plan, &options)? != (plan.expected_old.clone(), None) {
                return Err(stale());
            }
            let capture = self.completed_capture(
                &plan.capture_id,
                &plan.transport.workspace,
                &plan.transport.worktree,
            )?;
            if capture.plan.expected_oid != plan.expected_oid
                || capture.artifact.as_ref().is_none_or(|a| {
                    a.bundle_digest != plan.bundle_digest || a.bundle_bytes != plan.bundle_bytes
                })
            {
                return Err(stale());
            }
            let snapshot = self.bundle_snapshot(&capture)?;
            let (directory, home, repository) =
                self.initialize_transport(&plan.transport, &options)?;
            let bundle = directory.join("approved.bundle");
            store::write_new(&bundle, &snapshot)?;
            let run = |cwd: &Path, args: &[&str], input: Option<Vec<u8>>| -> Result<Vec<u8>> {
                let mut command = offline_command(&plan.transport, cwd, &home)?;
                command.args(args);
                let output = process::bounded_input_checked(
                    command,
                    &remaining(&options, started)?,
                    input,
                    || fetch::check_artifact_budget(&directory, 256 * 1024 * 1024, 20_000),
                )?;
                if !output.status.success() {
                    return Err(fail(
                        ErrorKind::InspectionFailed,
                        "Offline fetch import validation failed; no raw Git diagnostics retained",
                    ));
                }
                Ok(output.stdout)
            };
            let bundle = bundle
                .to_str()
                .ok_or_else(|| unsupported("Import artifact path must be UTF-8"))?;
            run(&repository, &["bundle", "verify", bundle], None)?;
            let heads = inspection::output_text(run(
                &repository,
                &["bundle", "list-heads", bundle],
                None,
            )?)?;
            if heads != format!("{} {}", plan.expected_oid.hex, fetch::CAPTURE_REF) {
                return Err(stale());
            }
            run(&repository, &["bundle", "unbundle", bundle], None)?;
            run(
                &repository,
                &[
                    "fsck",
                    "--strict",
                    "--full",
                    "--no-reflogs",
                    "--no-dangling",
                    &plan.expected_oid.hex,
                ],
                None,
            )?;
            fetch::object_inventory(
                &run(
                    &repository,
                    &[
                        "cat-file",
                        "--batch-all-objects",
                        "--batch-check=%(objectname) %(objectsize)",
                    ],
                    None,
                )?,
                &plan.transport.metadata.object_format,
            )?;
            let packs = fs::read_dir(repository.join("objects/pack"))?
                .map(|e| e.map(|e| e.path()))
                .collect::<std::io::Result<Vec<_>>>()?;
            let packs = packs
                .iter()
                .filter(|p| p.extension().is_some_and(|e| e == "pack"))
                .collect::<Vec<_>>();
            if packs.len() != 1 {
                return Err(unsupported("Expected one standalone bundle pack"));
            }
            let pack = inspection::file_bytes(packs[0], fetch::OBJECT_BYTES as usize)?;
            let loose = directory.join("loose.git");
            fetch::check_artifact_budget(&directory, 256 * 1024 * 1024, 20_000)?;
            self.initialize_bare_repository(
                &plan.transport,
                &directory,
                &home,
                &loose,
                &remaining(&options, started)?,
            )?;
            // Reject every persistent link/special file before untrusted data processing.
            fetch::check_artifact_budget(&directory, 256 * 1024 * 1024, 20_000)?;
            run(&loose, &["unpack-objects", "--strict"], Some(pack))?;
            run(
                &loose,
                &[
                    "fsck",
                    "--strict",
                    "--full",
                    "--no-reflogs",
                    "--no-dangling",
                    &plan.expected_oid.hex,
                ],
                None,
            )?;
            self.revalidate_transport(&plan.transport, &remaining(&options, started)?)?;
            self.import_checkpoint(&mut stored, FetchImportPhase::ImportingObjects)?;
            let operations = self.workspace.data_directory().join("operations");
            transaction::promote_loose_objects(
                self.workspace.connection(),
                &operations,
                &loose.join("objects"),
                &plan.transport.metadata,
                &remaining(&options, started)?,
                Some(65 * 1024 * 1024),
            )?;
            stored.value.objects_imported = true;
            self.persist_import(&stored)?;
            let root = plan.transport.metadata.root.to_path()?;
            run(
                &root,
                &[
                    "fsck",
                    "--strict",
                    "--full",
                    "--no-reflogs",
                    "--no-dangling",
                    &plan.expected_oid.hex,
                ],
                None,
            )?;
            self.revalidate_transport(&plan.transport, &remaining(&options, started)?)?;
            if self.read_tracking(plan, &remaining(&options, started)?)?
                != (plan.expected_old.clone(), None)
            {
                return Err(stale());
            }
            // Detached publication context avoids an implicit real HEAD reflog update.
            store::write_new(
                &repository.join("commondir"),
                &[
                    plan.transport
                        .metadata
                        .common_dir
                        .to_path()?
                        .as_os_str()
                        .as_encoded_bytes(),
                    b"\n",
                ]
                .concat(),
            )?;
            fs::write(
                repository.join("HEAD"),
                format!("{}\n", plan.expected_oid.hex),
            )?;
            let mut command = offline_command(&plan.transport, &repository, &home)?;
            command
                .env(
                    "GIT_COMMON_DIR",
                    plan.transport.metadata.common_dir.to_path()?,
                )
                .env("GIT_COMMITTER_NAME", "GitFleet")
                .env("GIT_COMMITTER_EMAIL", "gitfleet@localhost.invalid")
                .args([
                    "-c",
                    "core.fsync=all",
                    "-c",
                    "core.fsyncMethod=fsync",
                    "update-ref",
                    "--no-deref",
                    "--create-reflog",
                    "-m",
                    &format!("GitFleet fetch {}", plan.transport.id),
                    "--stdin",
                ]);
            self.import_checkpoint(&mut stored, FetchImportPhase::PublicationPending)?;
            let zero = "0".repeat(
                if plan.transport.metadata.object_format == ObjectFormat::Sha1 {
                    40
                } else {
                    64
                },
            );
            let old = plan
                .expected_old
                .as_ref()
                .map(|v| v.hex.as_str())
                .unwrap_or(&zero);
            process::reference_transaction(
                command,
                &remaining(&options, started)?,
                &plan.tracking_ref,
                &plan.expected_oid.hex,
                old,
                || {
                    self.revalidate_transport(&plan.transport, &remaining(&options, started)?)?;
                    if self.read_tracking(plan, &remaining(&options, started)?)?
                        != (plan.expected_old.clone(), None)
                    {
                        return Err(stale());
                    }
                    let lock = lock_path(plan)?;
                    let bytes = inspection::file_bytes(&lock, 256)?;
                    if bytes != format!("{}\n", plan.expected_oid.hex).as_bytes() {
                        return Err(stale());
                    }
                    stored.receipt = Some(Receipt {
                        identity: inspection::fs_identity(&lock)?,
                        hash: digest(&bytes),
                    });
                    self.import_checkpoint(&mut stored, FetchImportPhase::ReferencePrepared)
                },
            )?;
            stored.value.reference_status = FetchReferenceStatus::Updated;
            self.import_checkpoint(&mut stored, FetchImportPhase::ReferenceUpdated)?;
            if self.read_tracking(plan, &remaining(&options, started)?)?
                != (Some(plan.expected_oid.clone()), None)
            {
                return Err(fail(
                    ErrorKind::UncertainOutcome,
                    "Destination changed after reference publication",
                ));
            }
            let target = safe_ref_path(plan, false)?;
            store::sync(&File::open(&target)?)?;
            let common = plan.transport.metadata.common_dir.to_path()?;
            for target in [target, safe_ref_path(plan, true)?] {
                let mut parent = target.parent();
                while let Some(dir) = parent {
                    store::sync_dir(dir)?;
                    if dir == common {
                        break;
                    }
                    parent = dir.parent();
                }
            }
            self.import_checkpoint(&mut stored, FetchImportPhase::Complete)
        })();
        if let Err(error) = result {
            stored.value.phase = if error.kind == ErrorKind::UncertainOutcome
                || stored.value.reference_status == FetchReferenceStatus::Updated
            {
                FetchImportPhase::Uncertain
            } else if error.kind == ErrorKind::Cancelled {
                FetchImportPhase::Cancelled
            } else {
                FetchImportPhase::Blocked
            };
            stored.value.issue = Some(error.into());
            self.persist_import(&stored)?;
        }
        Ok(stored.value)
    }
    pub(crate) fn import_audit_detail(&self, raw: &str) -> Result<FetchImport> {
        let stored: StoredImport = serde_json::from_str(raw)?;
        if stored.version != 1 {
            return Err(fail(
                ErrorKind::NewerSchema,
                "Unsupported fetch import record",
            ));
        }
        Ok(stored.value)
    }
    pub(crate) fn import_notices(&mut self) -> Result<Vec<crate::application::FetchImportNotice>> {
        let mut statement=self.workspace.connection().prepare(&format!("SELECT json_extract(value,'$.value.plan.id'), json_extract(value,'$.value.plan.workspace'), json_extract(value,'$.value.plan.worktree') FROM state WHERE {} ORDER BY key LIMIT 100", crate::workspace::recovery::FETCH))?;
        let rows = statement.query_map([], |r| {
            Ok(crate::application::FetchImportNotice {
                operation_id: r.get(0)?,
                workspace_id: r.get(1)?,
                worktree_id: r.get(2)?,
            })
        })?;
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(Error::from)
    }
    pub(crate) fn imports(
        &mut self,
        workspace: &WorkspaceId,
        offset: usize,
    ) -> Result<FetchImportPage> {
        if offset > 1_000_000 {
            return Err(fail(
                ErrorKind::InvalidRequest,
                "Import history offset limit exceeded",
            ));
        }
        let mut statement=self.workspace.connection().prepare("SELECT value FROM state WHERE key LIKE 'fetch-import:%' AND json_extract(value,'$.value.plan.workspace')=?1 ORDER BY json_extract(value,'$.value.plan.created_at_ms') DESC,key LIMIT 51 OFFSET ?2")?;
        let rows = statement.query_map(params![workspace.0, offset as i64], |r| {
            r.get::<_, String>(0)
        })?;
        let mut items = Vec::new();
        let mut size = 0;
        let mut more = false;
        for row in rows {
            let raw = row?;
            if raw.len() > 1024 * 1024 {
                return Err(fail(ErrorKind::OutputLimit, "Import record exceeds 1 MiB"));
            }
            if items.len() == 50 || size + raw.len() > 4 * 1024 * 1024 {
                more = true;
                break;
            }
            size += raw.len();
            let stored: StoredImport = serde_json::from_str(&raw)?;
            if stored.version != 1 || stored.value.plan.transport.workspace != *workspace {
                return Err(stale());
            }
            let mut value = stored.value;
            if value.phase == FetchImportPhase::Proposed && stored.session != self.session {
                value.phase = FetchImportPhase::Interrupted;
            }
            items.push(value);
        }
        let next = more.then_some(offset + items.len());
        Ok(FetchImportPage { items, next })
    }
    pub(crate) fn inspect_import_recovery(
        &mut self,
        workspace: &WorkspaceId,
        id: &str,
    ) -> Result<FetchImportRecovery> {
        let stored = self.import(id)?;
        if stored.value.plan.transport.workspace != *workspace
            || (!unresolved(stored.value.phase) && stored.value.phase != FetchImportPhase::Proposed)
        {
            return Err(stale());
        }
        let plan = &stored.value.plan;
        let (actual, symbolic) = self.read_tracking(plan, &self.options.process)?;
        let lock = lock_path(plan)?;
        let lock_state = match fs::symlink_metadata(&lock) {
            Ok(meta) if meta.is_file() && !meta.is_symlink() => Some((
                inspection::fs_identity(&lock)?,
                digest(&inspection::file_bytes(&lock, 256)?),
            )),
            Ok(_) => return Err(unsupported("Unsafe reference lock")),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
            Err(e) => return Err(e.into()),
        };
        let owned = lock_state.as_ref().is_none_or(|(identity, hash)| {
            stored
                .receipt
                .as_ref()
                .is_some_and(|r| &r.identity == identity && &r.hash == hash)
        });
        let action = if symbolic.is_some() || !owned {
            "manual"
        } else if actual.as_ref() == Some(&plan.expected_oid) {
            "observe_target"
        } else if actual == plan.expected_old {
            "abort_unpublished"
        } else {
            "manual"
        };
        let explanation=match action {"observe_target"=>"Destination currently matches the approved captured commit. Record this observation without attributing an uncertain update or replaying Git.","abort_unpublished"=>"Destination remains at the approved old value. End without replay; imported unreferenced objects remain. Only a receipt-proven owned lock may be removed.",_=>"Ref/symbolic/lock state is unexpected. Manual investigation required; no Git mutation or lock removal is permitted."};
        let fingerprint = digest(&serde_json::to_vec(&(
            stored.value.clone(),
            stored.receipt,
            actual.clone(),
            symbolic,
            lock_state,
        ))?);
        Ok(FetchImportRecovery {
            operation: stored.value,
            action: action.into(),
            actual_oid: actual,
            fingerprint,
            explanation: explanation.into(),
        })
    }
    pub(crate) fn recover_import(&mut self, review: &FetchImportRecovery) -> Result<FetchImport> {
        let plan = &review.operation.plan;
        let _common = crate::locking::ExclusiveLock::acquire(
            File::open(plan.transport.metadata.common_dir.to_path()?)?,
            "Shared repository is write-locked",
        )?;
        let actual = self.inspect_import_recovery(&plan.transport.workspace, &plan.transport.id)?;
        if actual.fingerprint != review.fingerprint {
            return Err(stale());
        }
        if actual.action == "manual" {
            return Err(unsupported("Manual fetch import investigation is required"));
        }
        let mut stored = self.import(&plan.transport.id)?;
        let lock = lock_path(plan)?;
        if fs::symlink_metadata(&lock).is_ok() {
            let receipt = stored.receipt.as_ref().ok_or_else(stale)?;
            if inspection::fs_identity(&lock)? != receipt.identity
                || digest(&inspection::file_bytes(&lock, 256)?) != receipt.hash
            {
                return Err(stale());
            }
            fs::remove_file(&lock)?;
            store::sync_dir(lock.parent().ok_or_else(stale)?)?;
        }
        stored.value.phase = if actual.action == "observe_target" {
            FetchImportPhase::ObservedTarget
        } else {
            FetchImportPhase::Aborted
        };
        stored.value.reference_status = if actual.action == "observe_target" {
            FetchReferenceStatus::MatchesTarget
        } else {
            FetchReferenceStatus::Unchanged
        };
        stored.value.issue = None;
        self.persist_import(&stored)?;
        Ok(stored.value)
    }
}
