use serde::{Deserialize, Serialize};

fn default_sense_passthrough_max_bytes() -> usize {
    2_048
}

fn default_max_waiting_ticks() -> u64 {
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
    #[serde(default = "default_sense_passthrough_max_bytes")]
    pub sense_passthrough_max_bytes: usize,
    #[serde(default = "default_max_waiting_ticks")]
    pub max_waiting_ticks: u64,
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
            sense_passthrough_max_bytes: default_sense_passthrough_max_bytes(),
            max_waiting_ticks: default_max_waiting_ticks(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CortexControlDirective {
    #[serde(default)]
    pub ignore_all_trigger_for_ticks: Option<u64>,
    #[serde(default)]
    pub wait_for_sense: Option<WaitForSenseControlDirective>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WaitForSenseControlDirective {
    pub act_instance_id: String,
    #[serde(default)]
    pub expected_fq_sense_ids: Vec<String>,
    pub wait_ticks: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CortexOutput {
    #[serde(default)]
    pub control: CortexControlDirective,
    #[serde(default)]
    pub pending_primary_continuation: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct InputIr {
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct OutputIr {
    pub text: String,
}
