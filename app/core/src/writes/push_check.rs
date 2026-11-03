//! Native-approved pre-push checks only. There is deliberately no publication path.
use super::push_preview::remaining;
use super::*;
use crate::remote::{push::*, push_check::*};

#[derive(Serialize, Deserialize)]
struct Stored {
    version: u32,
    session: String,
    value: PushCheck,
}
fn stale() -> Error {
    fail(
        ErrorKind::StaleReview,
        "Pre-push check changed; obtain fresh evidence and review again",
    )
}
fn plan_digest(plan: &PushCheckPlan) -> Result<String> {
    let mut value = plan.clone();
    value.digest.clear();
    Ok(digest(&serde_json::to_vec(&value)?))
}
fn hook(meta: &Metadata, options: &Options) -> Result<Option<Hook>> {
    let (_, values, _) = adapter::config_snapshot(
        &meta.root.to_path()?,
        &options.process,
        options.global_config.as_deref(),
    )?;
    Ok(policy::named_hooks(meta, &values, &["pre-push"])?
        .into_iter()
        .next())
}
fn eligible<'a>(plan: &'a PushDependencyPlan, worktree: &str) -> Result<&'a PushPreview> {
    if plan.order != [worktree]
        || !plan.cycle.is_empty()
        || !plan.issues.is_empty()
        || plan
            .dependencies
            .iter()
            .any(|e| e.state != DependencyState::AlreadyReachable)
    {
        return Err(unsupported("Pre-push check requires every child commit already reachable on its explicitly observed destination; conditional child publication is not success"));
    }
    let preview = &plan.items.first().ok_or_else(stale)?.preview;
    if preview.relation != PushRelation::FastForward || preview.outgoing_count == 0 {
        return Err(unsupported(
            "Pre-push check requires outgoing commits and a non-force fast-forward comparison",
        ));
    }
    Ok(preview)
}
impl ControlledWrites {
    fn persist_push_check(&mut self, stored: &Stored) -> Result<()> {
        let payload = serde_json::to_string(stored)?;
        if payload.len() > 2 * 1024 * 1024 {
            return Err(fail(ErrorKind::OutputLimit, "Pre-push check exceeds 2 MiB"));
        }
        self.workspace.connection().execute(
            "INSERT OR REPLACE INTO state(key,value) VALUES(?1,?2)",
            params![format!("push-check:{}", stored.value.plan.id), payload],
        )?;
        Ok(())
    }
    fn decode_push_check(&self, payload: &str) -> Result<Stored> {
        if payload.len() > 2 * 1024 * 1024 {
            return Err(fail(ErrorKind::OutputLimit, "Pre-push check exceeds 2 MiB"));
        }
        let value: Stored = serde_json::from_str(payload)?;
        if value.version != 1 {
            return Err(fail(
                ErrorKind::NewerSchema,
                "Unsupported pre-push check version",
            ));
        }
        if value.value.published || value.value.plan.digest != plan_digest(&value.value.plan)? {
            return Err(stale());
        }
        Ok(value)
    }
    fn load_push_check(&mut self, id: &str) -> Result<Stored> {
        Uuid::parse_str(id).map_err(|_| stale())?;
        let payload: Option<String> = self
            .workspace
            .connection()
            .query_row(
                "SELECT value FROM state WHERE key=?1",
                [format!("push-check:{id}")],
                |r| r.get(0),
            )
            .optional()?;
        let stored = self.decode_push_check(&payload.ok_or_else(stale)?)?;
        if stored.value.plan.id != id {
            return Err(stale());
        }
        Ok(stored)
    }
    pub(crate) fn push_check_audit_detail(&self, payload: &str) -> Result<PushCheck> {
        let stored = self.decode_push_check(payload)?;
        let mut value = stored.value;
        if value.phase == PushCheckPhase::Running
            || (value.phase == PushCheckPhase::Proposed && stored.session != self.session)
        {
            value.phase = PushCheckPhase::Interrupted;
            value.issue=Some("Check interrupted. Hook effects may require inspection. No publication was started by Core; hooks are never replayed. Start a fresh separately approved check.".into());
        }
        Ok(value)
    }
    pub(crate) fn propose_push_check(
        &mut self,
        workspace: &WorkspaceId,
        entries: &BTreeMap<String, crate::workspace::StoredWorktree>,
        selection: PushPlanningSelection,
        hooks: HookPolicy,
        options: &ReadOptions,
    ) -> Result<PushCheckPlan> {
        let previous = std::mem::replace(&mut self.options.process, options.clone());
        let result = (|| {
            let start = Instant::now();
            let dependency =
                self.push_dependency_plan(workspace, entries, vec![selection.clone()], options)?;
            let preview = eligible(&dependency, &selection.worktree)?.clone();
            let entry = entries.get(&selection.worktree).ok_or_else(stale)?;
            let transport = self.prepare_transport_plan(
                workspace,
                entry,
                &preview.remote,
                &preview.destination_ref,
                &remaining(options, start)?,
            )?;
            let hook = hook(&entry.metadata, &self.options)?;
            policy::validate_hook_manifest(&hook.iter().cloned().collect::<Vec<_>>(), hooks)?;
            let hook_stdin = format!(
                "{} {} {} {}\n",
                preview.source_ref,
                preview.source_oid.hex,
                preview.destination_ref,
                preview.observed_oid.hex
            );
            let mut plan = PushCheckPlan {
                id: Uuid::new_v4().to_string(),
                workspace: workspace.clone(),
                worktree: entry.id.clone(),
                created_at_ms: now_ms(),
                selection,
                hook_args: [preview.remote.clone(), preview.destination.clone()],
                preview,
                dependencies: dependency.dependencies,
                transport,
                hooks,
                hook,
                hook_stdin,
                digest: String::new(),
            };
            plan.digest = plan_digest(&plan)?;
            self.revalidate_push_check(&plan, entries, &remaining(options, start)?)?;
            self.persist_push_check(&Stored {
                version: 1,
                session: self.session.clone(),
                value: PushCheck {
                    source_oid: plan.preview.source_oid.clone(),
                    plan: plan.clone(),
                    phase: PushCheckPhase::Proposed,
                    checked_at_ms: None,
                    hook_passed: false,
                    published: false,
                    issue: None,
                },
            })?;
            Ok(plan)
        })();
        self.options.process = previous;
        result
    }
    fn revalidate_push_check(
        &mut self,
        plan: &PushCheckPlan,
        entries: &BTreeMap<String, crate::workspace::StoredWorktree>,
        options: &ReadOptions,
    ) -> Result<()> {
        let start = Instant::now();
        let current = self.push_dependency_plan(
            &plan.workspace,
            entries,
            vec![plan.selection.clone()],
            options,
        )?;
        let preview = eligible(&current, &plan.worktree.0)?;
        if preview.source_oid != plan.preview.source_oid
            || preview.source_ref != plan.preview.source_ref
            || preview.generation != plan.preview.generation
            || preview.config_digest != plan.preview.config_digest
            || preview.destination != plan.preview.destination
            || preview.destination_ref != plan.preview.destination_ref
            || preview.observed_oid != plan.preview.observed_oid
            || serde_json::to_vec(&current.dependencies)? != serde_json::to_vec(&plan.dependencies)?
        {
            return Err(stale());
        }
        let entry = entries.get(&plan.worktree.0).ok_or_else(stale)?;
        if hook(&entry.metadata, &self.options)? != plan.hook {
            return Err(stale());
        }
        self.revalidate_transport(&plan.transport, &remaining(options, start)?)?;
        Ok(())
    }
    pub(crate) fn cancel_push_check(&mut self, id: &str) -> Result<()> {
        let mut stored = self.load_push_check(id)?;
        if stored.session != self.session || stored.value.phase != PushCheckPhase::Proposed {
            return Err(stale());
        }
        stored.value.phase = PushCheckPhase::Cancelled;
        self.persist_push_check(&stored)
    }
    pub(crate) fn execute_push_check(
        &mut self,
        plan: &PushCheckPlan,
        entries: &BTreeMap<String, crate::workspace::StoredWorktree>,
        cancellation: inspection::Cancellation,
    ) -> Result<PushCheck> {
        let mut stored = self.load_push_check(&plan.id)?;
        if stored.session != self.session
            || stored.value.phase != PushCheckPhase::Proposed
            || serde_json::to_vec(&stored.value.plan)? != serde_json::to_vec(plan)?
        {
            return Err(stale());
        }
        let mut options = self.options.process.clone();
        options.timeout = Duration::from_secs(60);
        options.cancellation = cancellation;
        let previous = std::mem::replace(&mut self.options.process, options.clone());
        let result = (|| {
            let start = Instant::now();
            stored.value.phase = PushCheckPhase::Running;
            self.persist_push_check(&stored)?;
            let result = (|| {
                self.revalidate_push_check(plan, entries, &remaining(&options, start)?)?;
                if plan.hooks == HookPolicy::Execute {
                    if let Some(hook) = plan.hook.as_ref().filter(|h| h.executable) {
                        let meta = &plan.transport.metadata;
                        if adapter::program_fingerprint(&hook.resolved.to_path()?)?
                            != hook.fingerprint
                        {
                            return Err(stale());
                        }
                        let mut command = adapter::clean(&hook.resolved.to_path()?);
                        command
                            .current_dir(meta.root.to_path()?)
                            .args(&plan.hook_args)
                            .env("GIT_DIR", meta.git_dir.to_path()?)
                            .env("GIT_COMMON_DIR", meta.common_dir.to_path()?)
                            .env("GIT_WORK_TREE", meta.root.to_path()?)
                            .env("GIT_INDEX_FILE", meta.git_dir.to_path()?.join("index"))
                            .env("GIT_OPTIONAL_LOCKS", "0")
                            .env("GIT_CONFIG_NOSYSTEM", "1")
                            .env("GIT_CONFIG_GLOBAL", "/dev/null")
                            .env("GIT_CONFIG_COUNT", "1")
                            .env("GIT_CONFIG_KEY_0", "core.hooksPath")
                            .env("GIT_CONFIG_VALUE_0", "");
                        let output = process::bounded_input(
                            command,
                            &remaining(&options, start)?,
                            Some(plan.hook_stdin.as_bytes().to_vec()),
                        )?;
                        if !output.status.success() {
                            return Err(fail(ErrorKind::HookRejected,"Reviewed pre-push hook rejected the check; raw hook output is not retained"));
                        }
                        stored.value.hook_passed = true;
                    }
                }
                self.revalidate_push_check(plan, entries, &remaining(&options, start)?)?;
                Ok(())
            })();
            stored.value.checked_at_ms = Some(now_ms());
            match &result {
                Ok(()) => stored.value.phase = PushCheckPhase::Complete,
                Err(e) => {
                    stored.value.phase = if e.kind == ErrorKind::Cancelled {
                        PushCheckPhase::Cancelled
                    } else {
                        PushCheckPhase::Blocked
                    };
                    stored.value.issue = Some(e.to_string());
                }
            }
            self.persist_push_check(&stored)?;
            result?;
            Ok(stored.value)
        })();
        self.options.process = previous;
        result
    }
}
