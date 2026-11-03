//! Presentation-independent application/session boundary. No native UI or transport APIs.
mod audit;
mod authentication;
pub use audit::{
    OperationAuditEntry, OperationAuditKind, OperationAuditPage, OperationAuditQuery,
    OperationAuditRecord, OperationAuditResult,
};
mod bulk;
mod bulk_state;
pub use bulk_state::{BatchConfiguration, BulkComposerState};
mod exploration;
pub use bulk::{
    BatchItem, BatchMode, BatchPage, BatchPhase, BatchRecord, BatchSelection, BatchView,
};
mod monitor;
mod review;
mod state;
use crate::{
    diff, domain::*, history, inspection, workspace::*, writes::*, Error, ErrorKind, Result,
};
pub use exploration::{
    ActivityEntry, ActivityReport, AuditEntry, AuditPage, AuditPhase, AuditQuery,
};
pub use monitor::{Monitor, MonitorStatus, MonitoringOptions};
pub use review::{Confirmation, ReviewDisplay};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
pub use state::{DesktopState, RepositoryTab, WindowPlacement};
use std::{
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
use uuid::Uuid;

fn invalid(message: &str) -> Error {
    Error::with_kind(ErrorKind::InvalidRequest, message)
}
fn stale() -> Error {
    Error::with_kind(
        ErrorKind::StaleReview,
        "Application context or review changed",
    )
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Request {
    AuthenticationSettings {},
    AuthenticationChoices {},
    ReviewAuthentication {
        change: crate::authentication::Change,
    },
    Graph {
        workspace: String,
        worktree: String,
        scope: history::HistoryScope,
        cursor: Option<crate::graph::Cursor>,
    },
    Activity {
        workspace: String,
        worktree: Option<String>,
        since: u64,
        until: u64,
        reference: Option<String>,
        author: Option<String>,
    },
    OperationAudit {
        workspace: String,
        query: OperationAuditQuery,
    },
    Audit {
        workspace: String,
        query: AuditQuery,
    },
    ReviewBatch {
        workspace: String,
        selections: Vec<BatchSelection>,
    },
    Batches {
        workspace: String,
        offset: usize,
    },
    LoadBulkComposer {
        workspace: String,
    },
    SaveBulkComposer {
        workspace: String,
        state: BulkComposerState,
    },
    Remotes {
        workspace: String,
        worktree: String,
    },
    ReviewRemoteObservation {
        workspace: String,
        worktree: String,
        remote: String,
        reference: String,
    },
    RemoteObservations {
        workspace: String,
        worktree: Option<String>,
        offset: usize,
    },
    ReviewUpstreamSetup {
        workspace: String,
        worktree: String,
        observation: String,
    },
    ReviewUpstreamRecovery {
        workspace: String,
        operation: String,
    },
    PushPreview {
        workspace: String,
        worktree: String,
        observation: String,
    },
    ReviewFetchCapture {
        workspace: String,
        worktree: String,
        observation: String,
    },
    ReviewFetchImport {
        workspace: String,
        worktree: String,
        capture: String,
    },
    FetchImports {
        workspace: String,
        offset: usize,
    },
    ReviewFetchImportRecovery {
        workspace: String,
        operation: String,
    },
    FetchCaptures {
        workspace: String,
        worktree: Option<String>,
        offset: usize,
    },
    Workspaces {},
    Create {
        name: String,
        picker: String,
    },
    Update {
        workspace: String,
        name: Option<String>,
        picker: Option<String>,
        exclusions: Option<crate::discovery::DiscoveryOptions>,
    },
    Remove {
        workspace: String,
    },
    Activate {
        workspace: String,
    },
    Inventory {
        workspace: String,
    },
    Scan {
        workspace: String,
    },
    Refresh {
        workspace: String,
        worktree: String,
    },
    History {
        workspace: String,
        worktree: String,
        cursor: Option<history::Cursor>,
    },
    Commit {
        workspace: String,
        worktree: String,
        oid: String,
    },
    Diff {
        workspace: String,
        worktree: String,
        mode: diff::DiffMode,
        path: Option<NativePath>,
    },
    Preview {
        workspace: String,
        worktree: String,
        path: NativePath,
    },
    LoadState {
        workspace: String,
    },
    SaveState {
        workspace: String,
        state: DesktopState,
    },
    TrustStatus {
        workspace: String,
    },
    ReviewTrust {
        workspace: String,
        worktrees: Vec<String>,
    },
    Revoke {
        workspace: String,
        worktree: String,
    },
    HookPolicy {
        workspace: String,
        worktree: String,
        policy: HookPolicy,
    },
    Propose {
        workspace: String,
        worktree: String,
        request: WriteRequest,
    },
    Operations {
        workspace: String,
        offset: usize,
    },
    ReviewRecovery {
        workspace: String,
        operation: String,
    },
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkspaceList {
    pub workspaces: Vec<Workspace>,
    pub active: Option<String>,
    pub recovery: Vec<RecoveryNotice>,
    pub fetch_import_recovery: Vec<FetchImportNotice>,
    pub upstream_recovery: Vec<RemoteRecoveryNotice>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct InventoryView {
    pub entries: Vec<StoredWorktree>,
    pub monitor: MonitorStatus,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TrustState {
    pub worktree: String,
    pub grant: String,
    pub validity: String,
    pub eligible: bool,
    pub reason: Option<String>,
    pub policy: Option<HookPolicy>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct OperationPage {
    pub items: Vec<Outcome>,
    pub next: Option<usize>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum Data {
    AuthenticationSettings(crate::authentication::Settings),
    AuthenticationChoices(crate::authentication::Choices),
    Graph(crate::graph::Page),
    Activity(exploration::ActivityReport),
    OperationAudit(OperationAuditPage),
    Audit(exploration::AuditPage),
    Batches(BatchPage),
    BulkComposer(BulkComposerState),
    Remotes(crate::remote::Inspection),
    RemoteObservations(crate::remote::observation::ObservationPage),
    FetchCaptures(crate::remote::fetch::FetchCapturePage),
    PushPreview(crate::remote::push::PushPreview),
    FetchImports(crate::remote::fetch::FetchImportPage),
    Workspaces(WorkspaceList),
    Inventory(InventoryView),
    Refreshed(Box<StoredWorktree>),
    History(history::HistoryPage),
    Commit(history::Commit),
    Diff(diff::Diff),
    Preview(diff::Preview),
    State(DesktopState),
    Trust(Vec<TrustState>),
    Review(ReviewDisplay),
    Operations(OperationPage),
    Done,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Response {
    pub version: u32,
    pub epoch: u64,
    pub data: Data,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RemoteRecoveryNotice {
    pub operation_id: String,
    pub workspace_id: String,
    pub worktree_id: String,
}
pub type FetchImportNotice = RemoteRecoveryNotice;
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RecoveryNotice {
    pub operation_id: String,
    pub workspace_id: String,
    pub worktree_id: String,
    pub phase: Phase,
    pub published: bool,
    pub reason: Option<String>,
}
/// One owner of app storage and M2 approval sessions. Adapter code cannot access its DB.
pub struct Application {
    writes: ControlledWrites,
    epoch: u64,
    read_options: inspection::ReadOptions,
    monitor: Monitor,
    pending: Option<review::Pending>,
    pickers: std::collections::BTreeMap<String, (PathBuf, Instant)>,
}
impl Application {
    pub fn open(directory: &Path, options: Options) -> Result<Self> {
        Self::open_with_monitoring(directory, options, MonitoringOptions::default())
    }
    pub fn open_with_monitoring(
        directory: &Path,
        options: Options,
        timing: MonitoringOptions,
    ) -> Result<Self> {
        let mut writes = ControlledWrites::open(directory, options)?;
        state::migrate(&mut writes)?;
        let mut app = Self {
            writes,
            epoch: 1,
            read_options: inspection::ReadOptions::default(),
            monitor: Monitor::with_options(timing)?,
            pending: None,
            pickers: Default::default(),
        };
        app.restart_monitor()?;
        Ok(app)
    }
    /// Native adapters call this only with the result of a native directory picker.
    pub fn register_directory(&mut self, path: &Path) -> Result<String> {
        let path = std::fs::canonicalize(path)?;
        if !path.is_dir() {
            return Err(invalid("Workspace root must be a directory"));
        }
        self.pickers
            .retain(|_, (_, at)| at.elapsed() < Duration::from_secs(300));
        if self.pickers.len() >= 8 {
            return Err(invalid("Too many pending directory selections"));
        }
        let id = Uuid::new_v4().to_string();
        self.pickers.insert(id.clone(), (path, Instant::now()));
        Ok(id)
    }
    fn directory(&mut self, id: &str) -> Result<PathBuf> {
        let (path, at) = self.pickers.remove(id).ok_or_else(stale)?;
        if at.elapsed() > Duration::from_secs(300) {
            return Err(stale());
        }
        Ok(path)
    }
    fn active(&self, id: &str) -> Result<WorkspaceId> {
        let id = WorkspaceId(id.to_owned());
        if self.writes.workspace.active()?.as_ref() != Some(&id) {
            return Err(stale());
        }
        Ok(id)
    }
    fn entry(&self, workspace: &str, worktree: &str) -> Result<StoredWorktree> {
        let id = self.active(workspace)?;
        let entry = self
            .writes
            .workspace
            .inventory(&id)?
            .into_iter()
            .find(|e| e.id.0 == worktree)
            .ok_or_else(|| invalid("Unknown worktree"))?;
        if !matches!(entry.availability, Availability::Available) {
            return Err(Error::with_kind(
                ErrorKind::Unavailable,
                "Worktree is unavailable",
            ));
        }
        let definition = self
            .writes
            .workspace
            .list()
            .iter()
            .find(|w| w.id == id)
            .ok_or_else(stale)?;
        if !entry
            .metadata
            .root
            .to_path()?
            .starts_with(definition.root.to_path()?)
        {
            return Err(stale());
        }
        let current = inspection::repository_metadata_with_options(
            &entry.metadata.root.to_path()?,
            &self.read_options,
        )?;
        if current.root_binding != entry.metadata.root_binding
            || current.git_binding != entry.metadata.git_binding
            || current.common_binding != entry.metadata.common_binding
        {
            return Err(stale());
        }
        Ok(entry)
    }
    fn abandon_review(&mut self) -> Result<()> {
        if let Some(pending) = self.pending.take() {
            match pending.action {
                review::Action::Write(proposal) => self
                    .writes
                    .application_cancel_proposal(&proposal.operation_id)?,
                review::Action::Batch(mut record) => self.cancel_batch_record(&mut record)?,
                review::Action::UpstreamSetup(plan) => {
                    self.writes.cancel_upstream(&plan.transport.id)?
                }
                review::Action::FetchImport(plan) => {
                    self.writes.cancel_import(&plan.transport.id)?
                }
                review::Action::FetchCapture(plan) => {
                    self.writes.cancel_capture(&plan.transport.id)?
                }
                review::Action::RemoteObservation(plan) => {
                    self.writes.cancel_observation(&plan.id)?
                }
                _ => {}
            }
        }
        self.writes.cancel_application_reviews();
        Ok(())
    }
    fn invalidate(&mut self) -> Result<()> {
        self.epoch = self.epoch.wrapping_add(1);
        self.pickers.clear();
        self.abandon_review()
    }
    fn restart_monitor(&mut self) -> Result<()> {
        self.monitor.stop();
        if let Some(id) = self.writes.workspace.active()? {
            if let Some(w) = self.writes.workspace.list().iter().find(|w| w.id == id) {
                self.monitor
                    .activate(w, &self.writes.workspace.inventory(&id)?, self.epoch)?;
            }
        }
        Ok(())
    }
    pub fn reset_presentation(&mut self) -> Result<()> {
        self.invalidate()?;
        self.restart_monitor()
    }
    #[cfg(feature = "write-test-support")]
    pub fn set_import_test_fault(
        &mut self,
        phase: crate::remote::fetch::FetchImportPhase,
        kind: TestFaultKind,
    ) {
        self.writes.set_import_fault(phase, kind);
    }
    #[cfg(feature = "write-test-support")]
    pub fn set_upstream_test_fault(
        &mut self,
        phase: crate::remote::upstream::UpstreamPhase,
        kind: TestFaultKind,
    ) {
        self.writes.set_upstream_fault(phase, kind);
    }
    pub fn epoch(&self) -> u64 {
        self.epoch
    }
    pub fn shutdown(&mut self) -> Result<()> {
        // Always stop observers even if an app-data write fails during cancellation.
        self.monitor.stop();
        self.pickers.clear();
        self.invalidate()
    }
    fn require_resolved(&mut self, workspace: &str) -> Result<()> {
        if self.writes.application_has_pending(workspace)? {
            return Err(Error::with_kind(
                ErrorKind::Busy,
                "Resolve interrupted operations before removing or changing workspace scope",
            ));
        }
        Ok(())
    }
    pub fn request_with_cancellation(
        &mut self,
        epoch: u64,
        request: Request,
        cancellation: inspection::Cancellation,
    ) -> Result<Response> {
        let previous = std::mem::replace(&mut self.read_options.cancellation, cancellation);
        let result = self.request(epoch, request);
        self.read_options.cancellation = previous;
        result
    }
    pub fn request(&mut self, epoch: u64, request: Request) -> Result<Response> {
        if self.read_options.cancellation.is_cancelled() {
            return Err(Error::with_kind(
                ErrorKind::Cancelled,
                "Application request cancelled before execution",
            ));
        }
        if epoch != self.epoch && !matches!(request, Request::Workspaces {}) {
            return Err(stale());
        }
        match &request {
            Request::Create { name, .. }
            | Request::Update {
                name: Some(name), ..
            } if name.trim().is_empty()
                || name.len() > 512
                || name.chars().any(char::is_control) =>
            {
                return Err(invalid(
                    "Workspace names must be nonempty, control-free, and at most 512 UTF-8 bytes",
                ))
            }
            Request::Update {
                exclusions: Some(options),
                ..
            } if options.excluded_names.len() + options.excluded_patterns.len() > 1000
                || options
                    .excluded_names
                    .iter()
                    .chain(&options.excluded_patterns)
                    .any(|p| p.len() > 1024) =>
            {
                return Err(invalid("Exclusion policy exceeds desktop limits"))
            }
            _ => {}
        }
        let data = match request {
            Request::Graph {
                workspace,
                worktree,
                scope,
                cursor,
            } => {
                let e = self.entry(&workspace, &worktree)?;
                let mut options = self.read_options.clone();
                options.timeout = Duration::from_secs(30);
                Data::Graph(crate::graph::page(
                    &e.metadata.root.to_path()?,
                    scope,
                    100,
                    cursor.as_ref(),
                    &options,
                )?)
            }
            Request::Activity {
                workspace,
                worktree,
                since,
                until,
                reference,
                author,
            } => Data::Activity(self.activity(
                &workspace,
                worktree.as_deref(),
                since,
                until,
                reference,
                author,
            )?),
            Request::OperationAudit { workspace, query } => {
                Data::OperationAudit(self.operation_audit(&workspace, query)?)
            }
            Request::Audit { workspace, query } => {
                let id = self.active(&workspace)?;
                Data::Audit(self.writes.application_audit(&id, query)?)
            }
            Request::ReviewBatch {
                workspace,
                selections,
            } => Data::Review(self.review_batch(&workspace, selections)?),
            Request::Batches { workspace, offset } => {
                Data::Batches(self.batches(&workspace, offset)?)
            }
            Request::Workspaces {} => Data::Workspaces(WorkspaceList {
                workspaces: self.writes.workspace.list().to_vec(),
                active: self.writes.workspace.active()?.map(|id| id.0),
                recovery: self.writes.application_pending()?,
                fetch_import_recovery: self.writes.import_notices()?,
                upstream_recovery: self.writes.upstream_notices()?,
            }),
            Request::Create { name, picker } => {
                let path = self.directory(&picker)?;
                let w = self.writes.workspace.create(&name, &path)?;
                self.writes.workspace.activate(&w.id)?;
                self.invalidate()?;
                self.writes
                    .workspace
                    .scan_with_options(&w.id, &self.read_options)?;
                self.restart_monitor()?;
                Data::Done
            }
            Request::Update {
                workspace,
                name,
                picker,
                exclusions,
            } => {
                let id = self.active(&workspace)?;
                if picker.is_some() || exclusions.is_some() {
                    self.require_resolved(&workspace)?;
                }
                let path = picker.map(|p| self.directory(&p)).transpose()?;
                self.writes
                    .workspace
                    .update(&id, name.as_deref(), path.as_deref(), exclusions)?;
                self.invalidate()?;
                self.writes
                    .workspace
                    .scan_with_options(&id, &self.read_options)?;
                self.restart_monitor()?;
                Data::Done
            }
            Request::Remove { workspace } => {
                self.require_resolved(&workspace)?;
                self.writes.workspace.remove(&WorkspaceId(workspace))?;
                self.invalidate()?;
                self.restart_monitor()?;
                Data::Done
            }
            Request::Activate { workspace } => {
                self.writes
                    .workspace
                    .activate(&WorkspaceId(workspace.clone()))?;
                self.invalidate()?;
                self.restart_monitor()?;
                Data::Done
            }
            Request::Inventory { workspace } => {
                let id = self.active(&workspace)?;
                Data::Inventory(InventoryView {
                    entries: self.writes.workspace.inventory(&id)?,
                    monitor: self.monitor.status(),
                })
            }
            Request::Scan { workspace } => {
                let id = self.active(&workspace)?;
                self.writes
                    .workspace
                    .scan_with_options(&id, &self.read_options)?;
                self.restart_monitor()?;
                Data::Inventory(InventoryView {
                    entries: self.writes.workspace.inventory(&id)?,
                    monitor: self.monitor.status(),
                })
            }
            Request::Refresh {
                workspace,
                worktree,
            } => {
                let id = self.active(&workspace)?;
                Data::Refreshed(Box::new(self.writes.workspace.refresh_with_options(
                    &id,
                    &WorktreeId(worktree),
                    &self.read_options,
                )?))
            }
            Request::AuthenticationChoices {} => {
                Data::AuthenticationChoices(crate::authentication::choices(&self.read_options)?)
            }
            Request::AuthenticationSettings {} => {
                Data::AuthenticationSettings(self.authentication_settings()?)
            }
            Request::ReviewAuthentication { change } => {
                Data::Review(self.review_authentication(change)?)
            }
            Request::ReviewRemoteObservation {
                workspace,
                worktree,
                remote,
                reference,
            } => {
                let entry = self.entry(&workspace, &worktree)?;
                let id = self.active(&workspace)?;
                self.abandon_review()?;
                let plan = self.writes.propose_observation(
                    &id,
                    &entry,
                    &remote,
                    &reference,
                    &self.read_options,
                )?;
                match self.set_review(review::Action::RemoteObservation(Box::new(plan.clone()))) {
                    Ok(display) => Data::Review(display),
                    Err(error) => {
                        self.writes.cancel_observation(&plan.id)?;
                        return Err(error);
                    }
                }
            }
            Request::ReviewUpstreamSetup {
                workspace,
                worktree,
                observation,
            } => {
                let entry = self.entry(&workspace, &worktree)?;
                let id = self.active(&workspace)?;
                self.abandon_review()?;
                let plan =
                    self.writes
                        .propose_upstream(&id, &entry, &observation, &self.read_options)?;
                match self.set_review(review::Action::UpstreamSetup(Box::new(plan.clone()))) {
                    Ok(display) => Data::Review(display),
                    Err(error) => {
                        self.writes.cancel_upstream(&plan.transport.id)?;
                        return Err(error);
                    }
                }
            }
            Request::ReviewUpstreamRecovery {
                workspace,
                operation,
            } => {
                let id = self.active(&workspace)?;
                self.abandon_review()?;
                let review = self.writes.inspect_upstream_recovery(&id, &operation)?;
                self.entry(&workspace, &review.operation.plan.transport.worktree.0)?;
                Data::Review(self.set_review(review::Action::UpstreamRecovery(Box::new(review)))?)
            }
            Request::PushPreview {
                workspace,
                worktree,
                observation,
            } => {
                let entry = self.entry(&workspace, &worktree)?;
                let id = self.active(&workspace)?;
                let mut options = self.read_options.clone();
                options.timeout = Duration::from_secs(30);
                Data::PushPreview(
                    self.writes
                        .push_preview(&id, &entry, &observation, &options)?,
                )
            }
            Request::ReviewFetchImport {
                workspace,
                worktree,
                capture,
            } => {
                let entry = self.entry(&workspace, &worktree)?;
                let id = self.active(&workspace)?;
                self.abandon_review()?;
                let plan = self
                    .writes
                    .propose_import(&id, &entry, &capture, &self.read_options)?;
                match self.set_review(review::Action::FetchImport(Box::new(plan.clone()))) {
                    Ok(display) => Data::Review(display),
                    Err(error) => {
                        self.writes.cancel_import(&plan.transport.id)?;
                        return Err(error);
                    }
                }
            }
            Request::FetchImports { workspace, offset } => {
                let id = self.active(&workspace)?;
                let mut page = self.writes.imports(&id, offset)?;
                for item in &mut page.items {
                    if item.phase==crate::remote::fetch::FetchImportPhase::Proposed && !self.pending.as_ref().is_some_and(|p|matches!(&p.action,review::Action::FetchImport(plan) if plan.transport.id==item.plan.transport.id) && self.review(&p.id).is_ok()) {item.phase=crate::remote::fetch::FetchImportPhase::Interrupted;}
                }
                Data::FetchImports(page)
            }
            Request::ReviewFetchImportRecovery {
                workspace,
                operation,
            } => {
                let id = self.active(&workspace)?;
                self.abandon_review()?;
                let review = self.writes.inspect_import_recovery(&id, &operation)?;
                self.entry(&workspace, &review.operation.plan.transport.worktree.0)?;
                Data::Review(
                    self.set_review(review::Action::FetchImportRecovery(Box::new(review)))?,
                )
            }
            Request::ReviewFetchCapture {
                workspace,
                worktree,
                observation,
            } => {
                let entry = self.entry(&workspace, &worktree)?;
                let id = self.active(&workspace)?;
                self.abandon_review()?;
                let plan =
                    self.writes
                        .propose_capture(&id, &entry, &observation, &self.read_options)?;
                match self.set_review(review::Action::FetchCapture(Box::new(plan.clone()))) {
                    Ok(display) => Data::Review(display),
                    Err(error) => {
                        self.writes.cancel_capture(&plan.transport.id)?;
                        return Err(error);
                    }
                }
            }
            Request::FetchCaptures {
                workspace,
                worktree,
                offset,
            } => {
                let id = self.active(&workspace)?;
                let selected = worktree
                    .map(|w| self.entry(&workspace, &w).map(|e| e.id))
                    .transpose()?;
                let mut page = self.writes.captures(&id, selected.as_ref(), offset)?;
                for capture in &mut page.items {
                    if capture.phase == crate::remote::fetch::FetchCapturePhase::Proposed {
                        let live=self.pending.as_ref().is_some_and(|pending|matches!(&pending.action,review::Action::FetchCapture(plan) if plan.transport.id==capture.plan.transport.id) && self.review(&pending.id).is_ok());
                        if !live {
                            capture.phase = crate::remote::fetch::FetchCapturePhase::Interrupted;
                        }
                    }
                }
                Data::FetchCaptures(page)
            }
            Request::RemoteObservations {
                workspace,
                worktree,
                offset,
            } => {
                let id = self.active(&workspace)?;
                let selected = worktree
                    .map(|worktree| self.entry(&workspace, &worktree).map(|entry| entry.id))
                    .transpose()?;
                let mut page = self.writes.observations(&id, selected.as_ref(), offset)?;
                for observation in &mut page.items {
                    if observation.phase == crate::remote::observation::ObservationPhase::Proposed {
                        let live = self.pending.as_ref().is_some_and(|pending| {
                            matches!(&pending.action, review::Action::RemoteObservation(plan) if plan.id == observation.plan.id)
                                && self.review(&pending.id).is_ok()
                        });
                        if !live {
                            observation.phase =
                                crate::remote::observation::ObservationPhase::Interrupted;
                        }
                    }
                }
                Data::RemoteObservations(page)
            }
            Request::LoadBulkComposer { workspace } => {
                Data::BulkComposer(self.load_bulk_composer(&workspace)?)
            }
            Request::SaveBulkComposer { workspace, state } => {
                self.save_bulk_composer(&workspace, &state)?;
                Data::Done
            }
            Request::Remotes {
                workspace,
                worktree,
            } => {
                let entry = self.entry(&workspace, &worktree)?;
                Data::Remotes(
                    self.writes
                        .application_remotes(&entry, &self.read_options)?,
                )
            }
            Request::History {
                workspace,
                worktree,
                cursor,
            } => {
                let e = self.entry(&workspace, &worktree)?;
                Data::History(history::page_with_options(
                    &e.metadata.root.to_path()?,
                    history::HistoryScope::LocalBranches,
                    100,
                    cursor.as_ref(),
                    &self.read_options,
                )?)
            }
            Request::Commit {
                workspace,
                worktree,
                oid,
            } => {
                let e = self.entry(&workspace, &worktree)?;
                Data::Commit(history::commit_with_options(
                    &e.metadata.root.to_path()?,
                    &ObjectId::parse(&oid, &e.metadata.object_format)?,
                    &self.read_options,
                )?)
            }
            Request::Diff {
                workspace,
                worktree,
                mode,
                path,
            } => {
                let e = self.entry(&workspace, &worktree)?;
                Data::Diff(diff::inspect_with_options(
                    &e.metadata.root.to_path()?,
                    mode,
                    path.as_ref(),
                    &self.read_options,
                )?)
            }
            Request::Preview {
                workspace,
                worktree,
                path,
            } => {
                let e = self.entry(&workspace, &worktree)?;
                Data::Preview(diff::preview_with_options(
                    &e.metadata.root.to_path()?,
                    &path,
                    &self.read_options,
                )?)
            }
            Request::LoadState { workspace } => {
                let id = self.active(&workspace)?;
                Data::State(state::load(&mut self.writes, &id)?)
            }
            Request::SaveState {
                workspace,
                state: ui,
            } => {
                let id = self.active(&workspace)?;
                state::save(&mut self.writes, &id, &ui)?;
                Data::Done
            }
            Request::TrustStatus { workspace } => {
                let id = self.active(&workspace)?;
                let entries = self.writes.workspace.inventory(&id)?;
                Data::Trust(
                    entries
                        .iter()
                        .map(|e| self.writes.application_trust_state(&id, e))
                        .collect::<Result<_>>()?,
                )
            }
            Request::ReviewTrust {
                workspace,
                worktrees,
            } => {
                let id = self.active(&workspace)?;
                if worktrees.is_empty() || worktrees.len() > 20 {
                    return Err(invalid("Trust review requires 1..20 explicit worktrees"));
                }
                self.abandon_review()?;
                let mut seen = std::collections::BTreeSet::new();
                let mut reviews = Vec::new();
                for w in worktrees {
                    if !seen.insert(w.clone()) {
                        return Err(invalid("Duplicate worktree"));
                    }
                    self.entry(&workspace, &w)?;
                    reviews.push(self.writes.inspect_trust(&id, &WorktreeId(w))?);
                }
                Data::Review(self.set_review(review::Action::Trust {
                    workspace: id,
                    reviews,
                })?)
            }
            Request::Revoke {
                workspace,
                worktree,
            } => {
                self.active(&workspace)?;
                self.writes
                    .workspace
                    .inventory(&WorkspaceId(workspace))?
                    .into_iter()
                    .find(|e| e.id.0 == worktree)
                    .ok_or_else(stale)?;
                self.writes.revoke_trust(&WorktreeId(worktree))?;
                self.abandon_review()?;
                Data::Done
            }
            Request::HookPolicy {
                workspace,
                worktree,
                policy,
            } => {
                self.entry(&workspace, &worktree)?;
                self.writes
                    .remember_hook_policy(&WorktreeId(worktree), policy)?;
                self.abandon_review()?;
                Data::Done
            }
            Request::Propose {
                workspace,
                worktree,
                request,
            } => {
                let id = self.active(&workspace)?;
                self.entry(&workspace, &worktree)?;
                self.abandon_review()?;
                let p = self.writes.propose(&id, &WorktreeId(worktree), request)?;
                Data::Review(self.set_review(review::Action::Write(Box::new(p)))?)
            }
            Request::Operations { workspace, offset } => {
                let id = self.active(&workspace)?;
                Data::Operations(self.writes.application_operations(&id, offset)?)
            }
            Request::ReviewRecovery {
                workspace,
                operation,
            } => {
                let id = self.active(&workspace)?;
                self.writes.application_operation_scope(&id, &operation)?;
                self.abandon_review()?;
                let review = self.writes.inspect_recovery(&operation)?;
                Data::Review(self.set_review(review::Action::Recovery {
                    workspace: id,
                    review,
                })?)
            }
        };
        Ok(Response {
            version: 1,
            epoch: self.epoch,
            data,
        })
    }
    /// Called by the native host timer; notifications only schedule restricted reads.
    pub fn reconcile(&mut self) -> Result<bool> {
        let Some(id) = self.writes.workspace.active()? else {
            return Ok(false);
        };
        let work = self.monitor.due();
        if work.discovery {
            self.writes
                .workspace
                .scan_with_options(&id, &self.read_options)?;
            self.restart_monitor()?;
            return Ok(true);
        }
        if work.status {
            for e in self.writes.workspace.inventory(&id)? {
                if let Err(error) = self.writes.workspace.refresh(&id, &e.id) {
                    self.monitor.failure(&error);
                }
            }
            return Ok(true);
        }
        Ok(false)
    }
    pub fn record_monitor_failure(&mut self, error: &Error) {
        self.monitor.failure(error);
    }
    pub fn request_schema() -> serde_json::Value {
        json!(schemars::schema_for!(Request))
    }
    pub fn response_schema() -> serde_json::Value {
        json!(schemars::schema_for!(Response))
    }
}
