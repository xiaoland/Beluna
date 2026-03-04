use std::collections::{BTreeMap, BTreeSet};

use tokio::time::Duration;

mod model;
mod patch;

pub use model::{
    CognitionState, GoalForest, GoalForestPatchOp, GoalNode, new_default_cognition_state,
};
pub(crate) use patch::apply_goal_forest_op;

use crate::{
    ai_gateway::chat::OutputMode,
    cortex::{
        error::{CortexError, extractor_failed},
        helpers::{self, CognitionOrgan, HelperRuntime},
        prompts,
    },
};

const GOAL_FOREST_EMPTY_FALLBACK: &str = concat!(
    "There's no trees in the goal forest currently.\n",
    "Try to plan some trees, and then plant, sprout, prune, trim them."
);

#[derive(Clone, Default)]
pub(crate) struct GoalForestHelper;

impl GoalForestHelper {
    pub(crate) async fn to_input_ir_section(
        &self,
        _runtime: &impl HelperRuntime,
        cycle_id: u64,
        _deadline: Duration,
        goal_forest: &GoalForest,
    ) -> String {
        let stage = CognitionOrgan::GoalForest.stage();
        let input_payload = helpers::pretty_json(&serde_json::json!({
            "goal_forest": goal_forest,
        }));
        helpers::log_organ_input(cycle_id, stage, &input_payload);

        let output = if goal_forest.nodes.is_empty() {
            goal_forest_empty_one_shot().to_string()
        } else {
            goal_forest_ascii(&goal_forest.nodes)
        };

        helpers::log_organ_output(cycle_id, stage, &output);
        output
    }

    pub(crate) async fn patch_goal_forest_with_sub_agent(
        &self,
        runtime: &impl HelperRuntime,
        cycle_id: u64,
        goal_forest_nodes: &mut Vec<GoalNode>,
        patch_instructions: &str,
    ) -> Result<serde_json::Value, CortexError> {
        let instructions = patch_instructions.trim();
        if instructions.is_empty() {
            return Err(extractor_failed("patch_instructions cannot be empty"));
        }

        let stage = CognitionOrgan::GoalForest.stage();
        let current_goal_forest = if goal_forest_nodes.is_empty() {
            goal_forest_empty_one_shot().to_string()
        } else {
            goal_forest_ascii(goal_forest_nodes)
        };

        let input_payload = helpers::pretty_json(&serde_json::json!({
            "current_goal_forest": current_goal_forest,
            "patch_instructions": instructions,
        }));
        helpers::log_organ_input(cycle_id, stage, &input_payload);

        let prompt =
            prompts::build_goal_forest_patch_sub_agent_prompt(&current_goal_forest, instructions);
        let response = runtime
            .run_organ(
                cycle_id,
                CognitionOrgan::GoalForest,
                runtime.limits().max_sub_output_tokens,
                prompts::goal_forest_patch_sub_agent_system_prompt(),
                prompt,
                OutputMode::JsonSchema {
                    name: "goal_forest_patch_ops".to_string(),
                    schema: patch_goal_forest_ops_json_schema(),
                    strict: true,
                },
            )
            .await?;

        let sub_agent_output = response.output_text.trim().to_string();
        tracing::debug!(
            target: "cortex",
            cycle_id = cycle_id,
            stage = stage,
            sub_agent_output = %sub_agent_output,
            "goal_forest_patch_sub_agent_output"
        );

        let ops = match parse_patch_goal_forest_ops(&sub_agent_output) {
            Ok(ops) => ops,
            Err(error) => {
                tracing::warn!(
                    target: "cortex",
                    cycle_id = cycle_id,
                    stage = stage,
                    error = %error,
                    sub_agent_output = %sub_agent_output,
                    "goal_forest_patch_sub_agent_parse_failed"
                );
                return Err(extractor_failed(error));
            }
        };
        let requested_ops = ops.len();
        let mut applied_ops = 0usize;
        for op in &ops {
            if apply_goal_forest_op(goal_forest_nodes, op) {
                applied_ops += 1;
            }
        }

        let updated_goal_forest = if goal_forest_nodes.is_empty() {
            goal_forest_empty_one_shot().to_string()
        } else {
            goal_forest_ascii(goal_forest_nodes)
        };
        let output = serde_json::json!({
            "requested_ops": requested_ops,
            "applied_ops": applied_ops,
            "current_goal_forest": updated_goal_forest,
        });

        let output_payload = helpers::pretty_json(&output);
        helpers::log_organ_output(cycle_id, stage, &output_payload);
        Ok(output)
    }
}

