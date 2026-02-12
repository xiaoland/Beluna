use serde::{Deserialize, Serialize};

use crate::admission::types::{IntentAttempt, RequestedResources};

pub type ReactionId = u64;
pub type SenseId = String;
pub type AttemptId = String;
pub type AttentionTag = String;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SenseDelta {
    pub sense_id: SenseId,
    pub source: String,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EndpointSnapshot {
    pub endpoint_key: String,
    pub blob: serde_json::Value,
    pub truncated: bool,
    pub blob_bytes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdmissionOutcomeSignal {
    pub attempt_id: AttemptId,
    pub code: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConstitutionalIntent {
    pub intent_key: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnvironmentalIntentSignal {
    pub signal_key: String,
    pub constraint_code: String,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmergentIntentCandidate {
    pub candidate_key: String,
    pub summary: String,
    pub provenance: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct IntentContext {
    #[serde(default)]
    pub constitutional: Vec<ConstitutionalIntent>,
    #[serde(default)]
    pub environmental: Vec<EnvironmentalIntentSignal>,
    #[serde(default)]
    pub emergent_candidates: Vec<EmergentIntentCandidate>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AffordanceCapability {
    pub affordance_key: String,
    #[serde(default)]
    pub allowed_capability_handles: Vec<String>,
    pub payload_schema: serde_json::Value,
    pub max_payload_bytes: usize,
    #[serde(default)]
    pub default_resources: RequestedResources,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct CapabilityCatalog {
    pub version: String,
    #[serde(default)]
    pub affordances: Vec<AffordanceCapability>,
}

impl CapabilityCatalog {
    pub fn resolve(&self, affordance_key: &str) -> Option<&AffordanceCapability> {
        self.affordances
            .iter()
            .find(|item| item.affordance_key == affordance_key)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReactionLimits {
    pub max_sense_items: usize,
    pub max_snapshot_items: usize,
    pub max_snapshot_bytes_per_item: usize,
    pub max_attempts: usize,
    pub max_payload_bytes: usize,
    pub max_cycle_time_ms: u64,
    pub max_primary_calls: u8,
    pub max_sub_calls: u8,
    pub max_repair_attempts: u8,
    pub max_primary_output_tokens: u64,
    pub max_sub_output_tokens: u64,
}

impl Default for ReactionLimits {
    fn default() -> Self {
        Self {
            max_sense_items: 64,
            max_snapshot_items: 32,
            max_snapshot_bytes_per_item: 16_384,
            max_attempts: 4,
            max_payload_bytes: 16_384,
            max_cycle_time_ms: 5_000,
            max_primary_calls: 1,
            max_sub_calls: 2,
            max_repair_attempts: 1,
            max_primary_output_tokens: 1_024,
            max_sub_output_tokens: 768,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReactionInput {
    pub reaction_id: ReactionId,
    #[serde(default)]
    pub sense_window: Vec<SenseDelta>,
    #[serde(default)]
    pub env_snapshots: Vec<EndpointSnapshot>,
    #[serde(default)]
    pub admission_feedback: Vec<AdmissionOutcomeSignal>,
    pub capability_catalog: CapabilityCatalog,
    #[serde(default)]
    pub limits: ReactionLimits,
    #[serde(default)]
    pub context: IntentContext,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReactionResult {
    pub reaction_id: ReactionId,
    #[serde(default)]
    pub based_on: Vec<SenseId>,
    #[serde(default)]
    pub attention_tags: Vec<AttentionTag>,
    #[serde(default)]
    pub attempts: Vec<IntentAttempt>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProseIr {
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AttemptDraft {
    pub intent_span: String,
    #[serde(default)]
    pub based_on: Vec<SenseId>,
    #[serde(default)]
    pub attention_tags: Vec<AttentionTag>,
    pub affordance_key: String,
    pub capability_handle: String,
    #[serde(default)]
    pub payload_draft: serde_json::Value,
    #[serde(default)]
    pub requested_resources: RequestedResources,
    #[serde(default)]
    pub commitment_hint: Option<String>,
    #[serde(default)]
    pub goal_hint: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClampViolationCode {
    MissingIntentSpan,
    MissingBasedOn,
    UnknownSenseId,
    UnknownAffordance,
    UnsupportedCapabilityHandle,
    PayloadTooLarge,
    PayloadSchemaViolation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClampViolation {
    pub code: ClampViolationCode,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClampResult {
    pub attempts: Vec<IntentAttempt>,
    pub based_on: Vec<SenseId>,
    pub attention_tags: Vec<AttentionTag>,
    pub violations: Vec<ClampViolation>,
    pub original_drafts: Vec<AttemptDraft>,
}
