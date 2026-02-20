use serde::{Deserialize, Serialize};

use crate::types::{Act, CognitionState, SenseId};

pub type AttentionTag = String;

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
    pub neural_signal_descriptor_id: String,
    #[serde(default)]
    pub payload_draft: serde_json::Value,
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
    UnsupportedNeuralSignalDescriptorId,
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
