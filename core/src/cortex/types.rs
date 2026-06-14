use std::collections::BTreeSet;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use validator::Validate;

use super::helpers::goal_forest_helper::{GoalForest, GoalNode};

fn default_sense_passthrough_max_bytes() -> usize {
    2_048
}

fn default_max_waiting_ticks() -> u64 {
    30
}

fn default_max_primary_turns_per_tick() -> u8 {
    4
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Validate, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct ReactionLimits {
    #[validate(range(min = 1))]
    pub max_attempts: usize,
    #[validate(range(min = 1))]
    pub max_payload_bytes: usize,
    #[validate(range(min = 1))]
    pub max_cycle_time_ms: u64,
    #[validate(range(min = 1, max = 1))]
    pub max_primary_calls: u8,
    #[serde(default = "default_max_primary_turns_per_tick")]
    #[validate(range(min = 1))]
    pub max_primary_turns_per_tick: u8,
    #[validate(range(min = 1))]
    pub max_sub_calls: u8,
    #[validate(range(min = 0, max = 1))]
    pub max_repair_attempts: u8,
    /// Paused: this config is retained for compatibility but currently ignored.
    #[validate(range(min = 1))]
    pub max_primary_output_tokens: u64,
    /// Paused: this config is retained for compatibility but currently ignored.
    #[validate(range(min = 1))]
    pub max_sub_output_tokens: u64,
    #[serde(default = "default_sense_passthrough_max_bytes")]
    #[validate(range(min = 1))]
    pub sense_passthrough_max_bytes: usize,
    #[serde(default = "default_max_waiting_ticks")]
    #[validate(range(min = 1))]
    pub max_waiting_ticks: u64,
}

impl Default for ReactionLimits {
    fn default() -> Self {
        Self {
            max_attempts: 4,
            max_payload_bytes: 16_384,
            max_cycle_time_ms: 60_000,
            max_primary_calls: 1,
            max_primary_turns_per_tick: default_max_primary_turns_per_tick(),
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
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CortexOutput {
    #[serde(default)]
    pub control: CortexControlDirective,
    #[serde(default)]
    pub pending_primary_continuation: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CognitionState {
    #[serde(default)]
    pub revision: u64,
    #[serde(default, alias = "goal_tree")]
    pub goal_forest: GoalForest,
}

pub fn new_default_cognition_state() -> CognitionState {
    CognitionState::default()
}

impl Default for CognitionState {
    fn default() -> Self {
        Self {
            revision: 0,
            goal_forest: GoalForest::default(),
        }
    }
}

pub(crate) fn validate_cognition_state(state: &CognitionState) -> Result<(), String> {
    let mut ids = BTreeSet::new();
    for node in &state.goal_forest.nodes {
        validate_goal_node(node, &mut ids)?;
    }
    Ok(())
}

fn validate_goal_node(node: &GoalNode, ids: &mut BTreeSet<String>) -> Result<(), String> {
    if node.id.trim().is_empty() {
        return Err("goal node id cannot be empty".to_string());
    }
    if node.status.trim().is_empty() {
        return Err(format!("goal node '{}' status cannot be empty", node.id));
    }
    if node.summary.trim().is_empty() {
        return Err(format!("goal node '{}' summary cannot be empty", node.id));
    }
    if !node.weight.is_finite() || !(0.0..=1.0).contains(&node.weight) {
        return Err(format!("goal node '{}' weight must be in [0,1]", node.id));
    }
    if !ids.insert(node.id.clone()) {
        return Err(format!("duplicate goal node id '{}'", node.id));
    }
    for child in &node.children {
        validate_goal_node(child, ids)?;
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct InputIr {
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct OutputIr {
    pub text: String,
}
