use tokio::time::Duration;

mod model;

pub use model::{GoalForest, GoalNode};

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
    "Try to plan some trees, and patch them with complete GoalNode[] replacements."
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
        let current_goal_forest_ascii = if goal_forest_nodes.is_empty() {
            goal_forest_empty_one_shot().to_string()
        } else {
            goal_forest_ascii(goal_forest_nodes)
        };
        let current_goal_forest_json = goal_forest_json(goal_forest_nodes);

        let input_payload = helpers::pretty_json(&serde_json::json!({
            "current_goal_forest_ascii": current_goal_forest_ascii,
            "current_goal_forest_json": current_goal_forest_json,
            "patch_instructions": instructions,
        }));
        helpers::log_organ_input(cycle_id, stage, &input_payload);

        let prompt = prompts::build_goal_forest_patch_sub_agent_prompt(
            &current_goal_forest_ascii,
            &current_goal_forest_json,
            instructions,
        );
        let response = runtime
            .run_organ(
                cycle_id,
                CognitionOrgan::GoalForest,
                runtime.limits().max_sub_output_tokens,
                prompts::goal_forest_patch_sub_agent_system_prompt(),
                prompt,
                OutputMode::JsonSchema {
                    name: "goal_forest_nodes".to_string(),
                    schema: goal_forest_nodes_json_schema(),
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

        let candidate_nodes = match parse_goal_forest_nodes(&sub_agent_output) {
            Ok(nodes) => nodes,
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

        let previous_node_count = count_total_nodes(goal_forest_nodes);
        let replaced_node_count = count_total_nodes(&candidate_nodes);
        *goal_forest_nodes = candidate_nodes;

        let updated_goal_forest = if goal_forest_nodes.is_empty() {
            goal_forest_empty_one_shot().to_string()
        } else {
            goal_forest_ascii(goal_forest_nodes)
        };
        let output = serde_json::json!({
            "previous_node_count": previous_node_count,
            "replaced_node_count": replaced_node_count,
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
    let mut lines = Vec::new();
    for root in goal_forest_nodes {
        append_goal_node_lines(&mut lines, root, 0, &[]);
    }
    lines.join("\n")
}

fn append_goal_node_lines(lines: &mut Vec<String>, node: &GoalNode, depth: usize, path: &[usize]) {
    let prefix = if depth == 0 {
        "+-- ".to_string()
    } else {
        format!("{}|-- ", "    ".repeat(depth))
    };

    if path.is_empty() {
        lines.push(format!(
            "{prefix}[{}] (w={:.2}) id={} :: {}",
            node.status, node.weight, node.id, node.summary
        ));
    } else {
        let numbering = path
            .iter()
            .map(|segment| segment.to_string())
            .collect::<Vec<_>>()
            .join(".");
        lines.push(format!(
            "{prefix}{} [{}] (w={:.2}) id={} :: {}",
            numbering, node.status, node.weight, node.id, node.summary
        ));
    }

    for (index, child) in node.children.iter().enumerate() {
        let mut child_path = path.to_vec();
        child_path.push(index + 1);
        append_goal_node_lines(lines, child, depth + 1, &child_path);
    }
}

pub(crate) fn goal_forest_empty_one_shot() -> &'static str {
    GOAL_FOREST_EMPTY_FALLBACK
}

fn parse_goal_forest_nodes(arguments_json: &str) -> Result<Vec<GoalNode>, String> {
    let value: serde_json::Value =
        serde_json::from_str(arguments_json).map_err(|err| err.to_string())?;
    let nodes_value = match value {
        serde_json::Value::Array(items) => serde_json::Value::Array(items),
        serde_json::Value::Object(mut map) => {
            if let Some(nodes_value) = map.remove("nodes") {
                nodes_value
            } else {
                let keys = map.keys().cloned().collect::<Vec<_>>().join(",");
                return Err(format!(
                    "expected GoalNode[] or object with 'nodes', got object with keys [{}]",
                    keys
                ));
            }
        }
        other => {
            return Err(format!(
                "expected GoalNode[] or object with 'nodes', got {}",
                json_type_name(&other)
            ));
        }
    };

    serde_json::from_value::<Vec<GoalNode>>(nodes_value).map_err(|err| err.to_string())
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

fn goal_forest_nodes_json_schema() -> serde_json::Value {
    serde_json::json!({
      "type": "array",
      "minItems": 0,
      "description": "complete replacement of goal forest roots as GoalNode[]",
      "items": {
        "$ref": "#/$defs/goalNode"
      },
      "$defs": {
        "goalNode": {
          "type": "object",
          "properties": {
            "status": {
              "type": "string",
              "minLength": 1
            },
            "weight": {
              "type": "number",
              "minimum": 0,
              "maximum": 1
            },
            "id": {
              "type": "string",
              "minLength": 1
            },
            "summary": {
              "type": "string",
              "minLength": 1
            },
            "children": {
              "type": "array",
              "items": {
                "$ref": "#/$defs/goalNode"
              },
              "default": []
            }
          },
          "required": ["status", "weight", "id", "summary", "children"],
          "additionalProperties": false
        }
      }
    })
}

fn count_total_nodes(nodes: &[GoalNode]) -> usize {
    nodes
        .iter()
        .map(|node| 1 + count_total_nodes(&node.children))
        .sum()
}
