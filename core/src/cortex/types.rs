use serde::{Deserialize, Serialize};

use crate::runtime_types::{Act, CognitionState, RequestedResources, SenseId};

pub type AttentionTag = String;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AffordanceCapability {
    pub endpoint_id: String,
    #[serde(default)]
    pub allowed_capability_ids: Vec<String>,
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
    pub fn resolve(&self, endpoint_id: &str) -> Option<&AffordanceCapability> {
        self.affordances
            .iter()
            .find(|item| item.endpoint_id == endpoint_id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReactionLimits {
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
            max_attempts: 4,
            max_payload_bytes: 16_384,
            max_cycle_time_ms: 60_000,
            max_primary_calls: 1,
            max_sub_calls: 2,
            max_repair_attempts: 1,
            max_primary_output_tokens: 1_024,
            max_sub_output_tokens: 768,
        }
    }
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
    pub endpoint_id: String,
    pub capability_id: String,
    #[serde(default)]
    pub capability_instance_id: String,
    #[serde(default)]
    pub payload_draft: serde_json::Value,
    #[serde(default)]
    pub requested_resources: RequestedResources,
    #[serde(default)]
    pub goal_hint: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClampViolationCode {
    MissingIntentSpan,
    MissingBasedOn,
    UnknownSenseId,
    UnknownEndpointId,
    UnsupportedCapabilityId,
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
    pub acts: Vec<Act>,
    pub based_on: Vec<SenseId>,
    pub attention_tags: Vec<AttentionTag>,
    pub violations: Vec<ClampViolation>,
    pub original_drafts: Vec<AttemptDraft>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CortexOutput {
    #[serde(default)]
    pub acts: Vec<Act>,
    pub new_cognition_state: CognitionState,
}
