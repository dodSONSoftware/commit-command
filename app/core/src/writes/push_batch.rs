//! Explicit child-first scheduling without automatic transport or restored authority.
use super::*;
use crate::remote::{push::*, push_batch::*, push_publication::*};
const LIMIT: usize = 4 * 1024 * 1024;
#[derive(Serialize, Deserialize)]
struct Stored {
    version: u32,
    session: String,
    value: PushBatch,
}
fn stale() -> Error {
    fail(
        ErrorKind::StaleReview,
        "Guided push queue changed or stopped; create a fresh plan and review",
    )
}
fn plan_digest(plan: &PushBatchPlan) -> Result<String> {
    let mut p = plan.clone();
    p.digest.clear();
    Ok(digest(&serde_json::to_vec(&p)?))
}
fn next(value: &PushBatch) -> Option<usize> {
    value.items.iter().position(|i| {
        !matches!(
            i.phase,
            PushBatchItemPhase::Complete | PushBatchItemPhase::SkippedObservedCurrent
        )
    })
}
fn same_source(a: &PushPreview, b: &PushPreview) -> bool {
    a.source_ref == b.source_ref
        && a.source_oid == b.source_oid
        && a.destination == b.destination
        && a.destination_ref == b.destination_ref
        && a.remote == b.remote
        && a.generation == b.generation
        && a.config_digest == b.config_digest
}
fn binding_identity(b: &ChildReachabilitySelection) -> Result<String> {
    Ok(serde_json::to_string(&(
        &b.path,
        &b.child_oid,
        &b.gitmodules_blob,
        &b.child_worktree,
    ))?)
}
fn mark_interrupted(stored: &mut Stored, session: &str) -> bool {
    let value = &mut stored.value;
    if value.phase == PushBatchPhase::Executing
        || (value.phase == PushBatchPhase::Ready && stored.session != session)
    {
        value.phase = PushBatchPhase::Interrupted;
        for item in &mut value.items {
            if !matches!(
                item.phase,
                PushBatchItemPhase::Complete | PushBatchItemPhase::SkippedObservedCurrent
            ) {
                item.phase = PushBatchItemPhase::Interrupted;
            }
        }
        return true;
    }
    false
}