pub(crate) fn goal_forest_json(goal_forest_nodes: &[GoalNode]) -> String {
    serde_json::to_string_pretty(goal_forest_nodes).unwrap_or_else(|_| "[]".to_string())
}

pub(crate) fn goal_forest_ascii(goal_forest_nodes: &[GoalNode]) -> String {
    let mut children_by_parent: BTreeMap<Option<&str>, Vec<&GoalNode>> = BTreeMap::new();
    for node in goal_forest_nodes {
        children_by_parent
            .entry(node.parent_id.as_deref())
            .or_default()
            .push(node);
    }

    for children in children_by_parent.values_mut() {
        children.sort_by(|lhs, rhs| compare_goal_node(lhs, rhs));
    }

    let mut lines = Vec::new();
    let mut visited = BTreeSet::new();

    if let Some(roots) = children_by_parent.get(&None) {
        for root in roots {
            append_goal_node_lines(&mut lines, &mut visited, &children_by_parent, root, 0);
        }
    }

    for node in goal_forest_nodes {
        if visited.contains(node.id.as_str()) {
            continue;
        }
        append_goal_node_lines(&mut lines, &mut visited, &children_by_parent, node, 0);
    }

    lines.join("\n")
}

fn append_goal_node_lines(
    lines: &mut Vec<String>,
    visited: &mut BTreeSet<String>,
    children_by_parent: &BTreeMap<Option<&str>, Vec<&GoalNode>>,
    node: &GoalNode,
    depth: usize,
) {
    if !visited.insert(node.id.clone()) {
        return;
    }

    let prefix = if depth == 0 {
        "+-- ".to_string()
    } else {
        format!("{}|-- ", "    ".repeat(depth))
    };
    match node.numbering.as_deref() {
        Some(numbering) => lines.push(format!(
            "{prefix}{} [{}] (w={:.2}) id={} :: {}",
            numbering, node.status, node.weight, node.id, node.summary
        )),
        None => lines.push(format!(
            "{prefix}[{}] (w={:.2}) id={} :: {}",
            node.status, node.weight, node.id, node.summary
        )),
    }

    if let Some(children) = children_by_parent.get(&Some(node.id.as_str())) {
        for child in children {
            append_goal_node_lines(lines, visited, children_by_parent, child, depth + 1);
        }
    }
}

fn compare_goal_node(lhs: &GoalNode, rhs: &GoalNode) -> std::cmp::Ordering {
    compare_numbering(lhs.numbering.as_deref(), rhs.numbering.as_deref())
        .then_with(|| lhs.id.cmp(&rhs.id))
}

pub(crate) fn goal_forest_empty_one_shot() -> &'static str {
    GOAL_FOREST_EMPTY_FALLBACK
}

fn compare_numbering(lhs: Option<&str>, rhs: Option<&str>) -> std::cmp::Ordering {
    match (lhs, rhs) {
        (None, None) => std::cmp::Ordering::Equal,
        (None, Some(_)) => std::cmp::Ordering::Less,
        (Some(_), None) => std::cmp::Ordering::Greater,
        (Some(lhs), Some(rhs)) => compare_numbering_str(lhs, rhs),
    }
}

fn compare_numbering_str(lhs: &str, rhs: &str) -> std::cmp::Ordering {
    let lhs_parts = parse_numbering(lhs);
    let rhs_parts = parse_numbering(rhs);
    let shared_len = lhs_parts.len().min(rhs_parts.len());

    for idx in 0..shared_len {
        match lhs_parts[idx].cmp(&rhs_parts[idx]) {
            std::cmp::Ordering::Equal => continue,
            ordering => return ordering,
        }
    }

    lhs_parts.len().cmp(&rhs_parts.len())
}

fn parse_numbering(numbering: &str) -> Vec<u64> {
    numbering
        .split('.')
        .map(|segment| segment.parse::<u64>().unwrap_or(0))
        .collect()
}

