//! Explicit local commit batches reuse immutable M2 proposals and never replay on restart.
use super::*;
use rusqlite::{params, OptionalExtension};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BatchMode {
    ReviewAll,
    IndexOnly,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BatchSelection {
    pub worktree: String,
    pub message: String,
    pub mode: BatchMode,
    pub excluded: Vec<NativePath>,
    pub hooks: HookPolicy,
    pub signing: SigningPreference,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BatchPhase {
    Proposed,
    Executing,
    Complete,
    Blocked,
    PartialFailure,
    Cancelled,
    NeedsReauthorization,
    Interrupted,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BatchItem {
    pub worktree: WorktreeId,
    pub path: NativePath,
    pub dependencies: Vec<WorktreeId>,
    pub operation: Option<String>,
    pub issue: Option<Issue>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BatchRecord {
    pub version: u32,
    pub id: String,
    pub workspace: WorkspaceId,
    pub created_at_ms: u64,
    pub phase: BatchPhase,
    pub issue: Option<Issue>,
    pub items: Vec<BatchItem>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BatchView {
    pub record: BatchRecord,
    pub outcomes: Vec<Outcome>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BatchPage {
    pub items: Vec<BatchView>,
    pub next: Option<usize>,
}

fn path_key(path: &NativePath) -> Result<Vec<u8>> {
    Ok(path.relative()?.as_os_str().as_encoded_bytes().to_vec())
}
fn selections(
    entry: &StoredWorktree,
    selection: &BatchSelection,
    nested: &[PathBuf],
    relationships: &[crate::discovery::Relationship],
) -> Result<Vec<SelectedPath>> {
    let status = entry
        .status
        .as_ref()
        .ok_or_else(|| invalid("Status unavailable"))?;
    if status.changes.iter().any(|c| c.conflict) {
        return Err(Error::with_kind(
            ErrorKind::UnsupportedRepositoryState,
            "Resolve conflicts before batch preparation",
        ));
    }
    let root = entry.metadata.root.to_path()?;
    let mut paths = BTreeMap::new();
    for change in &status.changes {
        let relative = change.path.to_path()?;
        // Git reports an untracked nested repository as a directory; never include its contents.
        if change.index == '?' && nested.iter().any(|p| root.join(&relative).starts_with(p)) {
            continue;
        }
        let source = match selection.mode {
            BatchMode::IndexOnly if !['.', '?', '!'].contains(&change.index) => {
                ContentSource::Index
            }
            BatchMode::ReviewAll if change.worktree != '.' || change.index == '?' => {
                ContentSource::WorkingTree
            }
            BatchMode::ReviewAll if !['.', '?', '!'].contains(&change.index) => {
                ContentSource::Index
            }
            _ => continue,
        };
        paths.insert(
            path_key(&change.path)?,
            SelectedPath {
                path: change.path.clone(),
                source,
            },
        );
        if let Some(original) = &change.original_path {
            paths.insert(
                path_key(original)?,
                SelectedPath {
                    path: original.clone(),
                    source,
                },
            );
        }
    }
    if matches!(selection.mode, BatchMode::ReviewAll) {
        // Restricted status deliberately avoids child worktrees. Complete discovery
        // has inspected their identities/HEADs separately; include changed gitlinks
        // in the parent's new exact review, never child file contents.
        for edge in relationships.iter().filter(|edge| {
            edge.parent == entry.metadata.root
                && matches!(edge.kind, crate::discovery::RelationshipKind::Submodule)
                && edge.gitlink_matches_head == Some(false)
                && edge.child_head.is_some()
        }) {
            let child = edge.child.to_path()?;
            let relative = child
                .strip_prefix(&root)
                .map_err(|_| invalid("Submodule relationship is outside the parent"))?;
            let path = NativePath::from_path(relative);
            paths.insert(
                path_key(&path)?,
                SelectedPath {
                    path,
                    source: ContentSource::WorkingTree,
                },
            );
        }
    }
    for path in &selection.excluded {
        if paths.remove(&path_key(path)?).is_none() {
            return Err(invalid(
                "An excluded path is no longer part of the selected changes; review again",
            ));
        }
    }
    if paths.is_empty() {
        return Err(invalid("No eligible paths remain in this worktree"));
    }
    Ok(paths.into_values().collect())
}
impl Application {
    fn save_batch(&mut self, record: &BatchRecord) -> Result<()> {
        let payload = serde_json::to_string(record)?;
        if payload.len() > 4 * 1024 * 1024 {
            return Err(Error::with_kind(
                ErrorKind::OutputLimit,
                "Batch record exceeds 4 MiB",
            ));
        }
        self.writes.workspace.connection().execute(
            "INSERT OR REPLACE INTO state(key,value) VALUES(?1,?2)",
            params![format!("batch:{}", record.id), payload],
        )?;
        Ok(())
    }
    fn batch(&mut self, workspace: &WorkspaceId, id: &str) -> Result<BatchRecord> {
        Uuid::parse_str(id).map_err(|_| invalid("Invalid batch ID"))?;
        let payload: Option<String> = self
            .writes
            .workspace
            .connection()
            .query_row(
                "SELECT value FROM state WHERE key=?1",
                [format!("batch:{id}")],
                |r| r.get(0),
            )
            .optional()?;
        let record: BatchRecord =
            serde_json::from_str(&payload.ok_or_else(|| invalid("Unknown batch"))?)?;
        if record.version != 1 {
            return Err(Error::with_kind(
                ErrorKind::NewerSchema,
                "Unsupported batch version",
            ));
        }
        if record.workspace != *workspace {
            return Err(invalid("Batch belongs to another workspace"));
        }
        Ok(record)
    }
    pub(super) fn batch_view(&mut self, record: BatchRecord) -> Result<BatchView> {
        let outcomes = record
            .items
            .iter()
            .filter_map(|i| i.operation.as_ref())
            .map(|id| self.writes.outcome(id))
            .collect::<Result<_>>()?;
        Ok(BatchView { record, outcomes })
    }
    pub(super) fn batches(&mut self, workspace: &str, offset: usize) -> Result<BatchPage> {
        let ws = self.active(workspace)?;
        if offset > 1_000_000 {
            return Err(invalid("Batch offset limit exceeded"));
        }
        let (records, mut more) = {
            let mut statement=self.writes.workspace.connection().prepare("SELECT value FROM state WHERE key LIKE 'batch:%' AND json_extract(value,'$.workspace')=?1 ORDER BY json_extract(value,'$.created_at_ms') DESC,key LIMIT 51 OFFSET ?2")?;
            let rows = statement
                .query_map(params![workspace, offset as i64], |r| r.get::<_, String>(0))?;
            let mut records = Vec::new();
            let mut size = 0;
            let mut more = false;
            for raw in rows {
                let raw = raw?;
                if raw.len() > 4 * 1024 * 1024 {
                    return Err(Error::with_kind(
                        ErrorKind::OutputLimit,
                        "Stored batch exceeds size limit",
                    ));
                }
                if records.len() == 50 || size + raw.len() > 4 * 1024 * 1024 {
                    more = true;
                    break;
                }
                size += raw.len();
                records.push(raw);
            }
            (records, more)
        };
        let mut items = Vec::new();
        let mut size = 0;
        for raw in records {
            let mut record: BatchRecord = serde_json::from_str(&raw)?;
            if record.version != 1 || record.workspace != ws {
                return Err(invalid("Invalid stored batch"));
            }
            // The serialized owner cannot observe an executing batch except after interruption.
            let live_review = self.pending.as_ref().is_some_and(|pending| {
                matches!(&pending.action, review::Action::Batch(batch) if batch.id == record.id)
                    && self.review(&pending.id).is_ok()
            });
            if record.phase == BatchPhase::Executing
                || (record.phase == BatchPhase::Proposed && !live_review)
            {
                record.phase = BatchPhase::Interrupted;
            }
            let view = self.batch_view(record)?;
            let length = serde_json::to_vec(&view)?.len();
            if size + length > 8 * 1024 * 1024 {
                if items.is_empty() {
                    return Err(Error::with_kind(
                        ErrorKind::OutputLimit,
                        "Batch result exceeds display budget",
                    ));
                }
                more = true;
                break;
            }
            size += length;
            items.push(view);
        }
        let next = more.then_some(offset + items.len());
        Ok(BatchPage { items, next })
    }
    pub(super) fn review_batch(
        &mut self,
        workspace: &str,
        selections_requested: Vec<BatchSelection>,
    ) -> Result<ReviewDisplay> {
        let ws = self.active(workspace)?;
        if selections_requested.is_empty() || selections_requested.len() > 20 {
            return Err(invalid("Batch review requires 1..20 explicit worktrees"));
        }
        let mut selected = BTreeMap::new();
        for selection in selections_requested {
            if selection.message.len() > 512 * 1024
                || selection.excluded.len() > 10_000
                || selected
                    .insert(selection.worktree.clone(), selection)
                    .is_some()
            {
                return Err(invalid("Duplicate worktree or oversized batch selection"));
            }
        }
        self.abandon_review()?;
        let scan = self
            .writes
            .workspace
            .scan_with_options(&ws, &self.read_options)?;
        if !scan.scan.complete {
            return Err(Error::with_kind(
                ErrorKind::Unavailable,
                "Complete discovery is required for batch dependency planning",
            ));
        }
        self.restart_monitor()?;
        let entries: BTreeMap<_, _> = scan
            .entries
            .into_iter()
            .filter(|e| selected.contains_key(&e.id.0))
            .map(|e| (e.id.0.clone(), e))
            .collect();
        if entries.len() != selected.len() {
            return Err(invalid("Unknown selected worktree"));
        }
        let mut dependencies: BTreeMap<String, BTreeSet<String>> = selected
            .keys()
            .map(|id| (id.clone(), BTreeSet::new()))
            .collect();
        for edge in &scan.scan.relationships {
            if !matches!(edge.kind, crate::discovery::RelationshipKind::Submodule) {
                continue;
            }
            let parent = entries.values().find(|e| e.metadata.root == edge.parent);
            let child = entries.values().find(|e| e.metadata.root == edge.child);
            if let (Some(parent), Some(child)) = (parent, child) {
                dependencies
                    .get_mut(&parent.id.0)
                    .ok_or_else(stale)?
                    .insert(child.id.0.clone());
            }
        }
        let selected_dependencies = dependencies.clone();
        let mut order = Vec::new();
        while !dependencies.is_empty() {
            let ready = dependencies
                .iter()
                .find(|(_, edges)| edges.is_empty())
                .map(|(id, _)| id.clone())
                .ok_or_else(|| invalid("Cyclic selected submodule dependencies"))?;
            dependencies.remove(&ready);
            for edges in dependencies.values_mut() {
                edges.remove(&ready);
            }
            order.push(ready);
        }
        let mut record = BatchRecord {
            version: 1,
            id: Uuid::new_v4().to_string(),
            workspace: ws.clone(),
            created_at_ms: now_ms(),
            phase: BatchPhase::Proposed,
            issue: None,
            items: Vec::new(),
        };
        for id in order {
            if self.read_options.cancellation.is_cancelled() {
                self.cancel_batch_record(&mut record)?;
                return Err(Error::with_kind(
                    ErrorKind::Cancelled,
                    "Batch preparation cancelled",
                ));
            }
            let entry = entries.get(&id).ok_or_else(stale)?;
            let selection = selected.remove(&id).ok_or_else(stale)?;
            let nested = scan
                .scan
                .repositories
                .iter()
                .filter(|r| r.metadata.root != entry.metadata.root)
                .map(|r| r.metadata.root.to_path())
                .collect::<Result<Vec<_>>>()?
                .into_iter()
                .filter(|path| {
                    entry
                        .metadata
                        .root
                        .to_path()
                        .is_ok_and(|root| path.starts_with(root))
                })
                .collect::<Vec<_>>();
            let result = (|| {
                let current = self.entry(workspace, &id)?;
                let selected = selections(&current, &selection, &nested, &scan.scan.relationships)?;
                self.writes.application_propose(
                    &ws,
                    &current.id,
                    WriteRequest::Commit {
                        selected,
                        message: selection.message,
                        hooks: selection.hooks,
                        signing: selection.signing,
                    },
                    self.read_options.cancellation.clone(),
                )
            })();
            let (operation, issue) = match result {
                Ok(p) => (Some(p.operation_id), None),
                Err(e) => (None, Some(e.into())),
            };
            record.items.push(BatchItem {
                worktree: entry.id.clone(),
                path: entry.metadata.root.clone(),
                dependencies: selected_dependencies[&id]
                    .iter()
                    .cloned()
                    .map(WorktreeId)
                    .collect(),
                operation,
                issue,
            });
        }
        self.save_batch(&record)?;
        match self.set_review(review::Action::Batch(record.clone())) {
            Ok(display) => Ok(display),
            Err(error) => {
                self.cancel_batch_record(&mut record)?;
                Err(error)
            }
        }
    }
    pub(super) fn cancel_batch_record(&mut self, record: &mut BatchRecord) -> Result<()> {
        for item in &record.items {
            if let Some(id) = &item.operation {
                self.writes.application_cancel_proposal(id)?;
            }
        }
        record.phase = BatchPhase::Cancelled;
        self.save_batch(record)
    }
    pub(super) fn describe_batch(
        &mut self,
        record: &BatchRecord,
    ) -> Result<(String, String, String)> {
        let blocked = record.items.iter().filter(|i| i.issue.is_some()).count();
        let summary=format!("BULK COMMIT\nBatch: {}\n{} explicitly selected worktrees; {} blocked.\nSequential execution stops on first failure. Successful commits are preserved.\nChanged parent gitlinks require fresh review; this batch is not atomic.\n{}",record.id,record.items.len(),blocked,if blocked>0{"No operations execute while a selected item is blocked."}else{"Review every exact per-worktree plan below."});
        let mut content = summary.clone();
        for item in &record.items {
            content.push_str(&format!("\n\nWorktree: {}\n", item.path.display()));
            if let Some(issue) = &item.issue {
                content.push_str(&format!("BLOCKED {:?}: {}\n", issue.kind, issue.message));
            }
            if let Some(id) = &item.operation {
                let p = self.writes.application_proposal(id)?;
                content.push_str(&self.describe_write(&p)?.2);
            }
            if content.len() > 32 * 1024 * 1024 {
                return Err(invalid("Batch review exceeds display budget"));
            }
        }
        Ok(("Review bulk commit".into(), summary, content))
    }
    pub(super) fn execute_batch(
        &mut self,
        mut record: BatchRecord,
        cancellation: inspection::Cancellation,
    ) -> Result<serde_json::Value> {
        self.active(&record.workspace.0)?;
        let stored = self.batch(&record.workspace, &record.id)?;
        if stored.phase != BatchPhase::Proposed
            || serde_json::to_vec(&stored)? != serde_json::to_vec(&record)?
        {
            return Err(stale());
        }
        if record.items.iter().any(|i| i.issue.is_some()) {
            self.cancel_batch_record(&mut record)?;
            record.phase = BatchPhase::Blocked;
            self.save_batch(&record)?;
            return Ok(serde_json::to_value(self.batch_view(record)?)?);
        }
        record.phase = BatchPhase::Executing;
        self.save_batch(&record)?;
        for item in &record.items {
            if cancellation.is_cancelled() {
                record.phase = BatchPhase::Cancelled;
                break;
            }
            let operation = item.operation.as_deref().ok_or_else(stale)?;
            let result = (|| {
                self.writes
                    .application_operation_scope(&record.workspace, operation)?;
                // A child commit changes the parent's candidate gitlink. The old parent
                // proposal cannot authorize content prepared after that publication.
                for dependency in &item.dependencies {
                    let child = record
                        .items
                        .iter()
                        .find(|i| &i.worktree == dependency)
                        .ok_or_else(stale)?;
                    let child_operation = child.operation.as_deref().ok_or_else(stale)?;
                    if self.writes.outcome(child_operation)?.published {
                        return Err(Error::with_kind(ErrorKind::StaleReview,
                            "Selected submodule committed; review the parent again with its new gitlink"));
                    }
                }
                self.entry(&record.workspace.0, &item.worktree.0)?;
                let proposal = self.writes.application_proposal(operation)?;
                let approval = self.writes.approve(&proposal)?;
                self.writes
                    .execute_with_cancellation(&approval, cancellation.clone())
            })();
            match result {
                Ok(outcome) if outcome.phase == Phase::Complete => {}
                Ok(outcome) => {
                    record.phase = match outcome.phase {
                        Phase::NeedsReauthorization => BatchPhase::NeedsReauthorization,
                        Phase::CancelledBeforePublication => BatchPhase::Cancelled,
                        _ => BatchPhase::PartialFailure,
                    };
                    break;
                }
                Err(error) => {
                    record.phase = match error.kind {
                        ErrorKind::StaleReview | ErrorKind::UntrustedRepository => {
                            BatchPhase::NeedsReauthorization
                        }
                        ErrorKind::Cancelled => BatchPhase::Cancelled,
                        _ => BatchPhase::PartialFailure,
                    };
                    record.issue = Some(error.into());
                    break;
                }
            }
        }
        if record.phase == BatchPhase::Executing {
            record.phase = BatchPhase::Complete;
        }
        // Proposed successors never acquire authority after a stop. Their outcomes remain durable.
        for item in &record.items {
            if let Some(id) = &item.operation {
                self.writes.application_cancel_proposal(id)?;
            }
        }
        self.save_batch(&record)?;
        Ok(serde_json::to_value(self.batch_view(record)?)?)
    }
}
