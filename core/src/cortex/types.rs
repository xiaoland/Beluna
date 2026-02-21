use serde::{Deserialize, Serialize};

use crate::types::{Act, CognitionState};

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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CortexOutput {
    #[serde(default)]
    pub acts: Vec<Act>,
    pub new_cognition_state: CognitionState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct InputIr {
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct OutputIr {
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct ActDraft {
    pub endpoint_id: String,
    pub neural_signal_descriptor_id: String,
    #[serde(default)]
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub(crate) struct ActsHelperOutput {
    #[serde(default)]
    pub acts: Vec<ActDraft>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub(crate) struct GoalStackPatch {
    #[serde(default)]
    pub ops: Vec<GoalStackPatchOp>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub(crate) enum GoalStackPatchOp {
    Push { goal_id: String, summary: String },
    Pop,
    ReplaceTop { goal_id: String, summary: String },
    Clear,
}