fn parse_patch_goal_forest_ops(arguments_json: &str) -> Result<Vec<GoalForestPatchOp>, String> {
    let value: serde_json::Value =
        serde_json::from_str(arguments_json).map_err(|err| err.to_string())?;
    let items = match value {
        serde_json::Value::Array(items) => items,
        serde_json::Value::Object(mut map) => {
            if let Some(ops_value) = map.remove("ops") {
                match ops_value {
                    serde_json::Value::Array(items) => items,
                    other => {
                        return Err(format!(
                            "invalid 'ops' field type: expected array, got {}",
                            json_type_name(&other)
                        ));
                    }
                }
            } else if map.contains_key("op") {
                vec![serde_json::Value::Object(map)]
            } else {
                let keys = map.keys().cloned().collect::<Vec<_>>().join(",");
                return Err(format!(
                    "expected array of patch ops, got object with keys [{}]",
                    keys
                ));
            }
        }
        other => {
            return Err(format!(
                "expected array of patch ops, got {}",
                json_type_name(&other)
            ));
        }
    };

    let mut ops = Vec::with_capacity(items.len());
    for (index, item) in items.into_iter().enumerate() {
        let op = serde_json::from_value::<GoalForestPatchOp>(item)
            .map_err(|err| format!("invalid patch op at index {}: {}", index, err))?;
        ops.push(op);
    }
    Ok(ops)
}

fn json_type_name(value: &serde_json::Value) -> &'static str {
    match value {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "bool",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
    }
}

fn patch_goal_forest_ops_json_schema() -> serde_json::Value {
    serde_json::json!({
      "type": "array",
      "minItems": 0,
      "description": "patch operations to apply, in order",
      "items": {
        "oneOf": [
          {
            "type": "object",
            "description": "add a new root goal (a new tree root)",
            "properties": {
              "op": {
                "type": "string",
                "const": "plant"
              },
              "status": {
                "type": "string",
                "default": "open"
              },
              "weight": {
                "type": "number",
                "minimum": 0,
                "maximum": 1,
                "default": 0
              },
              "id": {
                "type": "string",
                "description": "kebab-case phrase"
              },
              "summary": {
                "type": "string"
              }
            },
            "required": ["op", "id", "summary"],
            "additionalProperties": false
          },
          {
            "type": "object",
            "description": "add a non-root goal under selected parent",
            "properties": {
              "op": {
                "type": "string",
                "const": "sprout"
              },
              "parent_numbering": {
                "type": "string",
                "minLength": 1
              },
              "parent_id": {
                "type": "string",
                "minLength": 1
              },
              "numbering": {
                "type": "string",
                "description": "direct child numbering under parent, optional; auto-assign when omitted"
              },
              "status": {
                "type": "string",
                "default": "open"
              },
              "weight": {
                "type": "number",
                "minimum": 0,
                "maximum": 1,
                "default": 0
              },
              "id": {
                "type": "string",
                "description": "kebab-case phrase"
              },
              "summary": {
                "type": "string"
              }
            },
            "required": ["op", "id", "summary"],
            "additionalProperties": false,
            "anyOf": [
              { "required": ["parent_numbering"] },
              { "required": ["parent_id"] }
            ]
          },
          {
            "type": "object",
            "description": "change node fields; select with numbering or id",
            "properties": {
              "op": {
                "type": "string",
                "const": "trim"
              },
              "numbering": {
                "type": "string",
                "minLength": 1
              },
              "id": {
                "type": "string",
                "minLength": 1
              },
              "weight": {
                "type": "number",
                "description": "the new weight",
                "minimum": 0,
                "maximum": 1
              },
              "status": {
                "type": "string",
                "description": "the new status"
              }
            },
            "required": ["op"],
            "additionalProperties": false,
            "allOf": [
              {
                "anyOf": [
                  { "required": ["numbering"] },
                  { "required": ["id"] }
                ]
              },
              {
                "anyOf": [
                  { "required": ["weight"] },
                  { "required": ["status"] }
                ]
              }
            ]
          },
          {
            "type": "object",
            "description": "remove a goal node and its children; select with numbering or id",
            "properties": {
              "op": {
                "type": "string",
                "const": "prune"
              },
              "numbering": {
                "type": "string",
                "minLength": 1
              },
              "id": {
                "type": "string",
                "minLength": 1
              }
            },
            "required": ["op"],
            "additionalProperties": false,
            "anyOf": [
              { "required": ["numbering"] },
              { "required": ["id"] }
            ]
          }
        ]
      }
    })
}