impl ControlledWrites {
    fn persist_push_batch(&mut self, value: &Stored) -> Result<()> {
        let raw = serde_json::to_string(value)?;
        if raw.len() > LIMIT {
            return Err(fail(
                ErrorKind::OutputLimit,
                "Guided push queue exceeds 4 MiB",
            ));
        }
        self.workspace.connection().execute(
            "INSERT OR REPLACE INTO state(key,value) VALUES(?1,?2)",
            params![format!("push-batch:{}", value.value.plan.id), raw],
        )?;
        Ok(())
    }
    fn decode_push_batch(&self, raw: &str) -> Result<Stored> {
        if raw.len() > LIMIT {
            return Err(fail(
                ErrorKind::OutputLimit,
                "Guided push queue exceeds 4 MiB",
            ));
        }
        let value: Stored = serde_json::from_str(raw)?;
        if value.version != 1 {
            return Err(fail(
                ErrorKind::NewerSchema,
                "Unsupported guided push queue version",
            ));
        }
        if value.value.plan.digest != plan_digest(&value.value.plan)?
            || value.value.items.len() != value.value.plan.dependency_plan.order.len()
            || value
                .value
                .items
                .iter()
                .zip(&value.value.plan.dependency_plan.order)
                .any(|(i, id)| i.worktree.0 != *id)
        {
            return Err(stale());
        }
        Ok(value)
    }
    fn load_push_batch(&mut self, id: &str) -> Result<Stored> {
        Uuid::parse_str(id).map_err(|_| stale())?;
        let raw: Option<String> = self
            .workspace
            .connection()
            .query_row(
                "SELECT value FROM state WHERE key=?1",
                [format!("push-batch:{id}")],
                |r| r.get(0),
            )
            .optional()?;
        let value = self.decode_push_batch(&raw.ok_or_else(stale)?)?;
        if value.value.plan.id != id {
            return Err(stale());
        }
        Ok(value)
    }
    pub(crate) fn close_interrupted_push_batch(
        &mut self,
        workspace: &WorkspaceId,
        id: &str,
    ) -> Result<()> {
        let mut stored = self.load_push_batch(id)?;
        if stored.value.plan.workspace != *workspace
            || !mark_interrupted(&mut stored, &self.session)
        {
            return Err(stale());
        }
        self.persist_push_batch(&stored)
    }
    pub(crate) fn push_batch_session(&self) -> &str {
        &self.session
    }
    pub(crate) fn push_batch_notices(
        &mut self,
    ) -> Result<Vec<crate::application::RemoteRecoveryNotice>> {
        let mut statement=self.workspace.connection().prepare(&format!("SELECT json_extract(value,'$.value.plan.id'),json_extract(value,'$.value.plan.workspace'),(SELECT json_extract(i.value,'$.worktree') FROM json_each(state.value,'$.value.items') i WHERE json_extract(i.value,'$.phase') NOT IN ('complete','skipped_observed_current') LIMIT 1) FROM state WHERE {} ORDER BY key LIMIT 100", crate::workspace::recovery::BATCH))?;
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
    pub(crate) fn push_batch_audit_detail(&self, raw: &str) -> Result<PushBatch> {
        let mut stored = self.decode_push_batch(raw)?;
        mark_interrupted(&mut stored, &self.session);
        Ok(stored.value)
    }
    pub(crate) fn push_batch(&mut self, workspace: &WorkspaceId, id: &str) -> Result<PushBatch> {
        let stored = self.load_push_batch(id)?;
        if stored.value.plan.workspace != *workspace {
            return Err(stale());
        }
        self.push_batch_audit_detail(&serde_json::to_string(&stored)?)
    }
    pub(crate) fn push_batches(
        &mut self,
        workspace: &WorkspaceId,
        offset: usize,
    ) -> Result<PushBatchPage> {
        if offset > 1_000_000 {
            return Err(fail(
                ErrorKind::InvalidRequest,
                "Invalid guided queue offset",
            ));
        }
        let mut raw = Vec::new();
        let mut size = 0;
        let mut more = false;
        {
            let mut statement=self.workspace.connection().prepare("SELECT CASE WHEN length(CAST(value AS BLOB))<=4194304 THEN value ELSE NULL END FROM state WHERE key LIKE 'push-batch:%' AND json_extract(value,'$.value.plan.workspace')=?1 ORDER BY json_extract(value,'$.value.plan.created_at_ms') DESC,key LIMIT 51 OFFSET ?2")?;
            let rows = statement.query_map(params![workspace.0, offset as i64], |r| {
                r.get::<_, Option<String>>(0)
            })?;
            for row in rows {
                let row = row?.ok_or_else(|| {
                    fail(ErrorKind::OutputLimit, "Guided queue record exceeds limit")
                })?;
                if raw.len() == 50 || (!raw.is_empty() && size + row.len() > LIMIT) {
                    more = true;
                    break;
                }
                size += row.len();
                raw.push(row);
            }
        }
        let next = more.then_some(offset + raw.len());
        Ok(PushBatchPage {
            items: raw
                .iter()
                .map(|r| self.push_batch_audit_detail(r))
                .collect::<Result<_>>()?,
            next,
        })
    }
    pub(crate) fn create_push_batch(
        &mut self,
        workspace: &WorkspaceId,
        entries: &BTreeMap<String, crate::workspace::StoredWorktree>,
        selections: Vec<PushBatchSelection>,
        options: &ReadOptions,
    ) -> Result<PushBatch> {
        let previous = std::mem::replace(&mut self.options.process, options.clone());
        let result = (|| {
            let start = Instant::now();
            let dependency_plan = self.push_dependency_plan(
                workspace,
                entries,
                selections.iter().map(|s| s.selection.clone()).collect(),
                options,
            )?;
            if dependency_plan.order.is_empty()
                || !dependency_plan.cycle.is_empty()
                || !dependency_plan.issues.is_empty()
            {
                return Err(unsupported(
                    "Guided push queue requires a complete acyclic unblocked dependency plan",
                ));
            }
            // Known trust, hook and transport blockers are checked for every selected item
            // before any queue item can receive publication authority.
            for s in &selections {
                let entry = entries.get(&s.selection.worktree).ok_or_else(stale)?;
                let preview = &dependency_plan
                    .items
                    .iter()
                    .find(|p| p.worktree == s.selection.worktree)
                    .ok_or_else(stale)?
                    .preview;
                let transport = self.prepare_transport_plan(
                    workspace,
                    entry,
                    &preview.remote,
                    &preview.destination_ref,
                    &super::push_preview::remaining(options, start)?,
                )?;
                self.validate_publication_target(&transport)?;
                let hook = super::push_check::hook(&entry.metadata, &self.options)?;
                policy::validate_hook_manifest(&hook.iter().cloned().collect::<Vec<_>>(), s.hooks)?;
            }
            let mut plan = PushBatchPlan {
                id: Uuid::new_v4().to_string(),
                workspace: workspace.clone(),
                created_at_ms: now_ms(),
                selections,
                dependency_plan,
                digest: String::new(),
            };
            plan.digest = plan_digest(&plan)?;
            let items = plan
                .dependency_plan
                .order
                .iter()
                .map(|id| {
                    let preview = &plan
                        .dependency_plan
                        .items
                        .iter()
                        .find(|p| p.worktree == *id)
                        .expect("validated order")
                        .preview;
                    PushBatchProgress {
                        worktree: WorktreeId(id.clone()),
                        phase: if preview.relation == PushRelation::UpToDate {
                            PushBatchItemPhase::SkippedObservedCurrent
                        } else {
                            PushBatchItemPhase::Pending
                        },
                        publication: None,
                    }
                })
                .collect();
            let mut value = PushBatch {
                plan,
                phase: PushBatchPhase::Ready,
                items,
                issue: None,
            };
            if next(&value).is_none() {
                value.phase = PushBatchPhase::Complete;
            }
            self.persist_push_batch(&Stored {
                version: 1,
                session: self.session.clone(),
                value: value.clone(),
            })?;
            Ok(value)
        })();
        self.options.process = previous;
        result
    }
    pub(crate) fn propose_push_batch_item(
        &mut self,
        workspace: &WorkspaceId,
        id: &str,
        entries: &BTreeMap<String, crate::workspace::StoredWorktree>,
        selection: PushPlanningSelection,
        options: &ReadOptions,
    ) -> Result<PushPublicationPlan> {
        let mut stored = self.load_push_batch(id)?;
        if stored.session != self.session
            || stored.value.plan.workspace != *workspace
            || stored.value.phase != PushBatchPhase::Ready
        {
            return Err(stale());
        }
        let index = next(&stored.value).ok_or_else(stale)?;
        if stored.value.items[index].worktree.0 != selection.worktree
            || stored.value.items[index].phase != PushBatchItemPhase::Pending
        {
            return Err(stale());
        }
        let original = stored
            .value
            .plan
            .selections
            .iter()
            .find(|s| s.selection.worktree == selection.worktree)
            .ok_or_else(stale)?;
        if original.selection.source_oid != selection.source_oid {
            return Err(stale());
        }
        let identities =
            |items: &[ChildReachabilitySelection]| -> Result<std::collections::BTreeSet<String>> {
                items.iter().map(binding_identity).collect()
            };
        if original.selection.children.len() != selection.children.len()
            || identities(&original.selection.children)? != identities(&selection.children)?
        {
            return Err(stale());
        }
        // Child success is not inferred from an old plan. A separately approved
        // observation made after the selected child's completed item is required.
        let original_bindings = original
            .selection
            .children
            .iter()
            .map(|b| Ok((binding_identity(b)?, b)))
            .collect::<Result<BTreeMap<_, _>>>()?;
        for child in &selection.children {
            let old = original_bindings
                .get(&binding_identity(child)?)
                .ok_or_else(stale)?;
            let before = self.completed_observation(
                &old.child_observation,
                workspace,
                &WorktreeId(child.child_worktree.clone()),
            )?;
            let fresh = self.completed_observation(
                &child.child_observation,
                workspace,
                &WorktreeId(child.child_worktree.clone()),
            )?;
            if before.plan.remote != fresh.plan.remote
                || before.plan.reference != fresh.plan.reference
                || before.plan.destination != fresh.plan.destination
            {
                return Err(stale());
            }
            if let Some(progress) = stored
                .value
                .items
                .iter()
                .find(|i| i.worktree.0 == child.child_worktree)
            {
                let after = if let Some(publication) = &progress.publication {
                    let value = self.load_publication_for_batch(publication)?;
                    if progress.phase != PushBatchItemPhase::Complete
                        || value.phase != PushPublicationPhase::Complete
                        || !value.published
                    {
                        return Err(stale());
                    }
                    value.finished_at_ms.ok_or_else(stale)?
                } else {
                    if progress.phase != PushBatchItemPhase::SkippedObservedCurrent {
                        return Err(stale());
                    }
                    // A skipped item made no remote change; its existing fresh
                    // observation is still checked by the parent's normal review.
                    continue;
                };
                if fresh.plan.created_at_ms <= after
                    || fresh.observed_at_ms.ok_or_else(stale)? <= after
                {
                    return Err(fail(ErrorKind::StaleReview,"Observe the selected child's destination again after its completed queue item before reviewing the parent"));
                }
            }
        }
        let hooks = original.hooks;
        let binding = PushBatchBinding {
            id: id.into(),
            digest: stored.value.plan.digest.clone(),
            worktree: WorktreeId(selection.worktree.clone()),
        };
        let plan =
            self.propose_publication(workspace, entries, selection, hooks, options, Some(binding))?;
        let fixed = &stored
            .value
            .plan
            .dependency_plan
            .items
            .iter()
            .find(|p| p.worktree == plan.check.worktree.0)
            .ok_or_else(stale)?
            .preview;
        if !same_source(fixed, &plan.check.preview) {
            self.cancel_publication(&plan.id)?;
            return Err(stale());
        }
        stored.value.items[index].phase = PushBatchItemPhase::ReviewPending;
        stored.value.items[index].publication = Some(plan.id.clone());
        if let Err(error) = self.persist_push_batch(&stored) {
            self.cancel_publication(&plan.id)?;
            return Err(error);
        }
        Ok(plan)
    }
    pub(super) fn start_push_batch_publication(
        &mut self,
        plan: &PushPublicationPlan,
    ) -> Result<()> {
        let Some(binding) = &plan.batch else {
            return Ok(());
        };
        let mut stored = self.load_push_batch(&binding.id)?;
        let index = next(&stored.value).ok_or_else(stale)?;
        let item = &mut stored.value.items[index];
        if stored.session != self.session
            || stored.value.phase != PushBatchPhase::Ready
            || stored.value.plan.workspace != plan.check.workspace
            || stored.value.plan.digest != binding.digest
            || item.worktree != binding.worktree
            || item.worktree != plan.check.worktree
            || item.phase != PushBatchItemPhase::ReviewPending
            || item.publication.as_deref() != Some(&plan.id)
        {
            return Err(stale());
        }
        item.phase = PushBatchItemPhase::Executing;
        stored.value.phase = PushBatchPhase::Executing;
        self.persist_push_batch(&stored)
    }
    pub(super) fn finish_push_batch_publication(&mut self, value: &PushPublication) -> Result<()> {
        let Some(binding) = &value.plan.batch else {
            return Ok(());
        };
        let mut stored = self.load_push_batch(&binding.id)?;
        if stored.session != self.session || stored.value.plan.digest != binding.digest {
            return Err(stale());
        }
        let item = stored
            .value
            .items
            .iter_mut()
            .find(|i| i.worktree == binding.worktree)
            .ok_or_else(stale)?;
        // Proposal failure before association carries no queue execution authority.
        if item.publication.is_none() && stored.value.phase == PushBatchPhase::Ready {
            return Ok(());
        }
        if item.publication.as_deref() != Some(&value.plan.id) {
            return Err(stale());
        }
        if value.phase == PushPublicationPhase::Complete && value.published {
            item.phase = PushBatchItemPhase::Complete;
            stored.value.phase = if next(&stored.value).is_none() {
                PushBatchPhase::Complete
            } else {
                PushBatchPhase::Ready
            };
        } else {
            item.phase = if value.may_have_published {
                PushBatchItemPhase::Unconfirmed
            } else if value.phase == PushPublicationPhase::Cancelled {
                PushBatchItemPhase::Cancelled
            } else {
                PushBatchItemPhase::Blocked
            };
            stored.value.phase = PushBatchPhase::Stopped;
            stored.value.issue = value.issue.clone();
        }
        self.persist_push_batch(&stored)
    }
}
