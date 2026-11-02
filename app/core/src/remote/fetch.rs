//! Fetch downloads are immutable data, never authority to update managed refs.
use super::observation::ObservationPlan;
use crate::domain::{Issue, ObjectId};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FetchCapturePlan {
    #[serde(flatten)]
    pub transport: ObservationPlan,
    pub observation_id: String,
    pub expected_oid: ObjectId,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FetchCapturePhase {
    Proposed,
    Running,
    Complete,
    Blocked,
    Cancelled,
    Interrupted,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FetchArtifact {
    pub bundle_digest: String,
    pub bundle_bytes: u64,
    pub object_count: u32,
    pub object_bytes: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FetchCapture {
    pub plan: FetchCapturePlan,
    pub phase: FetchCapturePhase,
    pub artifact: Option<FetchArtifact>,
    pub captured_at_ms: Option<u64>,
    pub issue: Option<Issue>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FetchCapturePage {
    pub items: Vec<FetchCapture>,
    pub next: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FetchImportPlan {
    #[serde(flatten)]
    pub transport: ObservationPlan,
    pub capture_id: String,
    pub expected_oid: ObjectId,
    pub bundle_digest: String,
    pub bundle_bytes: u64,
    pub tracking_ref: String,
    pub expected_old: Option<ObjectId>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FetchImportPhase {
    Proposed,
    ImportingObjects,
    PublicationPending,
    ReferencePrepared,
    ReferenceUpdated,
    Complete,
    Blocked,
    Cancelled,
    Interrupted,
    Uncertain,
    Aborted,
    ObservedTarget,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FetchReferenceStatus {
    NotAttempted,
    Updated,
    MatchesTarget,
    Unchanged,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FetchImport {
    pub plan: FetchImportPlan,
    pub phase: FetchImportPhase,
    pub objects_imported: bool,
    pub reference_status: FetchReferenceStatus,
    pub issue: Option<Issue>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FetchImportPage {
    pub items: Vec<FetchImport>,
    pub next: Option<usize>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FetchImportRecovery {
    pub operation: FetchImport,
    pub action: String,
    pub actual_oid: Option<ObjectId>,
    pub fingerprint: String,
    pub explanation: String,
}
