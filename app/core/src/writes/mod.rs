//! Non-default production candidate. Trust and one-use approvals are enforced in Core.
//! Supported execution envelope and gate evidence are in docs/architecture/M2_CONTROLLED_WRITES.md.
#[cfg(not(unix))]
compile_error!(
    "controlled-writes currently requires a Unix target; default inspection is portable"
);
mod adapter;
#[cfg(feature = "application-services")]
mod application;
#[cfg(feature = "application-services")]
mod child_reachability;
#[cfg(feature = "application-services")]
mod fetch;
#[cfg(feature = "application-services")]
mod fetch_import;
mod policy;
#[cfg(feature = "application-services")]
mod push_batch;
#[cfg(feature = "application-services")]
mod push_check;
#[cfg(feature = "application-services")]
mod push_dependencies;
#[cfg(feature = "application-services")]
mod push_mappings;
#[cfg(feature = "application-services")]
mod push_preview;
#[cfg(feature = "application-services")]
mod push_publication;
#[cfg(feature = "application-services")]
mod remote;
mod store;
mod transaction;
#[cfg(feature = "application-services")]
mod upstream;
#[cfg(feature = "application-services")]
mod version_action;
use crate::domain::{
    digest, now_ms, NativePath, ObjectFormat, RepositoryId, WorkspaceId, WorktreeId,
};
use crate::inspection::{Metadata, ReadOptions};
use crate::{inspection, process, workspace::WorkspaceService, Error, ErrorKind, Result};
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use uuid::Uuid;

fn fail(kind: ErrorKind, message: &str) -> Error {
    Error::with_kind(kind, message)
}
fn unsupported(message: &str) -> Error {
    fail(ErrorKind::UnsupportedRepositoryState, message)
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "application-services", derive(schemars::JsonSchema))]
pub enum HookPolicy {
    Execute,
    BypassAll,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "application-services", derive(schemars::JsonSchema))]
pub enum SigningPreference {
    Configured,
    Unsigned,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "application-services", derive(schemars::JsonSchema))]
pub enum ContentSource {
    Index,
    WorkingTree,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "application-services", derive(schemars::JsonSchema))]
pub struct SelectedPath {
    pub path: NativePath,
    pub source: ContentSource,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "application-services", derive(schemars::JsonSchema))]
pub struct Hook {
    pub name: String,
    pub source: NativePath,
    pub resolved: NativePath,
    pub executable: bool,
    pub fingerprint: String,
    pub framework: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "application-services", derive(schemars::JsonSchema))]
pub struct Signing {
    pub format: String,
    pub key: String,
    pub program: Option<NativePath>,
    pub program_hash: String,
    pub fingerprint: String,
    pub public_key: Option<Vec<u8>>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct EffectiveConfig {
    config_hash: String,
    config_origins: Vec<Vec<u8>>,
    name: String,
    email: String,
    date: String,
    hooks: Vec<Hook>,
    signing: Option<Signing>,
    signer_home: Option<NativePath>,
    file_mode: bool,
}
#[derive(Debug, Clone)]
pub struct Options {
    pub process: ReadOptions,
    pub signer_home: Option<PathBuf>,
    pub global_config: Option<PathBuf>,
    pub approval_lifetime: Duration,
    #[cfg(feature = "application-services")]
    pub remote: crate::remote::TransportOptions,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            process: ReadOptions {
                timeout: Duration::from_secs(120),
                max_output: 16 * 1024 * 1024,
                ..ReadOptions::default()
            },
            signer_home: None,
            global_config: None,
            approval_lifetime: Duration::from_secs(600),
            #[cfg(feature = "application-services")]
            remote: crate::remote::TransportOptions::default(),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "application-services", derive(schemars::JsonSchema))]
pub struct TrustReview {
    pub review_id: String,
    pub repository_id: RepositoryId,
    pub worktree_id: WorktreeId,
    pub metadata: Metadata,
    pub binding: String,
    pub hooks: Vec<Hook>,
    pub signing: Option<Signing>,
    pub config_hash: String,
    pub config_origins: Vec<Vec<u8>>,
    pub signer_home: Option<NativePath>,
    pub digest: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[cfg_attr(feature = "application-services", derive(schemars::JsonSchema))]
pub enum WriteRequest {
    Commit {
        selected: Vec<SelectedPath>,
        message: String,
        hooks: HookPolicy,
        signing: SigningPreference,
    },
    Stage {
        paths: Vec<NativePath>,
        hooks: HookPolicy,
    },
    Unstage {
        paths: Vec<NativePath>,
        hooks: HookPolicy,
    },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "application-services", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Proposed,
    Approved,
    Preparing,
    HooksRunning,
    Revalidated,
    ObjectReady,
    IndexLockWritten,
    PublicationPending,
    ReferenceUpdated,
    IndexRenamed,
    HeadLogWritten,
    Published,
    IndexReconciled,
    PostProcessing,
    Complete,
    Blocked,
    NeedsReauthorization,
    CancelledBeforePublication,
    PublishedNeedsReconciliation,
    PublishedWithPostProcessingFailure,
    NeedsManualReconciliation,
    UncertainOutcome,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "application-services", derive(schemars::JsonSchema))]
