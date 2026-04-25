use std::collections::HashMap;

use serde::Deserialize;

use crate::{
    ai_gateway::chat::{ChatToolDefinition, ToolOverride},
    types::{NeuralSignalDescriptor, build_fq_neural_signal_id},
};

#[derive(Debug, Clone)]
pub(super) struct ActToolBinding {
    pub(super) alias: String,
    pub(super) descriptor: NeuralSignalDescriptor,
    pub(super) might_emit_sense_ids: Vec<String>,
}

pub(super) const PRIMARY_TOOL_EXPAND_SENSES: &str = "expand-senses";
pub(super) const PRIMARY_TOOL_PATCH_GOAL_FOREST: &str = "patch-goal-forest";
pub(super) const PRIMARY_TOOL_ADD_SENSE_DEFERRAL_RULE: &str = "add-sense-deferral-rule";
pub(super) const PRIMARY_TOOL_REMOVE_SENSE_DEFERRAL_RULE: &str = "remove-sense-deferral-rule";
pub(super) const PRIMARY_TOOL_SLEEP: &str = "sleep";
pub(super) const PRIMARY_TOOL_BREAK_PRIMARY_PHASE: &str = "break-primary-phase";

#[derive(Debug, Deserialize)]
pub(super) struct ExpandSenseTask {
    pub(super) sense_id: String,
    #[serde(default)]
    pub(super) use_subagent_and_instruction_is: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(super) struct ActToolArgs {
    #[serde(default)]
    pub(super) payload: serde_json::Value,
    pub(super) wait_for_sense: u64,
}

#[derive(Debug, Deserialize)]
pub(super) struct SleepArgs {
    pub(super) ticks: u64,
}

#[derive(Debug, Deserialize)]
pub(super) struct AddSenseDeferralRuleArgs {
    pub(super) rule_id: String,
    #[serde(default)]
    pub(super) min_weight: Option<f64>,
    #[serde(default)]
    pub(super) fq_sense_id_pattern: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(super) struct RemoveSenseDeferralRuleArgs {
    pub(super) rule_id: String,
}

#[derive(Debug, Deserialize)]
pub(super) struct PatchGoalForestArgs {
    pub(super) patch_instructions: String,
    #[serde(default)]
    pub(super) reset_context: bool,
}

pub(super) fn primary_internal_tools() -> Vec<ChatToolDefinition> {
    vec![
        ChatToolDefinition {
            name: PRIMARY_TOOL_EXPAND_SENSES.to_string(),
            description: Some(
                "Expand senses with raw payload or per-task sub-agent instruction.".to_string(),
            ),
            input_schema: serde_json::json!({
                "type": "array",
                "minItems": 1,
                "items": {
                    "type": "object",
                    "properties": {
                        "sense_id": { "type": "string", "minLength": 1 },
                        "use_subagent_and_instruction_is": { "type": "string", "minLength": 1 }
                    },
                    "required": ["sense_id"]
                }
            }),
        },
        ChatToolDefinition {
            name: PRIMARY_TOOL_ADD_SENSE_DEFERRAL_RULE.to_string(),
            description: Some("Add one sense deferral rule.".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "rule_id": { "type": "string", "minLength": 1 },
                    "min_weight": {
                        "type": "number", "minimum": 0, "maximum": 1,
                        "description": "Senses with a weight < this value will be deferred"
                    },
                    "fq_sense_id_pattern": {
                        "type": "string", "minLength": 1,
                        "description": "The senses matching this pattern will be deferred."
                    }
                },
                "required": ["rule_id"]
            }),
        },
        ChatToolDefinition {
            name: PRIMARY_TOOL_REMOVE_SENSE_DEFERRAL_RULE.to_string(),
            description: Some("Remove one sense deferral rule by rule_id.".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "rule_id": { "type": "string", "minLength": 1 }
                },
                "required": ["rule_id"],
            }),
        },
        ChatToolDefinition {
            name: PRIMARY_TOOL_SLEEP.to_string(),
            description: Some("Sleep for N ticks.".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "ticks": {
                        "type": "integer",
                        "minimum": 1
                    }
                },
                "required": ["ticks"]
            }),
        },
        ChatToolDefinition {
            name: PRIMARY_TOOL_BREAK_PRIMARY_PHASE.to_string(),
            description: Some(
                concat!(
                    "Mark that Primary has finished all reasoning, sense expansion, and act emissions ",
                    "for the current admitted tick. Call this only when Primary should do no further ",
                    "work in this tick. If this turn also emits act tool calls, those acts are dispatched ",
                    "before the runtime applies the break."
                )
                .to_string(),
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }),
        },
        ChatToolDefinition {
            name: PRIMARY_TOOL_PATCH_GOAL_FOREST.to_string(),
            description: Some("Patch the goal-forest as your will.".to_string()),
            input_schema: patch_goal_forest_tool_input_schema(),
        },
    ]
}

