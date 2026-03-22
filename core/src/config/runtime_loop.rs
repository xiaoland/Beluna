use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use validator::Validate;

fn default_tick_interval_ms() -> u64 {
    10_000
}

fn default_tick_missed_behavior() -> TickMissedBehavior {
    TickMissedBehavior::Skip
}

fn default_max_deferring_nums() -> usize {
    256
}

fn default_afferent_sidecar_capacity() -> usize {
    128
}

fn default_efferent_shutdown_drain_timeout_ms() -> u64 {
    5_000
}

fn default_sense_queue_capacity() -> usize {
    32
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TickMissedBehavior {
    Skip,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CoreLoopConfig {
    #[serde(default = "default_sense_queue_capacity")]
    #[validate(range(min = 1))]
    pub sense_queue_capacity: usize,
    #[serde(default = "default_max_deferring_nums")]
    #[validate(range(min = 1))]
    pub max_deferring_nums: usize,
    #[serde(default = "default_afferent_sidecar_capacity")]
    #[validate(range(min = 1))]
    pub afferent_sidecar_capacity: usize,
    #[serde(default = "default_efferent_shutdown_drain_timeout_ms")]
    #[validate(range(min = 1))]
    pub efferent_shutdown_drain_timeout_ms: u64,
    #[serde(default = "default_tick_interval_ms")]
    #[validate(range(min = 1))]
    pub tick_interval_ms: u64,
    #[serde(default = "default_tick_missed_behavior")]
    pub tick_missed_behavior: TickMissedBehavior,
}

impl Default for CoreLoopConfig {
    fn default() -> Self {
        Self {
            sense_queue_capacity: default_sense_queue_capacity(),
            max_deferring_nums: default_max_deferring_nums(),
            afferent_sidecar_capacity: default_afferent_sidecar_capacity(),
            efferent_shutdown_drain_timeout_ms: default_efferent_shutdown_drain_timeout_ms(),
            tick_interval_ms: default_tick_interval_ms(),
            tick_missed_behavior: default_tick_missed_behavior(),
        }
    }
}
