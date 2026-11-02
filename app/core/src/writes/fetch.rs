//! Separately authorized, bounded downloads never touch a managed object store or ref.
use super::*;
use crate::remote::fetch::*;

pub(super) const OBJECT_BYTES: u64 = 64 * 1024 * 1024;
const ARTIFACT_BYTES: u64 = 128 * 1024 * 1024;
const OBJECT_COUNT: usize = 10_000;
pub(super) const CAPTURE_REF: &str = "refs/heads/gitfleet-fetch";

#[derive(Serialize, Deserialize)]
struct StoredCapture {
    version: u32,
    session: String,
    value: FetchCapture,
}
fn stale() -> Error {
    fail(ErrorKind::StaleReview, "Fetch download review is stale")
}
fn artifact_budget(root: &Path) -> Result<()> {
    check_artifact_budget(root, ARTIFACT_BYTES, 1024)
}
pub(super) fn check_artifact_budget(root: &Path, max_bytes: u64, max_entries: usize) -> Result<()> {
    let mut directories = vec![root.to_path_buf()];
    let mut count = 0;
    let mut size = 0u64;
    while let Some(directory) = directories.pop() {
        if !fs::symlink_metadata(&directory)?.file_type().is_dir() {
            return Err(unsupported("Unsafe fetch artifact directory"));
        }
        for entry in fs::read_dir(directory)? {
            let entry = entry?;
            let metadata = match fs::symlink_metadata(entry.path()) {
                Ok(metadata) => metadata,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
                Err(error) => return Err(error.into()),
            };
            count += 1;
            if metadata.is_dir() {
                directories.push(entry.path());
            } else if metadata.is_file() {
                size = size.saturating_add(metadata.len());
            } else {
                return Err(unsupported("Unsafe fetch artifact type"));
            }
            if count > max_entries || size > max_bytes {
                return Err(fail(
                    ErrorKind::OutputLimit,
                    "Fetch artifact budget exceeded",
                ));
            }
        }
    }
    Ok(())
}
pub(super) fn object_inventory(list: &[u8], format: &ObjectFormat) -> Result<(u32, u64)> {
    let list = std::str::from_utf8(list)
        .map_err(|_| fail(ErrorKind::InvalidData, "Invalid captured object inventory"))?;
    let mut count = 0;
    let mut bytes = 0u64;
    for line in list.lines() {
        let (oid, size) = line
            .split_once(' ')
            .ok_or_else(|| fail(ErrorKind::InvalidData, "Invalid captured object inventory"))?;
        crate::domain::ObjectId::parse(oid, format)?;
        bytes = bytes.saturating_add(
            size.parse::<u64>()
                .map_err(|_| fail(ErrorKind::InvalidData, "Invalid object size"))?,
        );
        count += 1;
        if count > OBJECT_COUNT || bytes > OBJECT_BYTES {
            return Err(fail(
                ErrorKind::OutputLimit,
                "Fetch exceeds 10,000 objects or 64 MiB uncompressed",
            ));
        }
    }
    Ok((count as u32, bytes))
}
impl ControlledWrites {
    fn persist_capture(&mut self, stored: &StoredCapture) -> Result<()> {
        let raw = serde_json::to_string(stored)?;
        if raw.len() > 1024 * 1024 {
            return Err(fail(ErrorKind::OutputLimit, "Fetch record exceeds 1 MiB"));
        }
        self.workspace.connection().execute(
            "INSERT OR REPLACE INTO state(key,value) VALUES(?1,?2)",
            params![
                format!("fetch-capture:{}", stored.value.plan.transport.id),
                raw
            ],
        )?;
        Ok(())
    }
    fn capture(&mut self, id: &str) -> Result<StoredCapture> {
        Uuid::parse_str(id).map_err(|_| stale())?;
        let raw: Option<String> = self
            .workspace
            .connection()
            .query_row(
                "SELECT value FROM state WHERE key=?1",
                [format!("fetch-capture:{id}")],
                |r| r.get(0),
            )
            .optional()?;
        let raw = raw.ok_or_else(stale)?;
        if raw.len() > 1024 * 1024 {
            return Err(fail(ErrorKind::OutputLimit, "Fetch record exceeds 1 MiB"));
        }
        let stored: StoredCapture = serde_json::from_str(&raw)?;
        if stored.version != 1 {
            return Err(fail(ErrorKind::NewerSchema, "Unsupported fetch record"));
        }
        if stored.value.plan.transport.id != id {
            return Err(stale());
        }
        Ok(stored)
    }
    pub(super) fn completed_capture(
        &mut self,
        id: &str,
        workspace: &WorkspaceId,
        worktree: &WorktreeId,
    ) -> Result<FetchCapture> {
        let stored = self.capture(id)?;
        if stored.value.plan.transport.workspace != *workspace
            || stored.value.plan.transport.worktree != *worktree
            || stored.value.phase != FetchCapturePhase::Complete
            || stored.value.artifact.is_none()
        {
            return Err(stale());
        }
        Ok(stored.value)
    }
    pub(crate) fn propose_capture(
        &mut self,
        workspace: &WorkspaceId,
        entry: &crate::workspace::StoredWorktree,
        observation: &str,
        options: &ReadOptions,
    ) -> Result<FetchCapturePlan> {
        let source = self.completed_observation(observation, workspace, &entry.id)?;
        let previous = std::mem::replace(&mut self.options.process, options.clone());
        let result = self.prepare_transport_plan(
            workspace,
            entry,
            &source.plan.remote,
            &source.plan.reference,
            options,
        );
        self.options.process = previous;
        let transport = result?;
        if transport.config_digest != source.plan.config_digest
            || transport.generation != source.plan.generation
        {
            return Err(stale());
        }
        let plan = FetchCapturePlan {
            transport,
            observation_id: observation.into(),
            expected_oid: source.oid.ok_or_else(stale)?,
        };
        self.persist_capture(&StoredCapture {
            version: 1,
            session: self.session.clone(),
            value: FetchCapture {
                plan: plan.clone(),
                phase: FetchCapturePhase::Proposed,
                artifact: None,
                captured_at_ms: None,
                issue: None,
            },
        })?;
        Ok(plan)
    }
    pub(crate) fn cancel_capture(&mut self, id: &str) -> Result<()> {
        let mut stored = self.capture(id)?;
        if stored.value.phase == FetchCapturePhase::Proposed {
            stored.value.phase = FetchCapturePhase::Cancelled;
            self.persist_capture(&stored)?;
        }
        Ok(())
    }
    pub(crate) fn execute_capture(
        &mut self,
        plan: &FetchCapturePlan,
        cancellation: inspection::Cancellation,
    ) -> Result<FetchCapture> {
        let previous = std::mem::replace(&mut self.options.process.cancellation, cancellation);
        let result = self.execute_capture_inner(plan);
        self.options.process.cancellation = previous;
        result
    }
    fn execute_capture_inner(&mut self, plan: &FetchCapturePlan) -> Result<FetchCapture> {
        let mut stored = self.capture(&plan.transport.id)?;
        if stored.session != self.session
            || stored.value.phase != FetchCapturePhase::Proposed
            || serde_json::to_vec(&stored.value.plan)? != serde_json::to_vec(plan)?
        {
            return Err(stale());
        }
        stored.value.phase = FetchCapturePhase::Running;
        self.persist_capture(&stored)?;
        let started = Instant::now();
        let result = (|| {
            let _common = crate::locking::ExclusiveLock::acquire(
                File::open(plan.transport.metadata.common_dir.to_path()?)?,
                "Shared repository is write-locked",
            )?;
            let mut options = self.options.process.clone();
            options.max_output = 1024 * 1024;
            let url = self.revalidate_transport(&plan.transport, &options)?;
            let (directory, home, repository) =
                self.initialize_transport(&plan.transport, &options)?;
            let run = |args: &[&str]| -> Result<Vec<u8>> {
                self.options.process.check(started)?;
                let mut remaining = options.clone();
                remaining.timeout = remaining.timeout.saturating_sub(started.elapsed());
                let mut command = remote::command(&plan.transport, &repository, &home)?;
                command.args(args);
                let output = process::bounded_input_checked(command, &remaining, None, || {
                    artifact_budget(&directory)
                })?;
                if !output.status.success() {
                    return Err(fail(ErrorKind::InspectionFailed,"Fetch download failed (transport, authentication, certificate, redirect, object validation or server rejection); raw server diagnostics are not retained"));
                }
                Ok(output.stdout)
            };
            let refspec = format!("{}:{CAPTURE_REF}", plan.transport.reference);
            run(&[
                "-c",
                "fetch.unpackLimit=0",
                "-c",
                "fetch.fsckObjects=true",
                "fetch",
                "--no-tags",
                "--no-write-fetch-head",
                "--no-auto-maintenance",
                "--no-write-commit-graph",
                "--no-recurse-submodules",
                "--no-progress",
                "--refmap=",
                "--",
                &url,
                &refspec,
            ])?;
            let tip = inspection::output_text(run(&[
                "rev-parse",
                "--verify",
                &format!("{CAPTURE_REF}^{{commit}}"),
            ])?)?;
            if tip != plan.expected_oid.hex {
                return Err(fail(
                    ErrorKind::ConcurrentChange,
                    "Remote branch moved since observation; observe and review again",
                ));
            }
            run(&[
                "fsck",
                "--strict",
                "--full",
                "--no-reflogs",
                "--no-dangling",
                &tip,
            ])?;
            let list = run(&[
                "cat-file",
                "--batch-all-objects",
                "--batch-check=%(objectname) %(objectsize)",
            ])?;
            let (count, bytes) = object_inventory(&list, &plan.transport.metadata.object_format)?;
            let bundle = directory.join("capture.bundle");
            let bundle_text = bundle
                .to_str()
                .ok_or_else(|| unsupported("Fetch artifact path must be UTF-8"))?;
            run(&["bundle", "create", bundle_text, CAPTURE_REF])?;
            run(&["bundle", "verify", bundle_text])?;
            let payload = inspection::file_bytes(&bundle, OBJECT_BYTES as usize)?;
            store::sync(&File::open(&bundle)?)?;
            store::sync_dir(&directory)?;
            Ok(FetchArtifact {
                bundle_digest: digest(&payload),
                bundle_bytes: payload.len() as u64,
                object_count: count,
                object_bytes: bytes,
            })
        })();
        match result {
            Ok(artifact) => {
                stored.value.phase = FetchCapturePhase::Complete;
                stored.value.artifact = Some(artifact);
                stored.value.captured_at_ms = Some(now_ms());
            }
            Err(error) => {
                stored.value.phase = if error.kind == ErrorKind::Cancelled {
                    FetchCapturePhase::Cancelled
                } else {
                    FetchCapturePhase::Blocked
                };
                stored.value.issue = Some(error.into());
            }
        }
        self.persist_capture(&stored)?;
        Ok(stored.value)
    }
    pub(crate) fn capture_audit_detail(&self, raw: &str) -> Result<FetchCapture> {
        let stored: StoredCapture = serde_json::from_str(raw)?;
        if stored.version != 1 {
            return Err(fail(ErrorKind::NewerSchema, "Unsupported fetch record"));
        }
        Ok(stored.value)
    }
    pub(crate) fn captures(
        &mut self,
        workspace: &WorkspaceId,
        worktree: Option<&WorktreeId>,
        offset: usize,
    ) -> Result<FetchCapturePage> {
        if offset > 1_000_000 {
            return Err(fail(
                ErrorKind::InvalidRequest,
                "Fetch history offset limit exceeded",
            ));
        }
        let mut statement=self.workspace.connection().prepare("SELECT value FROM state WHERE key LIKE 'fetch-capture:%' AND json_extract(value,'$.value.plan.workspace')=?1 AND (?2 IS NULL OR json_extract(value,'$.value.plan.worktree')=?2) ORDER BY json_extract(value,'$.value.plan.created_at_ms') DESC,key LIMIT 51 OFFSET ?3")?;
        let rows = statement.query_map(
            params![workspace.0, worktree.map(|w| &w.0), offset as i64],
            |r| r.get::<_, String>(0),
        )?;
        let mut items = Vec::new();
        let mut size = 0;
        let mut more = false;
        for raw in rows {
            let raw = raw?;
            if raw.len() > 1024 * 1024 {
                return Err(fail(ErrorKind::OutputLimit, "Fetch record exceeds 1 MiB"));
            }
            if items.len() == 50 || size + raw.len() > 4 * 1024 * 1024 {
                more = true;
                break;
            }
            size += raw.len();
            let mut stored: StoredCapture = serde_json::from_str(&raw)?;
            if stored.version != 1 || stored.value.plan.transport.workspace != *workspace {
                return Err(fail(ErrorKind::NewerSchema, "Invalid fetch record"));
            }
            if stored.value.phase == FetchCapturePhase::Running
                || (stored.value.phase == FetchCapturePhase::Proposed
                    && stored.session != self.session)
            {
                stored.value.phase = FetchCapturePhase::Interrupted;
            }
            items.push(stored.value);
        }
        let next = more.then_some(offset + items.len());
        Ok(FetchCapturePage { items, next })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn inventory_limits_are_checked_before_bundle_creation() {
        let oid = "a".repeat(40);
        assert_eq!(
            object_inventory(
                format!("{oid} {OBJECT_BYTES}\n").as_bytes(),
                &ObjectFormat::Sha1
            )
            .unwrap(),
            (1, OBJECT_BYTES)
        );
        assert_eq!(
            object_inventory(
                format!("{oid} {}\n", OBJECT_BYTES + 1).as_bytes(),
                &ObjectFormat::Sha1
            )
            .unwrap_err()
            .kind,
            ErrorKind::OutputLimit
        );
        assert_eq!(
            object_inventory(
                format!("{oid} 1\n").repeat(OBJECT_COUNT + 1).as_bytes(),
                &ObjectFormat::Sha1
            )
            .unwrap_err()
            .kind,
            ErrorKind::OutputLimit
        );
        assert!(object_inventory(b"bad 1\n", &ObjectFormat::Sha1).is_err());
        assert!(object_inventory(format!("{oid} -1\n").as_bytes(), &ObjectFormat::Sha1).is_err());
    }
    #[test]
    fn artifact_budget_rejects_sparse_oversize_files_and_symlinks() {
        let root = std::env::temp_dir().join(format!("gitfleet-capture-budget-{}", Uuid::new_v4()));
        fs::create_dir(&root).unwrap();
        let file = File::create(root.join("pack")).unwrap();
        file.set_len(ARTIFACT_BYTES + 1).unwrap();
        assert_eq!(
            artifact_budget(&root).unwrap_err().kind,
            ErrorKind::OutputLimit
        );
        file.set_len(1).unwrap();
        assert!(artifact_budget(&root).is_ok());
        std::os::unix::fs::symlink("pack", root.join("alias")).unwrap();
        assert!(artifact_budget(&root).is_err());
        fs::remove_dir_all(root).unwrap();
    }
}
