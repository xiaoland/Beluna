use serde::{Deserialize, Serialize};

use crate::{
    ai_gateway::types_chat::OutputMode,
    cortex::{
        clamp::derive_act_instance_id,
        error::{CortexError, extractor_failed},
        helpers::{self, CognitionOrgan, HelperRuntime},
        prompts,
        testing::ActsHelperRequest as TestActsHelperRequest,
    },
    types::{Act, NeuralSignalDescriptor, NeuralSignalType, build_fq_neural_signal_id},
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct ActDraft {
    pub endpoint_id: String,
    pub fq_act_id: String,
    #[serde(default)]
    pub payload: serde_json::Value,
}

type ActsHelperOutput = Vec<ActDraft>;

#[derive(Clone, Default)]
pub(crate) struct ActsOutputHelper;

impl ActsOutputHelper {
    pub(crate) async fn to_structured_output(
        &self,
        runtime: &impl HelperRuntime,
        cycle_id: u64,
        acts_section: &str,
        act_descriptors: &[NeuralSignalDescriptor],
    ) -> Result<Vec<Act>, CortexError> {
        let stage = CognitionOrgan::Acts.stage();
        let input_payload = helpers::pretty_json(&serde_json::json!({
            "acts_section": acts_section,
            "act_descriptor_catalog": act_descriptors,
        }));
        helpers::log_organ_input(cycle_id, stage, &input_payload);

        let act_drafts = if let Some(hooks) = runtime.hooks() {
            let raw = (hooks.acts_helper)(TestActsHelperRequest {
                cycle_id,
                acts_section: acts_section.to_string(),
            })
            .await?;
            raw.into_iter()
                .map(|act| ActDraft {
                    endpoint_id: act.endpoint_id,
                    fq_act_id: act.fq_act_id,
                    payload: act.payload,
                })
                .collect()
        } else {
            let prompt = prompts::build_acts_helper_prompt(act_descriptors, acts_section);
            let response = runtime
                .run_organ(
                    cycle_id,
                    CognitionOrgan::Acts,
                    runtime.limits().max_sub_output_tokens,
                    prompts::acts_helper_system_prompt(),
                    prompt,
                    OutputMode::JsonSchema {
                        name: "acts_helper_output".to_string(),
                        schema: acts_json_schema(),
                        strict: true,
                    },
                )
                .await?;
            parse_acts_helper_output(&response.output_text)
                .map_err(|err| extractor_failed(err.to_string()))?
        };

        let acts = materialize_acts(cycle_id, act_drafts, act_descriptors);
        helpers::log_organ_output(cycle_id, stage, &helpers::pretty_json(&acts));
        Ok(acts)
    }
}

pub(crate) fn acts_json_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "array",
        "items": {
            "type": "object",
            "properties": {
                "endpoint_id": { "type": "string" },
                "fq_act_id": { "type": "string" },
                "payload": {}
            },
            "required": ["endpoint_id", "fq_act_id", "payload"],
            "additionalProperties": false
        }
    })
}

fn parse_acts_helper_output(text: &str) -> Result<ActsHelperOutput, serde_json::Error> {
    serde_json::from_str::<ActsHelperOutput>(text)
}

fn materialize_acts(
    cycle_id: u64,
    drafts: ActsHelperOutput,
    act_descriptors: &[NeuralSignalDescriptor],
) -> Vec<Act> {
    let mut acts = Vec::with_capacity(drafts.len());
    for draft in drafts {
        let Some(matched_descriptor) = act_descriptors.iter().find(|descriptor| {
            descriptor.r#type == NeuralSignalType::Act
                && descriptor_fq_act_id(descriptor) == draft.fq_act_id
        }) else {
            continue;
        };

        if matched_descriptor.endpoint_id != draft.endpoint_id {
            continue;
        }

        let payload = draft.payload;
        acts.push(Act {
            act_instance_id: derive_act_instance_id(
                cycle_id,
                &[],
                &draft.endpoint_id,
                &matched_descriptor.neural_signal_descriptor_id,
                &payload,
            ),
            endpoint_id: draft.endpoint_id,
            neural_signal_descriptor_id: matched_descriptor.neural_signal_descriptor_id.clone(),
            payload,
        });
    }
    acts
}

fn descriptor_fq_act_id(descriptor: &NeuralSignalDescriptor) -> String {
    build_fq_neural_signal_id(
        &descriptor.endpoint_id,
        &descriptor.neural_signal_descriptor_id,
    )
}
