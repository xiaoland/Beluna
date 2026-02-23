use crate::{
    ai_gateway::types_chat::OutputMode,
    cortex::{
        cognition::{CognitionState, GoalTreePatchOp},
        error::{CortexError, filler_failed},
        helpers::{self, CognitionOrgan, HelperRuntime},
        prompts,
        testing::GoalTreePatchHelperRequest as TestGoalTreePatchRequest,
    },
};

type GoalTreePatchHelperOutput = Vec<GoalTreePatchOp>;

#[derive(Clone, Default)]
pub(crate) struct GoalTreePatchOutputHelper;

impl GoalTreePatchOutputHelper {
    pub(crate) async fn to_structured_output(
        &self,
        runtime: &impl HelperRuntime,
        cycle_id: u64,
        goal_tree_patch_section: &str,
        cognition_state: &CognitionState,
    ) -> Result<GoalTreePatchHelperOutput, CortexError> {
        let stage = CognitionOrgan::GoalTreePatch.stage();
        let user_partition_json =
            crate::cortex::helpers::goal_tree_input_helper::goal_tree_user_partition_json(
                &cognition_state.goal_tree.user_partition,
            );
        let input_payload = helpers::pretty_json(&serde_json::json!({
            "goal_tree_patch_section": goal_tree_patch_section,
            "current_user_partition_json": &user_partition_json,
        }));
        helpers::log_organ_input(cycle_id, stage, &input_payload);

        let output = if let Some(hooks) = runtime.hooks() {
            (hooks.goal_tree_patch_helper)(TestGoalTreePatchRequest {
                cycle_id,
                goal_tree_patch_section: goal_tree_patch_section.to_string(),
                cognition_state: cognition_state.clone(),
            })
            .await?
        } else {
            let prompt = prompts::build_goal_tree_patch_helper_prompt(
                goal_tree_patch_section,
                &user_partition_json,
            );
            let response = runtime
                .run_organ(
                    cycle_id,
                    CognitionOrgan::GoalTreePatch,
                    runtime.limits().max_sub_output_tokens,
                    prompts::goal_tree_patch_helper_system_prompt(),
                    prompt,
                    OutputMode::JsonSchema {
                        name: "goal_tree_patch_helper_output".to_string(),
                        schema: goal_tree_patch_ops_json_schema(),
                        strict: true,
                    },
                )
                .await?;
            parse_goal_tree_patch_helper_output(&response.output_text)
                .map_err(|err| filler_failed(err.to_string()))?
        };
        helpers::log_organ_output(cycle_id, stage, &helpers::pretty_json(&output));
        Ok(output)
    }
}

pub(crate) fn goal_tree_patch_ops_json_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "array",
        "items": {
            "oneOf": [
                {
                    "type": "object",
                    "properties": {
                        "op": { "type": "string", "const": "sprout" },
                        "numbering": { "type": "string" },
                        "node_id": { "type": "string" },
                        "summary": { "type": "string" },
                        "weight": { "type": "number" }
                    },
                    "required": ["op", "numbering", "node_id", "summary", "weight"],
                    "additionalProperties": false
                },
                {
                    "type": "object",
                    "properties": {
                        "op": { "type": "string", "const": "prune" },
                        "numbering": { "type": "string" }
                    },
                    "required": ["op", "numbering"],
                    "additionalProperties": false
                },
                {
                    "type": "object",
                    "properties": {
                        "op": { "type": "string", "const": "tilt" },
                        "numbering": { "type": "string" },
                        "weight": { "type": "number" }
                    },
                    "required": ["op", "numbering", "weight"],
                    "additionalProperties": false
                }
            ]
        }
    })
}

pub(crate) fn parse_goal_tree_patch_helper_output(
    text: &str,
) -> Result<GoalTreePatchHelperOutput, serde_json::Error> {
    serde_json::from_str::<GoalTreePatchHelperOutput>(text)
}
