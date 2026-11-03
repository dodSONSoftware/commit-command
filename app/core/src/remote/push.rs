//! Local comparisons with past server observations. These DTOs confer no push authority.
use crate::domain::ObjectId;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PushRelation {
    UpToDate,
    FastForward,
    Behind,
    Diverged,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct OutgoingCommit {
    pub oid: ObjectId,
    pub subject: Vec<u8>,
    pub subject_truncated: bool,
}
/// A committed gitlink, without child checkout or remote reachability evidence.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SourceGitlink {
    pub path: Vec<u8>,
    pub oid: ObjectId,
}
/// Unique path/child pairs present in outgoing commit trees. No remote proof.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct OutgoingGitlink {
    pub path: Vec<u8>,
    pub oid: ObjectId,
    pub example_commit: ObjectId,
    pub mappings: Vec<HistoricalMapping>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HistoricalMapping {
    pub declared_url: Option<String>,
    pub resolution_base: Option<String>,
    pub gitmodules_blob: Option<ObjectId>,
    pub example_commit: ObjectId,
    pub destination: Option<super::Destination>,
    pub issue: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ChildReachabilitySelection {
    pub parent_observation: String,
    pub source_oid: ObjectId,
    pub path: Vec<u8>,
    pub child_oid: ObjectId,
    pub gitmodules_blob: ObjectId,
    pub child_worktree: String,
    pub child_observation: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ChildReachability {
    pub parent: PushPreview,
    pub selection: ChildReachabilitySelection,
    pub mapping: HistoricalMapping,
    pub child_remote: String,
    pub child_destination: String,
    pub child_reference: String,
    pub observed_child_oid: ObjectId,
    pub child_observed_at_ms: u64,
    pub child_generation: String,
    pub child_config_digest: String,
    pub checked_at_ms: u64,
    pub reachable: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PushPreview {
    pub source_ref: String,
    pub source_oid: ObjectId,
    pub remote: String,
    pub destination: String,
    pub destination_ref: String,
    pub observed_oid: ObjectId,
    pub observation_id: String,
    pub observed_at_ms: u64,
    pub generated_at_ms: u64,
    pub generation: String,
    pub config_digest: String,
    pub relation: PushRelation,
    pub outgoing_count: u32,
    pub incoming_count: u32,
    pub outgoing: Vec<OutgoingCommit>,
    pub outgoing_truncated: bool,
    pub source_gitlinks: Vec<SourceGitlink>,
    pub outgoing_gitlinks: Vec<OutgoingGitlink>,
    pub upstream_matches: bool,
}

/// Explicit planning selection. Nothing in this request authorizes publication.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PushPlanningSelection {
    pub worktree: String,
    pub observation: String,
    pub source_oid: ObjectId,
    pub children: Vec<ChildReachabilitySelection>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DependencyState {
    AlreadyReachable,
    SelectedChildRequired,
    Blocked,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PushDependency {
    pub parent_worktree: String,
    pub path: Vec<u8>,
    pub child_oid: ObjectId,
    pub gitmodules_blob: Option<ObjectId>,
    pub child_worktree: Option<String>,
    pub child_selected: bool,
    pub child_observation: Option<String>,
    pub observed_child_oid: Option<ObjectId>,
    pub child_observed_at_ms: Option<u64>,
    pub state: DependencyState,
    pub issue: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PlannedPush {
    pub worktree: String,
    pub preview: PushPreview,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PushDependencyPlan {
    pub generated_at_ms: u64,
    pub items: Vec<PlannedPush>,
    pub dependencies: Vec<PushDependency>,
    pub order: Vec<String>,
    /// Closed cycle witness, not merely the set of transitively blocked nodes.
    pub cycle: Vec<String>,
    pub issues: Vec<String>,
}
