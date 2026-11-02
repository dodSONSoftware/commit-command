//! Missing-only upstream configuration: one owned config.lock and one atomic rename.
use super::*;
use crate::remote::{observation::ObservationPlan, upstream::*};
use std::io::Read;
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
const CONFIG_LIMIT: usize = 1024 * 1024;
#[derive(Serialize, Deserialize)]
struct Stored {
    version: u32,
    session: String,
    value: UpstreamOperation,
    receipt: Option<Receipt>,
}
fn stale() -> Error {
    fail(
        ErrorKind::StaleReview,
        "Upstream configuration review changed",
    )
}
fn path(plan: &UpstreamPlan) -> Result<PathBuf> {
    Ok(plan.transport.metadata.common_dir.to_path()?.join("config"))
}
fn lock_path(plan: &UpstreamPlan) -> Result<PathBuf> {
    Ok(plan
        .transport
        .metadata
        .common_dir
        .to_path()?
        .join("config.lock"))
}
#[cfg(target_os = "macos")]
fn plain_metadata(file: &File) -> Result<String> {
    use std::os::fd::AsRawFd;
    use std::os::macos::fs::MetadataExt;
    unsafe extern "C" {
        fn acl_get_fd(fd: libc::c_int) -> *mut libc::c_void;
        fn acl_free(value: *mut libc::c_void) -> libc::c_int;
    }
    // Read-only Darwin checks reject flags/ACLs that this path cannot preserve.
    if file.metadata()?.st_flags() != 0 {
        return Err(unsupported("Configuration file flags are unsupported"));
    }
    let attributes = attributes(file)?;
    let acl = unsafe { acl_get_fd(file.as_raw_fd()) };
    if !acl.is_null() {
        unsafe {
            acl_free(acl);
        };
        return Err(unsupported("Configuration ACLs are unsupported"));
    }
    let error = std::io::Error::last_os_error();
    if error.raw_os_error() != Some(libc::ENOENT) {
        return Err(error.into());
    }
    Ok(attributes)
}
#[cfg(not(target_os = "macos"))]
fn plain_metadata(_file: &File) -> Result<String> {
    Err(unsupported("Upstream writes require macOS/APFS"))
}
#[cfg(target_os = "macos")]
fn attributes(file: &File) -> Result<String> {
    use std::os::fd::AsRawFd;
    let count = unsafe { libc::flistxattr(file.as_raw_fd(), std::ptr::null_mut(), 0, 0) };
    if count < 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    if count > 65536 {
        return Err(unsupported("Configuration attributes exceed 64 KiB"));
    }
    let mut names = vec![0u8; count as usize];
    let actual =
        unsafe { libc::flistxattr(file.as_raw_fd(), names.as_mut_ptr().cast(), names.len(), 0) };
    if actual != count {
        return Err(stale());
    }
    let mut names = names
        .split(|b| *b == 0)
        .filter(|v| !v.is_empty())
        .map(|v| v.to_vec())
        .collect::<Vec<_>>();
    names.sort();
    if names.len() > 256 {
        return Err(unsupported("Too many configuration attributes"));
    }
    let mut size = count as usize;
    let mut records = Vec::new();
    for name in names {
        let key = std::ffi::CString::new(name.clone()).map_err(|_| stale())?;
        let len = unsafe {
            libc::fgetxattr(
                file.as_raw_fd(),
                key.as_ptr(),
                std::ptr::null_mut(),
                0,
                0,
                0,
            )
        };
        if len < 0 {
            return Err(std::io::Error::last_os_error().into());
        }
        size = size.checked_add(len as usize).ok_or_else(stale)?;
        if size > 65536 {
            return Err(unsupported("Configuration attributes exceed 64 KiB"));
        }
        let mut value = vec![0u8; len as usize];
        let actual = unsafe {
            libc::fgetxattr(
                file.as_raw_fd(),
                key.as_ptr(),
                value.as_mut_ptr().cast(),
                value.len(),
                0,
                0,
            )
        };
        if actual != len {
            return Err(stale());
        }
        records.push((name, value));
    }
    Ok(digest(&serde_json::to_vec(&records)?))
}
#[cfg(target_os = "macos")]
fn copy_attributes(source: &Path, target: &File, expected: &str) -> Result<()> {
    use std::os::fd::AsRawFd;
    let source = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(source)?;
    if plain_metadata(&source)? != expected {
        return Err(stale());
    }
    // Descriptor-only OS metadata copying preserves even protected provenance attrs.
    if unsafe {
        libc::fcopyfile(
            source.as_raw_fd(),
            target.as_raw_fd(),
            std::ptr::null_mut(),
            libc::COPYFILE_XATTR,
        )
    } != 0
    {
        return Err(std::io::Error::last_os_error().into());
    }
    if plain_metadata(target)? != expected {
        return Err(unsupported(
            "Configuration attributes cannot be preserved exactly",
        ));
    }
    Ok(())
}
#[cfg(not(target_os = "macos"))]
fn copy_attributes(_source: &Path, _target: &File, _expected: &str) -> Result<()> {
    Err(unsupported("Upstream writes require macOS/APFS"))
}
fn snapshot(path: &Path) -> Result<(ConfigBinding, Vec<u8>)> {
    let mut file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)?;
    let attributes_digest = plain_metadata(&file)?;
    let meta = file.metadata()?;
    // Replacement preserves the current user's ownership and the original group/mode.
    if !meta.is_file()
        || meta.nlink() != 1
        || meta.uid() != unsafe { libc::geteuid() }
        || meta.permissions().mode() & 0o600 != 0o600
        || meta.permissions().mode() & 0o7000 != 0
        || meta.len() > CONFIG_LIMIT as u64
    {
        return Err(unsupported(
            "Configuration requires a bounded regular user-owned file with owner read/write access",
        ));
    }
    let mut bytes = Vec::new();
    Read::by_ref(&mut file)
        .take(CONFIG_LIMIT as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > CONFIG_LIMIT {
        return Err(fail(ErrorKind::OutputLimit, "Configuration exceeds 1 MiB"));
    }
    Ok((
        ConfigBinding {
            identity: inspection::fs_identity(path)?,
            digest: digest(&bytes),
            mode: meta.permissions().mode() & 0o777,
            group: meta.gid(),
            attributes_digest,
        },
        bytes,
    ))
}
fn quote(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}
fn candidate(plan: &UpstreamPlan, old: &[u8]) -> Result<Vec<u8>> {
    let branch = plan
        .source_ref
        .strip_prefix("refs/heads/")
        .ok_or_else(stale)?;
    let suffix = format!(
        "\n[branch {}]\n\tremote = {}\n\tmerge = {}\n",
        quote(branch),
        quote(&plan.transport.remote),
        quote(&plan.transport.reference)
    );
    if old.len() + suffix.len() > CONFIG_LIMIT {
        return Err(fail(
            ErrorKind::OutputLimit,
            "Updated configuration exceeds 1 MiB",
        ));
    }
    Ok([old, suffix.as_bytes()].concat())
}
fn records(bytes: &[u8]) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
    let Some(bytes) = bytes.strip_suffix(&[0]) else {
        if bytes.is_empty() {
            return Ok(Vec::new());
        }
        return Err(unsupported("Invalid configuration records"));
    };
    bytes
        .split(|b| *b == 0)
        .map(|line| {
            let (key, value) = line
                .iter()
                .position(|b| *b == b'\n')
                .map(|at| (&line[..at], &line[at + 1..]))
                .unwrap_or((line, &[]));
            if key.is_empty() {
                return Err(unsupported("Invalid configuration key"));
            }
            Ok((key.to_vec(), value.to_vec()))
        })
        .collect()
}
fn parse(
    plan: &ObservationPlan,
    bytes: Vec<u8>,
    options: &ReadOptions,
) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
    let mut command = remote::command(
        plan,
        &plan.metadata.root.to_path()?,
        &plan.metadata.common_dir.to_path()?,
    )?;
    command.env("GIT_ALLOW_PROTOCOL", "").args([
        "config",
        "--file",
        "-",
        "--no-includes",
        "--null",
        "--list",
    ]);
    let result = process::bounded_input(command, options, Some(bytes))?;
    if !result.status.success() {
        return Err(unsupported("Configuration cannot be parsed safely"));
    }
    records(&result.stdout)
}
fn check_identity(plan: &UpstreamPlan, options: &ReadOptions) -> Result<()> {
    let current = inspection::repository_metadata_with_options(
        &plan.transport.metadata.root.to_path()?,
        options,
    )?;
    let old = &plan.transport.metadata;
    if current.root_binding != old.root_binding
        || current.git_binding != old.git_binding
        || current.common_binding != old.common_binding
    {
        return Err(stale());
    }
    policy::safe_metadata_paths(old)
}
fn owned(stored: &Stored) -> Result<bool> {
    let lock = lock_path(&stored.value.plan)?;
    match fs::symlink_metadata(&lock) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(e.into()),
        Ok(m) if !m.is_file() || m.is_symlink() || m.nlink() != 1 => Ok(false),
        Ok(_) => {
            let Some(receipt) = stored.receipt.as_ref() else {
                return Ok(false);
            };
            Ok(inspection::fs_identity(&lock)? == receipt.identity
                && digest(&inspection::file_bytes(&lock, CONFIG_LIMIT)?) == receipt.hash)
        }
    }
}
fn pending(phase: UpstreamPhase) -> bool {
    matches!(
        phase,
        UpstreamPhase::LockPending
            | UpstreamPhase::LockCreated
            | UpstreamPhase::LockReady
            | UpstreamPhase::PublicationPending
            | UpstreamPhase::ConfigPublished
            | UpstreamPhase::Uncertain
    )
}
impl ControlledWrites {
    fn store_upstream(&mut self, stored: &Stored) -> Result<()> {
        let raw = serde_json::to_string(stored)?;
        if raw.len() > 1024 * 1024 {
            return Err(fail(
                ErrorKind::OutputLimit,
                "Upstream record exceeds 1 MiB",
            ));
        }
        self.workspace.connection().execute(
            "INSERT OR REPLACE INTO state(key,value) VALUES(?1,?2)",
            params![format!("upstream:{}", stored.value.plan.transport.id), raw],
        )?;
        Ok(())
    }
    fn load_upstream(&mut self, id: &str) -> Result<Stored> {
        Uuid::parse_str(id).map_err(|_| stale())?;
        let raw: Option<String> = self
            .workspace
            .connection()
            .query_row(
                "SELECT value FROM state WHERE key=?1",
                [format!("upstream:{id}")],
                |r| r.get(0),
            )
            .optional()?;
        let raw = raw.ok_or_else(stale)?;
        if raw.len() > 1024 * 1024 {
            return Err(fail(
                ErrorKind::OutputLimit,
                "Upstream record exceeds 1 MiB",
            ));
        }
        let stored: Stored = serde_json::from_str(&raw)?;
        if stored.version != 1 {
            return Err(fail(ErrorKind::NewerSchema, "Unsupported upstream record"));
        }
        if stored.value.plan.transport.id != id {
            return Err(stale());
        }
        Ok(stored)
    }
    fn upstream_local_pending(&mut self, common: &str) -> Result<()> {
        let yes:bool=self.workspace.connection().query_row("SELECT EXISTS(SELECT 1 FROM write_operations WHERE json_extract(payload,'$.metadata.common_binding')=?1 AND json_extract(payload,'$.phase') NOT IN ('complete','blocked','published_with_post_processing_failure','cancelled_before_publication','proposed','approved','needs_reauthorization'))",[common],|r|r.get(0))?;
        if yes {
            return Err(fail(
                ErrorKind::Busy,
                "Resolve interrupted local writes before setting an upstream",
            ));
        }
        Ok(())
    }
    fn validate_upstream_plan(
        &mut self,
        plan: &UpstreamPlan,
        options: &ReadOptions,
    ) -> Result<Vec<u8>> {
        self.revalidate_transport(&plan.transport, options)?;
        let (binding, bytes) = snapshot(&path(plan)?)?;
        if binding != plan.expected_config {
            return Err(stale());
        }
        let updated = candidate(plan, &bytes)?;
        if digest(&updated) != plan.candidate_digest {
            return Err(stale());
        }
        Ok(updated)
    }
    pub(crate) fn propose_upstream(
        &mut self,
        workspace: &WorkspaceId,
        entry: &crate::workspace::StoredWorktree,
        observation: &str,
        options: &ReadOptions,
    ) -> Result<UpstreamPlan> {
        let previous = std::mem::replace(&mut self.options.process, options.clone());
        let result = self.propose_upstream_inner(workspace, entry, observation, options);
        self.options.process = previous;
        result
    }
    fn propose_upstream_inner(
        &mut self,
        workspace: &WorkspaceId,
        entry: &crate::workspace::StoredWorktree,
        observation: &str,
        options: &ReadOptions,
    ) -> Result<UpstreamPlan> {
        self.ensure_fetch_import_resolved(&entry.metadata.common_binding, None)?;
        self.ensure_upstream_resolved(&entry.metadata.common_binding, None)?;
        self.upstream_local_pending(&entry.metadata.common_binding)?;
        let observed = self.completed_observation(observation, workspace, &entry.id)?;
        let transport = self.prepare_transport_plan(
            workspace,
            entry,
            &observed.plan.remote,
            &observed.plan.reference,
            options,
        )?;
        if transport.config_digest != observed.plan.config_digest
            || transport.destination != observed.plan.destination
            || transport.metadata.common_binding != observed.plan.metadata.common_binding
            || transport.metadata.root_binding != observed.plan.metadata.root_binding
            || transport.metadata.git_binding != observed.plan.metadata.git_binding
        {
            return Err(stale());
        }
        let (source_ref, source_oid) =
            inspection::read(&entry.metadata.root.to_path()?, options, |s| {
                if s.generation != transport.generation {
                    return Err(stale());
                }
                let reference = inspection::output_text(s.git(&["symbolic-ref", "-q", "HEAD"])?)?;
                Ok((
                    reference,
                    s.head()?
                        .ok_or_else(|| unsupported("Upstream setup requires a local commit"))?,
                ))
            })?;
        let branch = source_ref
            .strip_prefix("refs/heads/")
            .ok_or_else(stale)?
            .to_owned();
        if !branch.is_ascii() || !transport.remote.is_ascii() || !transport.reference.is_ascii() {
            return Err(unsupported(
                "Initial upstream setup requires ASCII branch and remote names",
            ));
        }
        // Effective values are transient here; never add remote/config secrets to
        // the write engine's deliberately restricted persisted policy map.
        let mut command = adapter::clean(Path::new("git"));
        command.current_dir(entry.metadata.root.to_path()?).args([
            "--no-pager",
            "config",
            "--null",
            "--list",
            "--includes",
        ]);
        if let Some(global) = self.options.global_config.as_deref() {
            command
                .env("GIT_CONFIG_GLOBAL", global)
                .env("GIT_CONFIG_NOSYSTEM", "1");
        }
        let output = process::bounded_input(command, options, None)?;
        if !output.status.success() {
            return Err(unsupported("Cannot inspect effective upstream policy"));
        }
        let values = records(&output.stdout)?;
        let prefix = format!("branch.{}.", branch.to_ascii_lowercase());
        if values.iter().any(|(k, _)| {
            k.to_ascii_lowercase().starts_with(prefix.as_bytes())
                || k.eq_ignore_ascii_case(b"extensions.worktreeconfig")
        }) {
            return Err(unsupported("Initial upstream setup requires missing branch configuration and no worktree config"));
        }
        let key = format!("remote.{}.fetch", transport.remote.to_ascii_lowercase());
        let mappings = values
            .iter()
            .filter(|(k, _)| k.eq_ignore_ascii_case(key.as_bytes()))
            .collect::<Vec<_>>();
        let expected = format!("+refs/heads/*:refs/remotes/{}/*", transport.remote);
        if mappings.len() != 1 || mappings[0].1 != expected.as_bytes() {
            return Err(unsupported(
                "Initial upstream setup requires the standard named-remote fetch mapping",
            ));
        }
        let report = crate::remote::inspect(
            &entry.metadata.root.to_path()?,
            self.options.global_config.as_deref(),
            options,
        )?;
        let remote = report
            .remotes
            .iter()
            .find(|r| r.name == transport.remote)
            .ok_or_else(stale)?;
        if report.config_digest != transport.config_digest
            || report.generation != transport.generation
            || remote.fetch_refspec_count != 1
        {
            return Err(stale());
        }
        let (binding, bytes) = snapshot(&entry.metadata.common_dir.to_path()?.join("config"))?;
        let mut plan = UpstreamPlan {
            transport,
            source_ref,
            source_oid,
            observation_id: observation.into(),
            observed_oid: observed.oid.ok_or_else(stale)?,
            observed_at_ms: observed.observed_at_ms.ok_or_else(stale)?,
            expected_config: binding,
            candidate_digest: String::new(),
        };
        let before = parse(&plan.transport, bytes.clone(), options)?;
        if before
            .iter()
            .any(|(key, _)| key.starts_with(b"include.") || key.starts_with(b"includeif."))
        {
            return Err(unsupported(
                "Local configuration includes require a broader upstream policy",
            ));
        }
        let updated = candidate(&plan, &bytes)?;
        let mut expected = before;
        expected.push((
            format!("branch.{branch}.remote").into_bytes(),
            plan.transport.remote.as_bytes().to_vec(),
        ));
        expected.push((
            format!("branch.{branch}.merge").into_bytes(),
            plan.transport.reference.as_bytes().to_vec(),
        ));
        if parse(&plan.transport, updated.clone(), options)? != expected {
            return Err(unsupported(
                "Candidate configuration changed unrelated semantics",
            ));
        }
        plan.candidate_digest = digest(&updated);
        self.validate_upstream_plan(&plan, options)?;
        self.store_upstream(&Stored {
            version: 1,
            session: self.session.clone(),
            receipt: None,
            value: UpstreamOperation {
                plan: plan.clone(),
                phase: UpstreamPhase::Proposed,
                configuration: UpstreamStatus::Unchanged,
                issue: None,
            },
        })?;
        Ok(plan)
    }
    pub(crate) fn cancel_upstream(&mut self, id: &str) -> Result<()> {
        let mut stored = self.load_upstream(id)?;
        if stored.value.phase == UpstreamPhase::Proposed {
            stored.value.phase = UpstreamPhase::Cancelled;
            self.store_upstream(&stored)?;
        }
        Ok(())
    }
    fn upstream_checkpoint(&mut self, stored: &mut Stored, phase: UpstreamPhase) -> Result<()> {
        stored.value.phase = phase;
        self.store_upstream(stored)?;
        #[cfg(feature = "write-test-support")]
        if self.upstream_fault.as_ref().is_some_and(|f| f.0 == phase) {
            let root = stored.value.plan.transport.metadata.root.to_path()?;
            if !root.starts_with(fs::canonicalize(std::env::temp_dir())?)
                || !root.join(".gitfleet-m2-fixture").is_file()
            {
                return Err(fail(
                    ErrorKind::InvalidRequest,
                    "Fault injection requires a disposable fixture",
                ));
            }
            match self.upstream_fault.take().expect("checked fault").1 {
                TestFaultKind::Sigkill => {
                    unsafe {
                        libc::kill(libc::getpid(), libc::SIGKILL);
                    };
                    loop {
                        std::thread::park();
                    }
                }
                _ => {
                    return Err(fail(
                        ErrorKind::Storage,
                        "Injected upstream storage failure",
                    ))
                }
            }
        }
        Ok(())
    }
    #[cfg(feature = "write-test-support")]
    pub(crate) fn set_upstream_fault(&mut self, phase: UpstreamPhase, kind: TestFaultKind) {
        self.upstream_fault = Some((phase, kind));
    }
    pub(crate) fn execute_upstream(
        &mut self,
        plan: &UpstreamPlan,
        cancellation: inspection::Cancellation,
    ) -> Result<UpstreamOperation> {
        let previous = std::mem::replace(&mut self.options.process.cancellation, cancellation);
        let result = self.execute_upstream_inner(plan);
        self.options.process.cancellation = previous;
        result
    }
    fn execute_upstream_inner(&mut self, plan: &UpstreamPlan) -> Result<UpstreamOperation> {
        let mut stored = self.load_upstream(&plan.transport.id)?;
        if stored.session != self.session
            || stored.value.phase != UpstreamPhase::Proposed
            || serde_json::to_vec(&stored.value.plan)? != serde_json::to_vec(plan)?
        {
            return Err(stale());
        }
        let common = plan.transport.metadata.common_dir.to_path()?;
        let _common = crate::locking::ExclusiveLock::acquire(
            File::open(&common)?,
            "Shared repository is write-locked",
        )?;
        self.ensure_fetch_import_resolved(&plan.transport.metadata.common_binding, None)?;
        self.ensure_upstream_resolved(
            &plan.transport.metadata.common_binding,
            Some(&plan.transport.id),
        )?;
        self.upstream_local_pending(&plan.transport.metadata.common_binding)?;
        let options = self.options.process.clone();
        let start = Instant::now();
        let lock = lock_path(plan)?;
        let mut lock_created = false;
        let result = (|| {
            options.check(start)?;
            let candidate = self.validate_upstream_plan(plan, &options)?;
            if fs::symlink_metadata(common.join("config.lock.lock")).is_ok() {
                return Err(fail(
                    ErrorKind::Busy,
                    "Foreign secondary configuration lock",
                ));
            }
            self.upstream_checkpoint(&mut stored, UpstreamPhase::LockPending)?;
            let mut file = OpenOptions::new()
                .read(true)
                .write(true)
                .create_new(true)
                .mode(0o600)
                .custom_flags(libc::O_NOFOLLOW)
                .open(&lock)?;
            lock_created = true;
            stored.receipt = Some(Receipt {
                identity: inspection::fs_identity(&lock)?,
                hash: digest(b""),
            });
            self.upstream_checkpoint(&mut stored, UpstreamPhase::LockCreated)?;
            plain_metadata(&file)?;
            copy_attributes(&path(plan)?, &file, &plan.expected_config.attributes_digest)?;
            if file.metadata()?.gid() != plan.expected_config.group {
                return Err(unsupported("Configuration group cannot be preserved"));
            }
            file.write_all(&candidate)?;
            store::sync(&file)?;
            stored.receipt.as_mut().ok_or_else(stale)?.hash = plan.candidate_digest.clone();
            file.set_permissions(fs::Permissions::from_mode(plan.expected_config.mode))?;
            store::sync(&file)?;
            store::sync_dir(&common)?;
            self.upstream_checkpoint(&mut stored, UpstreamPhase::LockReady)?;
            self.validate_upstream_plan(plan, &options)?;
            if !owned(&stored)? {
                return Err(stale());
            }
            if plain_metadata(&file)? != plan.expected_config.attributes_digest {
                return Err(stale());
            }
            self.upstream_checkpoint(&mut stored, UpstreamPhase::PublicationPending)?;
            options.check(start)?;
            fs::rename(&lock, path(plan)?)?;
            stored.value.configuration = UpstreamStatus::Updated;
            store::sync(&file)?;
            store::sync_dir(&common)?;
            self.upstream_checkpoint(&mut stored, UpstreamPhase::ConfigPublished)?;
            let binding = snapshot(&path(plan)?)?.0;
            if binding.digest != plan.candidate_digest
                || binding.mode != plan.expected_config.mode
                || binding.group != plan.expected_config.group
                || binding.attributes_digest != plan.expected_config.attributes_digest
                || stored
                    .receipt
                    .as_ref()
                    .is_none_or(|r| r.identity != binding.identity)
            {
                return Err(fail(
                    ErrorKind::UncertainOutcome,
                    "Configuration or metadata changed after publication",
                ));
            }
            let report = crate::remote::inspect(
                &plan.transport.metadata.root.to_path()?,
                self.options.global_config.as_deref(),
                &options,
            )?;
            if report.branch.as_deref() != Some(&plan.source_ref)
                || report.upstream_remote.as_deref() != Some(&plan.transport.remote)
                || report.upstream_ref.as_deref() != Some(&plan.transport.reference)
            {
                return Err(fail(
                    ErrorKind::UncertainOutcome,
                    "Published upstream could not be verified",
                ));
            }
            let root = plan.transport.metadata.root.to_path()?;
            let head = inspection::read(&root, &options, |s| s.head())?;
            if head.as_ref() != Some(&plan.source_oid) {
                return Err(fail(
                    ErrorKind::UncertainOutcome,
                    "Source branch changed after configuration publication",
                ));
            }
            self.upstream_checkpoint(&mut stored, UpstreamPhase::Complete)
        })();
        if let Err(error) = result {
            let mut uncertain = stored.value.configuration == UpstreamStatus::Updated
                || stored.value.phase == UpstreamPhase::PublicationPending
                || error.kind == ErrorKind::UncertainOutcome;
            if !uncertain && lock_created && fs::symlink_metadata(&lock).is_ok() {
                if owned(&stored)? {
                    fs::remove_file(&lock)?;
                    store::sync_dir(&common)?;
                } else {
                    uncertain = true;
                }
            }
            stored.value.phase = if uncertain {
                UpstreamPhase::Uncertain
            } else if error.kind == ErrorKind::Cancelled {
                UpstreamPhase::Cancelled
            } else {
                UpstreamPhase::Blocked
            };
            stored.value.issue = Some(error.into());
            self.store_upstream(&stored)?;
        }
        Ok(stored.value)
    }
    pub(crate) fn upstream_audit_detail(&self, raw: &str) -> Result<UpstreamOperation> {
        let stored: Stored = serde_json::from_str(raw)?;
        if stored.version != 1 {
            return Err(fail(ErrorKind::NewerSchema, "Unsupported upstream record"));
        }
        Ok(stored.value)
    }
    pub(crate) fn upstream_notices(
        &mut self,
    ) -> Result<Vec<crate::application::RemoteRecoveryNotice>> {
        let mut statement=self.workspace.connection().prepare("SELECT json_extract(value,'$.value.plan.id'),json_extract(value,'$.value.plan.workspace'),json_extract(value,'$.value.plan.worktree') FROM state WHERE key LIKE 'upstream:%' AND json_extract(value,'$.value.phase') IN ('lock_pending','lock_created','lock_ready','publication_pending','config_published','uncertain') ORDER BY key LIMIT 100")?;
        let rows = statement.query_map([], |r| {
            Ok(crate::application::RemoteRecoveryNotice {
                operation_id: r.get(0)?,
                workspace_id: r.get(1)?,
                worktree_id: r.get(2)?,
            })
        })?;
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(Error::from)
    }
    pub(crate) fn inspect_upstream_recovery(
        &mut self,
        workspace: &WorkspaceId,
        id: &str,
    ) -> Result<UpstreamRecovery> {
        let stored = self.load_upstream(id)?;
        let plan = &stored.value.plan;
        if &plan.transport.workspace != workspace
            || (!pending(stored.value.phase) && stored.value.phase != UpstreamPhase::Proposed)
        {
            return Err(stale());
        }
        check_identity(plan, &self.options.process)?;
        let (actual, _) = snapshot(&path(plan)?)?;
        let lock = lock_path(plan)?;
        let lock_state = match fs::symlink_metadata(&lock) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
            Err(e) => return Err(e.into()),
            Ok(m) if !m.is_file() || m.is_symlink() || m.nlink() != 1 => {
                return Err(unsupported("Unsafe configuration lock"))
            }
            Ok(_) => Some((
                inspection::fs_identity(&lock)?,
                digest(&inspection::file_bytes(&lock, CONFIG_LIMIT)?),
            )),
        };
        let lock_owned = lock_state.as_ref().is_none_or(|(id, hash)| {
            stored
                .receipt
                .as_ref()
                .is_some_and(|r| r.identity == *id && r.hash == *hash)
        });
        let action = if !lock_owned {
            "manual"
        } else if actual.digest == plan.candidate_digest
            && actual.mode == plan.expected_config.mode
            && actual.group == plan.expected_config.group
            && actual.attributes_digest == plan.expected_config.attributes_digest
        {
            "observe_configuration"
        } else if actual == plan.expected_config {
            "abort_unpublished"
        } else {
            "manual"
        };
        let explanation=match action {"observe_configuration"=>"Configuration currently matches the approved candidate. Record the observation without attributing uncertain publication, replaying configuration writes or renewing trust.","abort_unpublished"=>"The original configuration is unchanged. End without replay and remove only a receipt-proven owned config.lock.",_=>"Configuration or lock state is unexpected. Manual investigation required; no writes or lock removal are permitted."};
        let fingerprint = digest(&serde_json::to_vec(&(
            stored.value.clone(),
            stored.receipt,
            actual,
            lock_state,
        ))?);
        Ok(UpstreamRecovery {
            operation: stored.value,
            action: action.into(),
            fingerprint,
            explanation: explanation.into(),
        })
    }
    pub(crate) fn recover_upstream(
        &mut self,
        review: &UpstreamRecovery,
    ) -> Result<UpstreamOperation> {
        let plan = &review.operation.plan;
        let common = plan.transport.metadata.common_dir.to_path()?;
        let _common = crate::locking::ExclusiveLock::acquire(
            File::open(&common)?,
            "Shared repository is write-locked",
        )?;
        let current =
            self.inspect_upstream_recovery(&plan.transport.workspace, &plan.transport.id)?;
        if current.fingerprint != review.fingerprint {
            return Err(stale());
        }
        if current.action == "manual" {
            return Err(unsupported("Manual upstream investigation is required"));
        }
        let mut stored = self.load_upstream(&plan.transport.id)?;
        let lock = lock_path(plan)?;
        if fs::symlink_metadata(&lock).is_ok() {
            if !owned(&stored)? {
                return Err(stale());
            }
            fs::remove_file(lock)?;
            store::sync_dir(&common)?;
        }
        stored.value.phase = if current.action == "observe_configuration" {
            UpstreamPhase::ObservedConfiguration
        } else {
            UpstreamPhase::Aborted
        };
        stored.value.configuration = if current.action == "observe_configuration" {
            UpstreamStatus::MatchesTarget
        } else {
            UpstreamStatus::Unchanged
        };
        stored.value.issue = None;
        self.store_upstream(&stored)?;
        Ok(stored.value)
    }
}
