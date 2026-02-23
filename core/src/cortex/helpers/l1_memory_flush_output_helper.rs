use crate::{
    ai_gateway::types_chat::OutputMode,
    cortex::{
        cognition::CognitionState,
        error::{CortexError, filler_failed},
        helpers::{self, CognitionOrgan, HelperRuntime},
        prompts,
        testing::L1MemoryFlushHelperRequest as TestL1MemoryFlushRequest,
    },
};

type L1MemoryFlushHelperOutput = Vec<String>;

#[derive(Clone, Default)]
pub(crate) struct L1MemoryFlushOutputHelper;

impl L1MemoryFlushOutputHelper {
    pub(crate) async fn to_structured_output(
        &self,
        runtime: &impl HelperRuntime,
        cycle_id: u64,
        l1_memory_flush_section: &str,
        cognition_state: &CognitionState,
    ) -> Result<L1MemoryFlushHelperOutput, CortexError> {
        let stage = CognitionOrgan::L1MemoryFlush.stage();
        let l1_memory_json = crate::cortex::helpers::goal_tree_input_helper::l1_memory_json(
            &cognition_state.l1_memory,
        );
        let input_payload = helpers::pretty_json(&serde_json::json!({
            "l1_memory_flush_section": l1_memory_flush_section,
            "current_l1_memory_json": &l1_memory_json,
        }));
        helpers::log_organ_input(cycle_id, stage, &input_payload);

        let output = if let Some(hooks) = runtime.hooks() {
            (hooks.l1_memory_flush_helper)(TestL1MemoryFlushRequest {
                cycle_id,
                l1_memory_flush_section: l1_memory_flush_section.to_string(),
                cognition_state: cognition_state.clone(),
            })
            .await?
        } else {
            let prompt = prompts::build_l1_memory_flush_helper_prompt(
                l1_memory_flush_section,
                &l1_memory_json,
            );
            let response = runtime
                .run_organ(
                    cycle_id,
                    CognitionOrgan::L1MemoryFlush,
                    runtime.limits().max_sub_output_tokens,
                    prompts::l1_memory_flush_helper_system_prompt(),
                    prompt,
                    OutputMode::JsonSchema {
                        name: "l1_memory_flush_helper_output".to_string(),
                        schema: l1_memory_flush_json_schema(),
                        strict: true,
                    },
                )
                .await?;
            parse_l1_memory_flush_helper_output(&response.output_text)
                .map_err(|err| filler_failed(err.to_string()))?
        };
        helpers::log_organ_output(cycle_id, stage, &helpers::pretty_json(&output));
        Ok(output)
    }
}

pub(crate) fn l1_memory_flush_json_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "array",
        "items": { "type": "string" }
    })
}

pub(crate) fn parse_l1_memory_flush_helper_output(
    text: &str,
) -> Result<L1MemoryFlushHelperOutput, serde_json::Error> {
    serde_json::from_str::<L1MemoryFlushHelperOutput>(text)
}
