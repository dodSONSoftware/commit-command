//! One-use, native-reviewed version replacement with durable non-replaying recovery.
use super::*;
use crate::actions::{
    semver::*,
    version::*,
    version_file::{self, Target},
};
use crate::domain::Issue;
use std::os::unix::fs::MetadataExt;
const LIMIT: usize = 64 * 1024;
#[derive(Serialize, Deserialize)]
struct Stored {
    version: u32,
    session: String,
    value: VersionInvocation,
}
fn stale() -> Error {
    fail(
        ErrorKind::StaleReview,
        "Version Action changed; inspect and review again",
    )
}
fn plan_digest(plan: &VersionPlan) -> Result<String> {
    let mut p = plan.clone();
    p.digest.clear();
    Ok(digest(&serde_json::to_vec(&p)?))
}
fn projected(mut value: VersionInvocation, same_session: bool) -> VersionInvocation {
    if matches!(
        value.phase,
        VersionPhase::Preparing | VersionPhase::Replacing
    ) {
        value.phase = VersionPhase::Unconfirmed;
    } else if value.phase == VersionPhase::Proposed && !same_session {
        value.phase = VersionPhase::Interrupted;
    }
    value
}
impl ControlledWrites {
    pub(crate) fn retained_version_file(
        &mut self,
        workspace: &WorkspaceId,
        id: &str,
    ) -> Result<RetainedVersionFile> {
        let stored = self.load_version(workspace, id)?;
        let relative = PathBuf::from("operations")
            .join(id)
            .join("version.candidate");
        let location = NativePath::from_path(&self.workspace.data_directory().join(&relative));
        // Pin every component from the app-state root too; an operation ID is
        // never interpreted as a renderer-selected filesystem location.
        let inspected = Target::open(
            self.workspace.data_directory(),
            &NativePath::from_path(&relative),
        )
        .and_then(|target| target.read());
        let (file, bytes) = inspected.map_err(|error| {
            Error::with_kind(
                error.kind,
                format!(
                    "Retained file cannot be displayed inline (owned regular file, at most 1024 bytes required). Inspect {} without overwriting the current version; recovery data was preserved",
                    location.display()
                ),
            )
        })?;
        Ok(RetainedVersionFile {
            operation: id.into(),
            location,
            fingerprint: digest(&serde_json::to_vec(&(&file, &bytes))?),
            matches_recorded_before: bytes == stored.value.plan.version.before,
            bytes,
            read_at_ms: now_ms(),
        })
    }
    fn persist_version(&mut self, stored: &Stored) -> Result<()> {
        let raw = serde_json::to_string(stored)?;
        if raw.len() > LIMIT {
            return Err(fail(
                ErrorKind::OutputLimit,
                "Version Action record exceeds limit",
            ));
        }
        self.workspace.connection().execute(
            "INSERT OR REPLACE INTO state(key,value) VALUES(?1,?2)",
            params![format!("version-action:{}", stored.value.plan.id), raw],
        )?;
        Ok(())
    }
    fn decode_version(raw: &str) -> Result<Stored> {
        if raw.len() > LIMIT {
            return Err(fail(
                ErrorKind::OutputLimit,
                "Version Action record exceeds limit",
            ));
        }
        let stored: Stored = serde_json::from_str(raw)?;
        if stored.version != 1 {
            return Err(fail(
                ErrorKind::NewerSchema,
                "Unsupported version Action journal",
            ));
        }
        stored.value.plan.version.validate()?;
        if plan_digest(&stored.value.plan)? != stored.value.plan.digest {
            return Err(stale());
        }
        Ok(stored)
    }
    pub(crate) fn version_audit_detail(&self, raw: &str) -> Result<VersionInvocation> {
        let s = Self::decode_version(raw)?;
        Ok(projected(s.value, s.session == self.session))
    }
    fn load_version(&mut self, workspace: &WorkspaceId, id: &str) -> Result<Stored> {
        Uuid::parse_str(id).map_err(|_| stale())?;
        let raw: Option<String> = self
            .workspace
            .connection()
            .query_row(
                "SELECT value FROM state WHERE key=?1",
                [format!("version-action:{id}")],
                |r| r.get(0),
            )
            .optional()?;
        let stored = Self::decode_version(&raw.ok_or_else(stale)?)?;
        if stored.value.plan.id != id || stored.value.plan.workspace != *workspace {
            return Err(stale());
        }
        Ok(stored)
    }
    pub(crate) fn version_action(
        &mut self,
        workspace: &WorkspaceId,
        id: &str,
    ) -> Result<VersionInvocation> {
        let stored = self.load_version(workspace, id)?;
        Ok(projected(stored.value, stored.session == self.session))
    }
    pub(crate) fn version_actions(
        &mut self,
        workspace: &WorkspaceId,
        worktree: Option<&str>,
        offset: usize,
    ) -> Result<VersionPage> {
        if offset > 1_000_000 {
            return Err(fail(
                ErrorKind::InvalidRequest,
                "Invalid version Action offset",
            ));
        }
        let mut stmt=self.workspace.connection().prepare("SELECT CASE WHEN length(CAST(value AS BLOB))<=65536 THEN value ELSE NULL END FROM state WHERE key LIKE 'version-action:%' AND json_extract(value,'$.value.plan.workspace')=?1 AND (?2 IS NULL OR json_extract(value,'$.value.plan.worktree')=?2) ORDER BY json_extract(value,'$.value.plan.created_at_ms') DESC,key LIMIT 51 OFFSET ?3")?;
        let rows = stmt.query_map(params![workspace.0, worktree, offset as i64], |r| {
            r.get::<_, Option<String>>(0)
        })?;
        let mut items = Vec::new();
        let mut more = false;
        for raw in rows {
            if items.len() == 50 {
                more = true;
                break;
            }
            let s =
                Self::decode_version(&raw?.ok_or_else(|| {
                    fail(ErrorKind::OutputLimit, "Version journal exceeds limit")
                })?)?;
            items.push(projected(s.value, s.session == self.session));
        }
        let next = more.then_some(offset + items.len());
        Ok(VersionPage { items, next })
    }
    pub(crate) fn propose_version(
        &mut self,
        workspace: &WorkspaceId,
        worktree: &WorktreeId,
        path: NativePath,
        increment: Increment,
        prerelease: PrereleasePolicy,
    ) -> Result<VersionPlan> {
        let entry = self.entry(workspace, worktree)?;
        let metadata = entry.metadata;
        self.ensure_fetch_import_resolved(&metadata.common_binding, None)?;
        self.ensure_upstream_resolved(&metadata.common_binding, None)?;
        let (index, _, branch, head) = policy::preflight(&metadata, &self.options)?;
        let config = policy::effective(&metadata, SigningPreference::Unsigned, &self.options)?;
        self.trusted(worktree, &metadata, &config)?;
        let version = crate::actions::preview_version_file(
            &metadata.root.to_path()?,
            &path,
            increment,
            prerelease,
            &self.options.process,
        )?;
        let target = Target::open(&metadata.root.to_path()?, &path)?;
        let (file, bytes) = target.read()?;
        if bytes != version.before {
            return Err(stale());
        }
        let (root_device, root_inode, parent_device, parent_inode) = target.identities()?;
        if fs::metadata(self.workspace.data_directory())?.dev() != parent_device {
            return Err(unsupported(
                "Version recovery storage and target must share the tested filesystem",
            ));
        }
        let mut plan = VersionPlan {
            id: Uuid::new_v4().to_string(),
            workspace: workspace.clone(),
            worktree: worktree.clone(),
            metadata,
            path,
            version,
            file,
            root_device,
            root_inode,
            parent_device,
            parent_inode,
            config_digest: config.config_hash,
            index_digest: digest(&index),
            branch,
            head,
            created_at_ms: now_ms(),
            digest: String::new(),
        };
        plan.digest = plan_digest(&plan)?;
        self.persist_version(&Stored {
            version: 1,
            session: self.session.clone(),
            value: VersionInvocation {
                plan: plan.clone(),
                phase: VersionPhase::Proposed,
                definition_approved_at_ms: None,
                candidate: None,
                applied: false,
                observed: None,
                finished_at_ms: None,
                issue: None,
            },
        })?;
        Ok(plan)
    }
    pub(crate) fn cancel_version(&mut self, workspace: &WorkspaceId, id: &str) -> Result<()> {
        let mut stored = self.load_version(workspace, id)?;
        if stored.session != self.session || stored.value.phase != VersionPhase::Proposed {
            return Err(stale());
        }
        stored.value.phase = VersionPhase::Cancelled;
        stored.value.finished_at_ms = Some(now_ms());
        self.persist_version(&stored)
    }
    fn version_target(&self, plan: &VersionPlan) -> Result<Target> {
        let target = Target::open(&plan.metadata.root.to_path()?, &plan.path)?;
        if target.identities()?
            != (
                plan.root_device,
                plan.root_inode,
                plan.parent_device,
                plan.parent_inode,
            )
        {
            return Err(stale());
        }
        Ok(target)
    }
    pub(crate) fn execute_version(
        &mut self,
        plan: &VersionPlan,
        cancellation: inspection::Cancellation,
    ) -> Result<VersionInvocation> {
        let mut stored = self.load_version(&plan.workspace, &plan.id)?;
        if stored.session != self.session
            || stored.value.phase != VersionPhase::Proposed
            || serde_json::to_vec(&stored.value.plan)? != serde_json::to_vec(plan)?
        {
            return Err(stale());
        }
        let mut target = None;
        let result = (|| -> Result<()> {
            if cancellation.is_cancelled() {
                return Err(fail(
                    ErrorKind::Cancelled,
                    "Version Action cancelled before replacement",
                ));
            }
            let entry = self.entry(&plan.workspace, &plan.worktree)?;
            if policy::binding(&entry.metadata)? != policy::binding(&plan.metadata)? {
                return Err(stale());
            }
            let _common = crate::locking::ExclusiveLock::acquire(
                File::open(plan.metadata.common_dir.to_path()?)?,
                "Shared repository is write-locked",
            )?;
            self.ensure_fetch_import_resolved(&plan.metadata.common_binding, None)?;
            self.ensure_upstream_resolved(&plan.metadata.common_binding, None)?;
            let (index, _, branch, head) = policy::preflight(&entry.metadata, &self.options)?;
            let config =
                policy::effective(&entry.metadata, SigningPreference::Unsigned, &self.options)?;
            self.trusted(&plan.worktree, &entry.metadata, &config)?;
            if config.config_hash != plan.config_digest
                || digest(&index) != plan.index_digest
                || branch != plan.branch
                || head != plan.head
            {
                return Err(stale());
            }
            let pinned = self.version_target(plan)?;
            let (file, bytes) = pinned.read()?;
            if file != plan.file || bytes != plan.version.before {
                return Err(stale());
            }
            target = Some(pinned);
            stored.value.definition_approved_at_ms = Some(now_ms());
            stored.value.phase = VersionPhase::Preparing;
            self.persist_version(&stored)?;
            let pinned = target.as_ref().expect("opened target");
            let artifact_path = self.directory(&plan.id)?;
            store::private_dir(&artifact_path)?;
            let artifact = version_file::private_artifact(&artifact_path)?;
            let candidate = pinned.candidate(&artifact, &plan.version.after, plan.file.mode)?;
            stored.value.candidate = Some(candidate.clone());
            stored.value.phase = VersionPhase::Replacing;
            self.persist_version(&stored)?;
            #[cfg(feature = "write-test-support")]
            self.version_checkpoint(plan, false)?;
            if cancellation.is_cancelled() {
                return Err(fail(
                    ErrorKind::Cancelled,
                    "Version Action cancelled before replacement",
                ));
            }
            self.version_target(plan)?;
            pinned.replace(&artifact, &candidate, &plan.file, &plan.version.before)?;
            #[cfg(feature = "write-test-support")]
            self.version_checkpoint(plan, true)?;
            let (after, bytes) = pinned.read()?;
            if bytes != plan.version.after
                || after.device != candidate.device
                || after.inode != candidate.inode
                || after.mode != plan.file.mode
            {
                return Err(fail(
                    ErrorKind::UncertainOutcome,
                    "Version result changed after replacement; inspect without replay",
                ));
            }
            stored.value.phase = VersionPhase::Applied;
            stored.value.applied = true;
            stored.value.observed = Some(ObservedVersionState::Applied);
            Ok(())
        })();
        if let Err(error) = result {
            stored.value.phase = if matches!(
                stored.value.phase,
                VersionPhase::Preparing | VersionPhase::Replacing
            ) {
                VersionPhase::Unconfirmed
            } else if error.kind == ErrorKind::Cancelled {
                VersionPhase::Cancelled
            } else {
                VersionPhase::Blocked
            };
            stored.value.issue = Some(Issue {
                kind: error.kind,
                message: error.to_string(),
            });
        }
        // Retain the exchanged original (or an unfinished candidate) in private
        // recovery storage. Never delete a raced external edit or roll it back.
        stored.value.finished_at_ms = Some(now_ms());
        self.persist_version(&stored)?;
        Ok(stored.value)
    }
    pub(crate) fn review_version_recovery(
        &mut self,
        workspace: &WorkspaceId,
        id: &str,
    ) -> Result<VersionRecovery> {
        let invocation = self.version_action(workspace, id)?;
        if invocation.phase != VersionPhase::Unconfirmed {
            return Err(stale());
        }
        self.entry(workspace, &invocation.plan.worktree)?;
        let target = self.version_target(&invocation.plan)?;
        let (file, bytes) = target.read()?;
        let artifact_path = self.directory(&invocation.plan.id)?;
        let backup = if artifact_path.exists() {
            Some(version_file::read_artifact(
                &version_file::private_artifact(&artifact_path)?,
            )?)
        } else {
            None
        };
        let backup_fingerprint = digest(&serde_json::to_vec(&backup)?);
        let mut observed = if file.mode != invocation.plan.file.mode {
            ObservedVersionState::Conflict
        } else {
            invocation.plan.version.classify(&bytes)?
        };
        if observed == ObservedVersionState::Applied
            && !backup.as_ref().is_some_and(|(saved, bytes)| {
                version_file::same_previous_file(saved, &invocation.plan.file)
                    && bytes == &invocation.plan.version.before
            })
        {
            observed = ObservedVersionState::Conflict;
        }
        Ok(VersionRecovery {
            invocation,
            observed,
            file,
            backup_fingerprint,
        })
    }
    pub(crate) fn recover_version(
        &mut self,
        review: &VersionRecovery,
    ) -> Result<VersionInvocation> {
        let plan = &review.invocation.plan;
        let _common = crate::locking::ExclusiveLock::acquire(
            File::open(plan.metadata.common_dir.to_path()?)?,
            "Shared repository is write-locked",
        )?;
        let fresh = self.review_version_recovery(&plan.workspace, &plan.id)?;
        if serde_json::to_vec(&fresh)? != serde_json::to_vec(review)? {
            return Err(stale());
        }
        let mut stored = self.load_version(&plan.workspace, &plan.id)?;
        stored.value.phase = VersionPhase::Reconciled;
        stored.value.observed = Some(review.observed);
        stored.value.finished_at_ms = Some(now_ms());
        self.persist_version(&stored)?;
        Ok(stored.value)
    }
    pub(crate) fn version_stage(&mut self, workspace: &WorkspaceId, id: &str) -> Result<Proposal> {
        let value = self.version_action(workspace, id)?;
        if !matches!(
            value.phase,
            VersionPhase::Applied | VersionPhase::Reconciled
        ) || value.observed != Some(ObservedVersionState::Applied)
        {
            return Err(stale());
        }
        self.entry(workspace, &value.plan.worktree)?;
        let target = self.version_target(&value.plan)?;
        let (file, bytes) = target.read()?;
        if bytes != value.plan.version.after || file.mode != value.plan.file.mode {
            return Err(stale());
        }
        self.propose(
            workspace,
            &value.plan.worktree,
            WriteRequest::Stage {
                paths: vec![value.plan.path],
                hooks: HookPolicy::Execute,
            },
        )
    }
    pub(crate) fn version_notices(
        &mut self,
    ) -> Result<Vec<crate::application::RemoteRecoveryNotice>> {
        let mut stmt=self.workspace.connection().prepare("SELECT json_extract(value,'$.value.plan.id'),json_extract(value,'$.value.plan.workspace'),json_extract(value,'$.value.plan.worktree') FROM state WHERE key LIKE 'version-action:%' AND json_extract(value,'$.value.phase') IN ('preparing','replacing','unconfirmed') ORDER BY key LIMIT 100")?;
        let rows = stmt.query_map([], |r| {
            Ok(crate::application::RemoteRecoveryNotice {
                operation_id: r.get(0)?,
                workspace_id: r.get(1)?,
                worktree_id: r.get(2)?,
            })
        })?;
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(Error::from)
    }
    #[cfg(feature = "write-test-support")]
    pub(crate) fn set_version_fault(&mut self, after: bool, kind: TestFaultKind) {
        self.version_fault = Some((after, kind));
    }
    #[cfg(feature = "write-test-support")]
    fn version_checkpoint(&mut self, plan: &VersionPlan, after: bool) -> Result<()> {
        if !self.version_fault.as_ref().is_some_and(|f| f.0 == after) {
            return Ok(());
        }
        let root = plan.metadata.root.to_path()?;
        if !root.starts_with(fs::canonicalize(std::env::temp_dir())?)
            || !root.join(".gitfleet-m2-fixture").is_file()
        {
            return Err(fail(
                ErrorKind::InvalidRequest,
                "Fault injection requires a disposable fixture",
            ));
        }
        match self.version_fault.take().expect("checked fault").1 {
            TestFaultKind::Sigkill => {
                unsafe {
                    libc::kill(libc::getpid(), libc::SIGKILL);
                }
                loop {
                    std::thread::park();
                }
            }
            _ => Err(fail(
                ErrorKind::Storage,
                "Injected version replacement storage failure",
            )),
        }
    }
}
