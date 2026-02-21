use serde::Deserialize;

use crate::{
    cortex::types::{ActsHelperOutput, GoalStackPatch, GoalStackPatchOp},
    types::{CognitionState, GoalFrame},
};

#[derive(Debug, Clone, Deserialize, Default)]
pub(crate) struct GoalStackHelperOutput {
    #[serde(default)]
    pub patch: GoalStackPatch,
}

pub(crate) fn empty_goal_stack_patch() -> GoalStackPatch {
    GoalStackPatch::default()
}

pub(crate) fn apply_goal_stack_patch(
    previous: &CognitionState,
    patch: &GoalStackPatch,
) -> CognitionState {
    let mut next = previous.clone();
    next.revision = next.revision.saturating_add(1);

    for op in &patch.ops {
        match op {
            GoalStackPatchOp::Push { goal_id, summary } => {
                next.goal_stack.push(GoalFrame {
                    goal_id: goal_id.clone(),
                    summary: summary.clone(),
                });
            }
            GoalStackPatchOp::Pop => {
                next.goal_stack.pop();
            }
            GoalStackPatchOp::ReplaceTop { goal_id, summary } => {
                if next.goal_stack.is_empty() {
                    next.goal_stack.push(GoalFrame {
                        goal_id: goal_id.clone(),
                        summary: summary.clone(),
                    });
                } else if let Some(top) = next.goal_stack.last_mut() {
                    top.goal_id = goal_id.clone();
                    top.summary = summary.clone();
                }
            }
            GoalStackPatchOp::Clear => {
                next.goal_stack.clear();
            }
        }
    }

    next
}

pub(crate) fn acts_json_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "acts": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "endpoint_id": { "type": "string" },
                        "neural_signal_descriptor_id": { "type": "string" },
                        "payload": {}
                    },
                    "required": ["endpoint_id", "neural_signal_descriptor_id", "payload"],
                    "additionalProperties": false
                }
            }
        },
        "required": ["acts"],
        "additionalProperties": false
    })
}

pub(crate) fn goal_stack_patch_json_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "patch": {
                "type": "object",
                "properties": {
                    "ops": {
                        "type": "array",
                        "items": {
                            "oneOf": [
                                {
                                    "type": "object",
                                    "properties": {
                                        "op": { "type": "string", "const": "push" },
                                        "goal_id": { "type": "string" },
                                        "summary": { "type": "string" }
                                    },
                                    "required": ["op", "goal_id", "summary"],
                                    "additionalProperties": false
                                },
                                {
                                    "type": "object",
                                    "properties": {
                                        "op": { "type": "string", "const": "pop" }
                                    },
                                    "required": ["op"],
                                    "additionalProperties": false
                                },
                                {
                                    "type": "object",
                                    "properties": {
                                        "op": { "type": "string", "const": "replace_top" },
                                        "goal_id": { "type": "string" },
                                        "summary": { "type": "string" }
                                    },
                                    "required": ["op", "goal_id", "summary"],
                                    "additionalProperties": false
                                },
                                {
                                    "type": "object",
                                    "properties": {
                                        "op": { "type": "string", "const": "clear" }
                                    },
                                    "required": ["op"],
                                    "additionalProperties": false
                                }
                            ]
                        }
                    }
                },
                "required": ["ops"],
                "additionalProperties": false
            }
        },
        "required": ["patch"],
        "additionalProperties": false
    })
}

pub(crate) fn parse_acts_helper_output(text: &str) -> Result<ActsHelperOutput, serde_json::Error> {
    serde_json::from_str::<ActsHelperOutput>(text)
}

pub(crate) fn parse_goal_stack_helper_output(
    text: &str,
) -> Result<GoalStackHelperOutput, serde_json::Error> {
    serde_json::from_str::<GoalStackHelperOutput>(text)
}
