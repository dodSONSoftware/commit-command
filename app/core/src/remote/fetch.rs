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