pub struct Proposal {
    pub operation_id: String,
    pub workspace_id: WorkspaceId,
    pub repository_id: RepositoryId,
    pub worktree_id: WorktreeId,
    pub request: WriteRequest,
    pub branch: String,
    pub expected_head: Option<String>,
    pub approved_tree: String,
    pub message: Vec<u8>,
    pub author_name: String,
    pub author_email: String,
    pub author_date: String,
    pub hooks: HookPolicy,
    pub hook_manifest: Vec<Hook>,
    pub signing: Option<Signing>,
    pub created_at_ms: u64,
    pub review_digest: String,
    pub patch: Vec<u8>,
}
/// An in-memory capability. It cannot be deserialized, printed or constructed by adapters.
pub struct Approval {
    operation: String,
    token: String,
    session: String,
    issued: Instant,
}
impl std::fmt::Debug for Approval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Approval(<redacted>)")
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "application-services", derive(schemars::JsonSchema))]
pub struct Outcome {
    pub operation_id: String,
    pub phase: Phase,
    pub published: bool,
    pub new_head: Option<String>,
    pub index_reconciled: bool,
    pub post_processing: Vec<String>,
    pub reason: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Receipt {
    identity: String,
    hash: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Operation {
    proposal: Proposal,
    metadata: Metadata,
    binding: String,
    config: EffectiveConfig,
    original_index_hash: String,
    original_entries: Vec<policy::Entry>,
    selected_fingerprints: Vec<(NativePath, String)>,
    candidate_hash: String,
    selected_index_hash: String,
    new_head: Option<String>,
    phase: Phase,
    published: bool,
    reconciled: bool,
    lock: Option<Receipt>,
    head_lock: Option<Receipt>,
    head_log_before: Option<String>,
    post_attempted: Vec<String>,
    post_results: Vec<String>,
    reason: Option<String>,
}
impl Operation {
    fn outcome(&self) -> Outcome {
        Outcome {
            operation_id: self.proposal.operation_id.clone(),
            phase: self.phase,
            published: self.published,
            new_head: self.new_head.clone(),
            index_reconciled: self.reconciled,
            post_processing: self.post_results.clone(),
            reason: self.reason.clone(),
        }
    }
}
pub struct ControlledWrites {
    pub(crate) workspace: WorkspaceService,
    options: Options,
    session: String,
    trust_reviews: BTreeMap<String, TrustReview>,
    recovery_reviews: BTreeMap<String, RecoveryReview>,
    #[cfg(feature = "write-test-support")]
    test_fault: Option<TestFault>,
    #[cfg(all(feature = "write-test-support", feature = "application-services"))]
    publication_fault: Option<(bool, TestFaultKind)>,
    #[cfg(all(feature = "write-test-support", feature = "application-services"))]
    version_fault: Option<(bool, TestFaultKind)>,
    #[cfg(all(feature = "write-test-support", feature = "application-services"))]
    import_fault: Option<(crate::remote::fetch::FetchImportPhase, TestFaultKind)>,
    #[cfg(all(feature = "write-test-support", feature = "application-services"))]
    upstream_fault: Option<(crate::remote::upstream::UpstreamPhase, TestFaultKind)>,
}
#[cfg(feature = "write-test-support")]
#[derive(Debug, Clone)]
pub struct TestFault {
    pub at: Phase,
    pub kind: TestFaultKind,
}
#[cfg(feature = "write-test-support")]
#[derive(Debug, Clone)]
pub enum TestFaultKind {
    Sigkill,
    StorageFailure,
    CorruptCandidate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "application-services", derive(schemars::JsonSchema))]
pub struct RecoveryReview {
    pub operation_id: String,
    pub outcome: Outcome,
    pub action: String,
    pub explanation: String,
    pub fingerprint: String,
}

#[cfg(feature = "application-services")]
mod ssh_transport;

#[cfg(feature = "application-services")]
pub(crate) fn credential_fingerprint(path: &Path) -> Result<String> {
    adapter::program_fingerprint(path)
}

#[cfg(feature = "application-services")]
pub(crate) fn authentication_tools(
    options: &crate::remote::TransportOptions,
    transport: &crate::remote::Transport,
) -> Result<Vec<crate::remote::observation::ToolBinding>> {
    let mut tools = remote::toolchain(
        &ReadOptions::default(),
        options.ca_file.as_deref(),
        transport,
    )?;
    if *transport == crate::remote::Transport::Ssh {
        tools.extend(ssh_transport::bindings(options.ssh.as_ref())?);
    }
    if let Some(policy) = &options.https_credentials {
        tools.extend(crate::credentials::bindings(policy)?);
    }
    Ok(tools)
}

#[cfg(feature = "application-services")]
impl ControlledWrites {
    pub(crate) fn authentication_options(&self) -> crate::remote::TransportOptions {
        self.options.remote.clone()
    }
    pub(crate) fn authentication_global_config(&self) -> Option<&Path> {
        self.options.global_config.as_deref()
    }
}
#[cfg(feature = "application-services")]
pub(crate) fn authentication_helper_choices(
    options: &ReadOptions,
) -> Result<Vec<crate::authentication::HelperChoice>> {
    let tools = remote::toolchain(options, None, &crate::remote::Transport::Https)?;
    let directory = tools
        .iter()
        .find(|t| t.role == "exec_directory")
        .ok_or_else(|| unsupported("Git helper directory unavailable"))?
        .path
        .to_path()?;
    let helper = directory.join("git-credential-osxkeychain");
    Ok(crate::credentials::helper_binding(&helper)
        .ok()
        .and_then(|t| t.path.to_path().ok())
        .and_then(|p| p.to_str().map(str::to_owned))
        .map(|path| {
            vec![crate::authentication::HelperChoice {
                label: "macOS Keychain (existing Git helper)".into(),
                path,
            }]
        })
        .unwrap_or_default())
}
#[cfg(feature = "application-services")]
pub(crate) fn authentication_agent_available(socket: &Path) -> bool {
    ssh_transport::socket_binding(socket).is_ok()
}
