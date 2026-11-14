//! A new native approval authorizes exactly one ancestry-checked, expected-old ref update.
use super::push_preview::remaining;
use super::*;
use crate::domain::ObjectId;
use crate::remote::{
    observation::ObservationPlan, push::PushPlanningSelection, push_check::PushCheckPhase,
    push_publication::*,
};

const RECORD_LIMIT: usize = 2 * 1024 * 1024;
const PACK_LIMIT: usize = 128 * 1024 * 1024;
const SOURCE_REF: &str = "refs/heads/gitfleet-approved-push";

#[derive(Serialize, Deserialize)]
struct Stored {
    version: u32,
    session: String,
    value: PushPublication,
}
fn stale() -> Error {
    fail(
        ErrorKind::StaleReview,
        "Push publication review changed; obtain fresh evidence and approval",
    )
}
fn plan_digest(plan: &PushPublicationPlan) -> Result<String> {
    let mut value = plan.clone();
    value.digest.clear();
    Ok(digest(&serde_json::to_vec(&value)?))
}
fn observed_state(value: &PushPublication, oid: Option<&ObjectId>) -> PushRemoteState {
    match oid {
        Some(oid) if oid == &value.plan.check.preview.source_oid => {
            PushRemoteState::ApprovedSourcePresent
        }
        Some(oid) if oid == &value.plan.check.preview.observed_oid => {
            PushRemoteState::ExpectedOldPresent
        }
        Some(_) => PushRemoteState::Different,
        None => PushRemoteState::Missing,
    }
}
fn probe(
    plan: &ObservationPlan,
    repository: &Path,
    home: &Path,
    url: &str,
    options: &ReadOptions,
) -> Result<Option<ObjectId>> {
    let mut command = remote::command(plan, repository, home)?;
    command.args([
        "ls-remote",
        "--refs",
        "--exit-code",
        "--",
        url,
        &plan.reference,
    ]);
    let output = process::transport(command, options, || Ok(()))?;
    if output.status.code() == Some(2) && output.stdout.is_empty() {
        return Ok(None);
    }
    if !output.status.success() {
        return Err(fail(
            ErrorKind::InspectionFailed,
            "Push destination observation failed; raw server output is not retained",
        ));
    }
    let line = inspection::output_text(output.stdout)?;
    let (oid, reference) = line
        .split_once('\t')
        .ok_or_else(|| fail(ErrorKind::InvalidData, "Invalid push destination response"))?;
    if reference != plan.reference {
        return Err(stale());
    }
    ObjectId::parse(oid, &plan.metadata.object_format).map(Some)
}
struct IsolatedTransport<'a> {
    plan: &'a ObservationPlan,
    repository: &'a Path,
    home: &'a Path,
    artifact: &'a Path,
}
impl IsolatedTransport<'_> {
    fn run(
        &self,
        args: &[&str],
        input: Option<Vec<u8>>,
        objects: Option<&Path>,
        options: &ReadOptions,
    ) -> Result<Vec<u8>> {
        let mut command = remote::command(self.plan, self.repository, self.home)?;
        command.args(args);
        if let Some(objects) = objects {
            command.env("GIT_OBJECT_DIRECTORY", objects);
        }
        let output = process::bounded_input(command, options, input)?;
        fetch::check_artifact_budget(self.artifact, PACK_LIMIT as u64, 1024)?;
        if !output.status.success() {
            return Err(fail(
                ErrorKind::InspectionFailed,
                "Isolated push preparation failed; managed repository remains unchanged",
            ));
        }
        Ok(output.stdout)
    }
    fn index_pack(&self, pack: Vec<u8>, options: &ReadOptions) -> Result<()> {
        let trailer_len = if self.plan.metadata.object_format == ObjectFormat::Sha1 {
            20
        } else {
            32
        };
        if pack.len() < 12 + trailer_len || !pack.starts_with(b"PACK") {
            return Err(fail(ErrorKind::InvalidData, "Invalid approved source pack"));
        }
        let name = pack[pack.len() - trailer_len..]
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        let path = self
            .repository
            .join("objects/pack")
            .join(format!("pack-{name}.pack"));
        store::write_new(&path, &pack)?;
        drop(pack);
        let mut command = remote::command(self.plan, self.repository, self.home)?;
        command.args(["index-pack", "--strict"]).arg(&path);
        let output = process::transport(command, options, || {
            fetch::check_artifact_budget(self.artifact, PACK_LIMIT as u64, 1024)
        })?;
        if !output.status.success() {
            return Err(fail(
                ErrorKind::InvalidData,
                "Approved source pack verification failed",
            ));
        }
        Ok(())
    }
}

