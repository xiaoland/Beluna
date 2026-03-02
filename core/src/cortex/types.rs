use serde::{Deserialize, Serialize};

use crate::{cortex::cognition::CognitionState, types::Act};

fn default_max_internal_steps() -> u8 {
    4
}

fn default_sense_passthrough_max_bytes() -> usize {
    2_048
}

fn default_max_waiting_seconds() -> u64 {
    30
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ReactionLimits {
    pub max_attempts: usize,
    pub max_payload_bytes: usize,
    pub max_cycle_time_ms: u64,
    pub max_primary_calls: u8,
    pub max_sub_calls: u8,
    pub max_repair_attempts: u8,
    /// Paused: this config is retained for compatibility but currently ignored.
    pub max_primary_output_tokens: u64,
    /// Paused: this config is retained for compatibility but currently ignored.
    pub max_sub_output_tokens: u64,
    #[serde(default = "default_max_internal_steps")]
    pub max_internal_steps: u8,
    #[serde(default = "default_sense_passthrough_max_bytes")]
    pub sense_passthrough_max_bytes: usize,
    #[serde(default = "default_max_waiting_seconds")]
    pub max_waiting_seconds: u64,
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
            max_internal_steps: default_max_internal_steps(),
            sense_passthrough_max_bytes: default_sense_passthrough_max_bytes(),
            max_waiting_seconds: default_max_waiting_seconds(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmittedAct {
    pub act: Act,
    #[serde(default)]
    pub wait_for_sense_seconds: u64,
    #[serde(default)]
    pub expected_fq_sense_ids: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CortexControlDirective {
    #[serde(default)]
    pub ignore_all_trigger_for_seconds: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CortexOutput {
    #[serde(default)]
    pub emitted_acts: Vec<EmittedAct>,
    pub new_cognition_state: CognitionState,
    #[serde(default)]
    pub control: CortexControlDirective,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct InputIr {
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct OutputIr {
    pub text: String,
}