pub(super) fn build_act_tool_bindings(
    act_descriptors: &[NeuralSignalDescriptor],
    sense_descriptors: &[NeuralSignalDescriptor],
) -> Vec<ActToolBinding> {
    let mut endpoint_emitted_sense_catalog: HashMap<String, Vec<String>> = HashMap::new();
    for descriptor in sense_descriptors {
        let fq_sense_id = build_fq_neural_signal_id(
            &descriptor.endpoint_id,
            &descriptor.neural_signal_descriptor_id,
        );
        endpoint_emitted_sense_catalog
            .entry(descriptor.endpoint_id.clone())
            .or_default()
            .push(fq_sense_id);
    }
    for might_emit_sense_ids in endpoint_emitted_sense_catalog.values_mut() {
        might_emit_sense_ids.sort();
        might_emit_sense_ids.dedup();
    }

    act_descriptors
        .iter()
        .map(|descriptor| {
            let might_emit_sense_ids = endpoint_emitted_sense_catalog
                .get(&descriptor.endpoint_id)
                .cloned()
                .unwrap_or_default();
            ActToolBinding {
                alias: transport_safe_act_tool_alias(
                    &descriptor.endpoint_id,
                    &descriptor.neural_signal_descriptor_id,
                ),
                descriptor: descriptor.clone(),
                might_emit_sense_ids,
            }
        })
        .collect()
}

pub(super) fn dynamic_act_tool_overrides(
    act_bindings: &[ActToolBinding],
    max_waiting_ticks: u64,
) -> Vec<ToolOverride> {
    act_bindings
        .iter()
        .map(|binding| {
            ToolOverride::Set(ChatToolDefinition {
                name: binding.alias.clone(),
                // TODO: replace with NSDescriptor.description
                description: Some(format!(
                    "Emit {}",
                    build_fq_neural_signal_id(
                        &binding.descriptor.endpoint_id,
                        &binding.descriptor.neural_signal_descriptor_id
                    )
                )),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "payload": binding.descriptor.payload_schema.clone(),
                        "wait_for_sense": {
                            "type": "integer",
                            "description": "Number of ticks to wait for the specified senses after dispatching the act.",
                            "minimum": 0,
                            "maximum": max_waiting_ticks,
                        }
                    },
                    "required": ["payload", "wait_for_sense"],
                    "additionalProperties": false
                }),
            })
        })
        .collect()
}

pub(super) fn parse_patch_goal_forest_args(
    arguments_json: &str,
) -> Result<PatchGoalForestArgs, String> {
    serde_json::from_str::<PatchGoalForestArgs>(arguments_json).map_err(|err| err.to_string())
}

fn transport_safe_act_tool_alias(endpoint_id: &str, neural_signal_descriptor_id: &str) -> String {
    let fq_act_id = build_fq_neural_signal_id(endpoint_id, neural_signal_descriptor_id);
    let mut normalized = String::with_capacity(fq_act_id.len());
    for ch in fq_act_id.chars() {
        match ch {
            '.' => normalized.push('-'),
            '/' => normalized.push('_'),
            c if c.is_ascii_alphanumeric() => normalized.push(c),
            _ => normalized.push('_'),
        }
    }
    format!("act_{normalized}")
}

fn patch_goal_forest_tool_input_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "patch_instructions": {
                "type": "string",
                "minLength": 1
            },
            "reset_context": {
                "type": "boolean",
                "default": false,
                "description": "Reset to avoid context rot, the goal forest will maintains your cognition continuity"
            }
        },
        "required": ["patch_instructions"],
        "additionalProperties": false
    })
}