impl ControlledWrites {
    #[cfg(feature = "write-test-support")]
    pub(crate) fn set_publication_fault(&mut self, after_transport: bool, kind: TestFaultKind) {
        self.publication_fault = Some((after_transport, kind));
    }
    #[cfg(feature = "write-test-support")]
    fn publication_checkpoint(
        &mut self,
        plan: &PushPublicationPlan,
        after_transport: bool,
    ) -> Result<()> {
        if !self
            .publication_fault
            .as_ref()
            .is_some_and(|f| f.0 == after_transport)
        {
            return Ok(());
        }
        let root = plan.check.transport.metadata.root.to_path()?;
        if !root.starts_with(fs::canonicalize(std::env::temp_dir())?)
            || !root.join(".gitfleet-m2-fixture").is_file()
        {
            return Err(fail(
                ErrorKind::InvalidRequest,
                "Fault injection requires a disposable fixture",
            ));
        }
        match self.publication_fault.take().expect("checked fault").1 {
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
                "Injected publication storage failure",
            )),
        }
    }
    fn persist_publication(&mut self, stored: &Stored) -> Result<()> {
        let payload = serde_json::to_string(stored)?;
        if payload.len() > RECORD_LIMIT {
            return Err(fail(
                ErrorKind::OutputLimit,
                "Push publication record exceeds 2 MiB",
            ));
        }
        self.workspace.connection().execute(
            "INSERT OR REPLACE INTO state(key,value) VALUES(?1,?2)",
            params![
                format!("push-publication:{}", stored.value.plan.id),
                payload
            ],
        )?;
        Ok(())
    }
    fn decode_publication(&self, payload: &str) -> Result<Stored> {
        if payload.len() > RECORD_LIMIT {
            return Err(fail(
                ErrorKind::OutputLimit,
                "Push publication record exceeds 2 MiB",
            ));
        }
        let stored: Stored = serde_json::from_str(payload)?;
        if stored.version != 1 {
            return Err(fail(
                ErrorKind::NewerSchema,
                "Unsupported push publication record version",
            ));
        }
        if stored.value.plan.digest != plan_digest(&stored.value.plan)? {
            return Err(stale());
        }
        Ok(stored)
    }
    fn load_publication(&mut self, id: &str) -> Result<Stored> {
        Uuid::parse_str(id).map_err(|_| stale())?;
        let payload: Option<String> = self
            .workspace
            .connection()
            .query_row(
                "SELECT value FROM state WHERE key=?1",
                [format!("push-publication:{id}")],
                |r| r.get(0),
            )
            .optional()?;
        let stored = self.decode_publication(&payload.ok_or_else(stale)?)?;
        if stored.value.plan.id != id {
            return Err(stale());
        }
        Ok(stored)
    }
    pub(super) fn load_publication_for_batch(&mut self, id: &str) -> Result<PushPublication> {
        Ok(self.load_publication(id)?.value)
    }
    pub(crate) fn publication_audit_detail(&self, payload: &str) -> Result<PushPublication> {
        let stored = self.decode_publication(payload)?;
        let mut value = stored.value;
        match value.phase {
            PushPublicationPhase::Publishing => {
                value.phase = PushPublicationPhase::Unconfirmed;
                value.may_have_published = true;
            }
            PushPublicationPhase::Checking | PushPublicationPhase::Preparing => {
                value.phase = PushPublicationPhase::Interrupted
            }
            PushPublicationPhase::Proposed if stored.session != self.session => {
                value.phase = PushPublicationPhase::Interrupted
            }
            _ => {}
        }
        Ok(value)
    }
    pub(super) fn validate_publication_target(&self, plan: &ObservationPlan) -> Result<()> {
        let Some(target) = &plan.local_target else {
            return Ok(());
        };
        let target = target.to_path()?;
        // Local transport exists solely for disposable acceptance fixtures. Never
        // permit publication into any managed workspace, including metadata aliases.
        for workspace in self.workspace.list() {
            let root = fs::canonicalize(workspace.root.to_path()?)?;
            if target.starts_with(&root) || root.starts_with(&target) {
                return Err(unsupported(
                    "Push fixture destination overlaps a managed workspace",
                ));
            }
        }
        let common = plan.metadata.common_dir.to_path()?;
        if target.starts_with(&common) || common.starts_with(&target) {
            return Err(unsupported(
                "Push fixture destination overlaps managed repository metadata",
            ));
        }
        for (name, directory) in [
            ("HEAD", false),
            ("config", false),
            ("objects", true),
            ("refs", true),
        ] {
            let metadata = fs::symlink_metadata(target.join(name))?;
            if metadata.is_symlink()
                || metadata.is_dir() != directory
                || (!directory && !metadata.is_file())
            {
                return Err(unsupported(
                    "Push fixture requires standalone bare repository metadata",
                ));
            }
        }
        if target.join(".git").exists() || target.join("commondir").exists() {
            return Err(unsupported(
                "Push fixture requires a standalone bare destination",
            ));
        }
        Ok(())
    }
    pub(crate) fn propose_publication(
        &mut self,
        workspace: &WorkspaceId,
        entries: &BTreeMap<String, crate::workspace::StoredWorktree>,
        selection: PushPlanningSelection,
        hooks: HookPolicy,
        options: &ReadOptions,
        batch: Option<crate::remote::push_batch::PushBatchBinding>,
    ) -> Result<PushPublicationPlan> {
        // Always prepare a new check: a previously completed check is never publication authority.
        let check = self.propose_push_check(workspace, entries, selection, hooks, options)?;
        if let Err(error) = self.validate_publication_target(&check.transport) {
            self.cancel_push_check(&check.id)?;
            return Err(error);
        }
        let mut plan = PushPublicationPlan {
            id: Uuid::new_v4().to_string(),
            created_at_ms: now_ms(),
            check,
            digest: String::new(),
            batch,
        };
        plan.digest = plan_digest(&plan)?;
        let persisted = self.persist_publication(&Stored {
            version: 1,
            session: self.session.clone(),
            value: PushPublication {
                plan: plan.clone(),
                phase: PushPublicationPhase::Proposed,
                published: false,
                may_have_published: false,
                finished_at_ms: None,
                observed_remote_oid: None,
                remote_state: None,
                reconciled_observation: None,
                object_count: None,
                object_bytes: None,
                issue: None,
            },
        });
        if let Err(error) = persisted {
            self.cancel_push_check(&plan.check.id)?;
            return Err(error);
        }
        Ok(plan)
    }
    pub(crate) fn cancel_publication(&mut self, id: &str) -> Result<()> {
        let mut stored = self.load_publication(id)?;
        if stored.session != self.session || stored.value.phase != PushPublicationPhase::Proposed {
            return Err(stale());
        }
        self.cancel_push_check(&stored.value.plan.check.id)?;
        stored.value.phase = PushPublicationPhase::Cancelled;
        stored.value.finished_at_ms = Some(now_ms());
        self.persist_publication(&stored)?;
        self.finish_push_batch_publication(&stored.value)
    }
    pub(crate) fn execute_publication(
        &mut self,
        plan: &PushPublicationPlan,
        entries: &BTreeMap<String, crate::workspace::StoredWorktree>,
        cancellation: inspection::Cancellation,
    ) -> Result<PushPublication> {
        let mut stored = self.load_publication(&plan.id)?;
        if stored.session != self.session
            || stored.value.phase != PushPublicationPhase::Proposed
            || serde_json::to_vec(&stored.value.plan)? != serde_json::to_vec(plan)?
        {
            return Err(stale());
        }
        let mut options = self.options.process.clone();
        options.timeout = Duration::from_secs(120);
        options.cancellation = cancellation.clone();
        let previous = std::mem::replace(&mut self.options.process, options.clone());
        let result = (|| -> Result<()> {
            self.start_push_batch_publication(plan)?;
            let start = Instant::now();
            let transport = &plan.check.transport;
            self.validate_publication_target(transport)?;
            let _common = crate::locking::ExclusiveLock::acquire(
                File::open(transport.metadata.common_dir.to_path()?)?,
                "Shared repository is write-locked",
            )?;
            stored.value.phase = PushPublicationPhase::Checking;
            self.persist_publication(&stored)?;
            let check = self.execute_push_check(&plan.check, entries, cancellation)?;
            if check.phase != PushCheckPhase::Complete {
                return Err(fail(
                    if check.phase == PushCheckPhase::Cancelled {
                        ErrorKind::Cancelled
                    } else {
                        ErrorKind::HookRejected
                    },
                    "Publication stopped before transport; inspect the fresh pre-push check record",
                ));
            }
            self.revalidate_push_check(&plan.check, entries, &remaining(&options, start)?)?;
            let url = self.revalidate_transport(transport, &remaining(&options, start)?)?;
            stored.value.phase = PushPublicationPhase::Preparing;
            self.persist_publication(&stored)?;
            let (directory, home, repository) =
                self.initialize_transport(transport, &remaining(&options, start)?)?;
            let isolated = IsolatedTransport {
                plan: transport,
                repository: &repository,
                home: &home,
                artifact: &directory,
            };
            let objects = transport.metadata.common_dir.to_path()?.join("objects");
            let source = &plan.check.preview.source_oid.hex;
            let list = isolated.run(
                &["rev-list", "--objects", "--no-object-names", source],
                None,
                Some(&objects),
                &remaining(&options, start)?,
            )?;
            let count = list
                .split(|b| *b == b'\n')
                .filter(|v| !v.is_empty())
                .count();
            if count > 10_000 {
                return Err(fail(
                    ErrorKind::OutputLimit,
                    "Push source exceeds 10,000 reachable objects",
                ));
            }
            let details = isolated.run(
                &["cat-file", "--batch-check=%(objectname) %(objectsize)"],
                Some(list),
                Some(&objects),
                &remaining(&options, start)?,
            )?;
            let (count, bytes) =
                fetch::object_inventory(&details, &transport.metadata.object_format)?;
            stored.value.object_count = Some(count);
            stored.value.object_bytes = Some(bytes);
            let mut pack_options = remaining(&options, start)?;
            pack_options.max_output = 64 * 1024 * 1024;
            let pack = isolated.run(
                &["pack-objects", "--stdout", "--revs"],
                Some(format!("{source}\n").into_bytes()),
                Some(&objects),
                &pack_options,
            )?;
            isolated.index_pack(pack, &remaining(&options, start)?)?;
            isolated.run(
                &["update-ref", SOURCE_REF, source],
                None,
                None,
                &remaining(&options, start)?,
            )?;
            isolated.run(
                &[
                    "fsck",
                    "--strict",
                    "--full",
                    "--no-reflogs",
                    "--no-dangling",
                    source,
                ],
                None,
                None,
                &remaining(&options, start)?,
            )?;
            isolated.run(
                &[
                    "merge-base",
                    "--is-ancestor",
                    &plan.check.preview.observed_oid.hex,
                    source,
                ],
                None,
                None,
                &remaining(&options, start)?,
            )?;
            if probe(
                transport,
                &repository,
                &home,
                &url,
                &remaining(&options, start)?,
            )?
            .as_ref()
                != Some(&plan.check.preview.observed_oid)
            {
                return Err(fail(
                    ErrorKind::ConcurrentChange,
                    "Destination moved since observation; obtain fresh evidence and approval",
                ));
            }
            self.revalidate_push_check(&plan.check, entries, &remaining(&options, start)?)?;
            self.validate_publication_target(transport)?;
            // The lease is only the expected-old CAS guard. The isolated ancestry check above
            // is mandatory: no divergent update, user force flag or '+' refspec is exposed.
            let lease = format!(
                "--force-with-lease={}:{}",
                plan.check.preview.destination_ref, plan.check.preview.observed_oid.hex
            );
            let refspec = format!("{source}:{}", plan.check.preview.destination_ref);
            let mut command = remote::command(transport, &repository, &home)?;
            command.args([
                "push",
                "--porcelain",
                "--atomic",
                "--no-verify",
                &lease,
                "--",
                &url,
                &refspec,
            ]);
            stored.value.phase = PushPublicationPhase::Publishing;
            stored.value.may_have_published = true;
            self.persist_publication(&stored)?;
            #[cfg(feature = "write-test-support")]
            self.publication_checkpoint(plan, false)?;
            let output = process::transport(command, &remaining(&options, start)?, || {
                fetch::check_artifact_budget(&directory, PACK_LIMIT as u64, 1024)
            })?;
            #[cfg(feature = "write-test-support")]
            self.publication_checkpoint(plan, true)?;
            // Any missing/rejected/ambiguous receipt after transport starts remains uncertain.
            let receipt = std::str::from_utf8(&output.stdout).map_err(|_| stale())?;
            let updates: Vec<_> = receipt
                .lines()
                .filter(|line| line.as_bytes().get(1) == Some(&b'\t'))
                .collect();
            if !output.status.success()
                || updates.len() != 1
                || !updates[0].starts_with(" \t")
                || updates[0].split('\t').nth(1) != Some(refspec.as_str())
            {
                return Err(fail(ErrorKind::InspectionFailed,"Push transport did not provide an exact fast-forward receipt; observe destination before any fresh review"));
            }
            let oid = probe(
                transport,
                &repository,
                &home,
                &url,
                &remaining(&options, start)?,
            )?;
            stored.value.observed_remote_oid = oid.clone();
            if oid.as_ref() != Some(&plan.check.preview.source_oid) {
                return Err(fail(
                    ErrorKind::ConcurrentChange,
                    "Post-push destination verification did not match approved source",
                ));
            }
            stored.value.phase = PushPublicationPhase::Complete;
            stored.value.published = true;
            stored.value.remote_state = Some(PushRemoteState::ApprovedSourcePresent);
            Ok(())
        })();
        if let Err(error) = result {
            stored.value.phase = if stored.value.may_have_published {
                PushPublicationPhase::Unconfirmed
            } else if error.kind == ErrorKind::Cancelled {
                PushPublicationPhase::Cancelled
            } else {
                PushPublicationPhase::Blocked
            };
            stored.value.issue = Some(error.into());
        }
        stored.value.finished_at_ms = Some(now_ms());
        let cleanup = self.cancel_pending_push_check(&plan.check.id);
        let saved = self.persist_publication(&stored);
        self.options.process = previous;
        saved?;
        cleanup?;
        self.finish_push_batch_publication(&stored.value)?;
        Ok(stored.value)
    }
    pub(crate) fn publication_notices(
        &mut self,
    ) -> Result<Vec<crate::application::RemoteRecoveryNotice>> {
        let mut statement=self.workspace.connection().prepare(&format!("SELECT json_extract(value,'$.value.plan.id'),json_extract(value,'$.value.plan.check.workspace'),json_extract(value,'$.value.plan.check.worktree') FROM state WHERE {} ORDER BY key LIMIT 100", crate::workspace::recovery::PUBLICATION))?;
        let rows = statement.query_map([&self.session], |r| {
            Ok(crate::application::RemoteRecoveryNotice {
                operation_id: r.get(0)?,
                workspace_id: r.get(1)?,
                worktree_id: r.get(2)?,
            })
        })?;
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(Error::from)
    }
    pub(crate) fn inspect_publication_recovery(
        &mut self,
        workspace: &WorkspaceId,
        operation: &str,
        observation: &str,
    ) -> Result<PushPublicationRecovery> {
        let stored = self.load_publication(operation)?;
        let value = self.publication_audit_detail(&serde_json::to_string(&stored)?)?;
        if value.plan.check.workspace != *workspace
            || !matches!(
                value.phase,
                PushPublicationPhase::Unconfirmed | PushPublicationPhase::Interrupted
            )
        {
            return Err(stale());
        }
        let observed =
            self.completed_observation(observation, workspace, &value.plan.check.worktree)?;
        let expected = &value.plan.check.transport;
        // A recent pre-attempt observation says nothing about this attempt's outcome.
        // The session serializes approval/execution, so a newly proposed observation
        // after the terminal record (or after the lost review's creation) is required.
        let after = value.finished_at_ms.unwrap_or(value.plan.created_at_ms);
        if observed.plan.created_at_ms <= after
            || observed.observed_at_ms.ok_or_else(stale)? <= after
            || observed.plan.destination != expected.destination
            || observed.plan.reference != expected.reference
            || observed.plan.remote != expected.remote
            || observed.plan.metadata.root_binding != expected.metadata.root_binding
            || observed.plan.metadata.git_binding != expected.metadata.git_binding
            || observed.plan.metadata.common_binding != expected.metadata.common_binding
            || now_ms().saturating_sub(observed.observed_at_ms.ok_or_else(stale)?) > 300_000
        {
            return Err(stale());
        }
        self.revalidate_transport(&observed.plan, &self.options.process.clone())?;
        let state = observed_state(&value, observed.oid.as_ref());
        Ok(PushPublicationRecovery {
            operation: value,
            observation: observed,
            remote_state: state,
        })
    }
    pub(crate) fn recover_publication(
        &mut self,
        review: &PushPublicationRecovery,
    ) -> Result<PushPublication> {
        let current = self.inspect_publication_recovery(
            &review.operation.plan.check.workspace,
            &review.operation.plan.id,
            &review.observation.plan.id,
        )?;
        if serde_json::to_vec(&current)? != serde_json::to_vec(review)? {
            return Err(stale());
        }
        let mut stored = self.load_publication(&review.operation.plan.id)?;
        stored.value.phase = PushPublicationPhase::Reconciled;
        stored.value.observed_remote_oid = review.observation.oid.clone();
        stored.value.remote_state = Some(review.remote_state);
        stored.value.reconciled_observation = Some(review.observation.plan.id.clone());
        stored.value.issue = None;
        stored.value.finished_at_ms = Some(now_ms());
        self.persist_publication(&stored)?;
        Ok(stored.value)
    }
}
