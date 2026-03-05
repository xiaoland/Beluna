use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tokio::time::{Duration, timeout};

use crate::{
    ai_gateway::chat::OutputMode,
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
        deadline: Duration,
        acts_section: &str,
        act_descriptors: &[NeuralSignalDescriptor],
        sense_descriptors: &[NeuralSignalDescriptor],
    ) -> Vec<Act> {
        let stage = CognitionOrgan::Acts.stage();
        let input_payload = helpers::pretty_json(&serde_json::json!({
            "acts_section": acts_section,
            "act_descriptor_catalog": act_descriptors,
            "sense_descriptor_catalog": sense_descriptors,
        }));
        helpers::log_organ_input(cycle_id, stage, &input_payload);

        if let Some(drafts) = parse_direct_json_acts_if_valid(acts_section, act_descriptors) {
            let acts = materialize_acts(cycle_id, drafts, act_descriptors, sense_descriptors);
            helpers::log_organ_output(cycle_id, stage, &helpers::pretty_json(&acts));
            return acts;
        }

        let act_drafts_result = timeout(deadline, async {
            if let Some(hooks) = runtime.hooks() {
                let raw = (hooks.acts_helper)(TestActsHelperRequest {
                    cycle_id,
                    acts_section: acts_section.to_string(),
                })
                .await?;
                Ok::<ActsHelperOutput, CortexError>(
                    raw.into_iter()
                        .map(|act| ActDraft {
                            endpoint_id: act.endpoint_id,
                            fq_act_id: act.fq_act_id,
                            payload: act.payload,
                        })
                        .collect(),
                )
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
                    .map_err(|err| extractor_failed(err.to_string()))
            }
        })
        .await;

        match act_drafts_result {
            Ok(Ok(act_drafts)) => {
                let acts =
                    materialize_acts(cycle_id, act_drafts, act_descriptors, sense_descriptors);
                helpers::log_organ_output(cycle_id, stage, &helpers::pretty_json(&acts));
                acts
            }
            Ok(Err(err)) => {
                runtime.emit_stage_failed(cycle_id, stage);
                tracing::warn!(
                    target: "cortex",
                    cycle_id = cycle_id,
                    stage = stage,
                    error = %err,
                    "acts_helper_failed_fallback_empty"
                );
                let fallback = Vec::new();
                helpers::log_organ_output(cycle_id, stage, &helpers::pretty_json(&fallback));
                fallback
            }
            Err(_) => {
                runtime.emit_stage_failed(cycle_id, stage);
                tracing::warn!(
                    target: "cortex",
                    cycle_id = cycle_id,
                    stage = stage,
                    "acts_helper_timeout_fallback_empty"
                );
                let fallback = Vec::new();
                helpers::log_organ_output(cycle_id, stage, &helpers::pretty_json(&fallback));
                fallback
            }
        }
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

fn parse_direct_json_acts_if_valid(
    acts_section: &str,
    act_descriptors: &[NeuralSignalDescriptor],
) -> Option<ActsHelperOutput> {
    let parsed = serde_json::from_str::<serde_json::Value>(acts_section.trim()).ok()?;
    let drafts = serde_json::from_value::<ActsHelperOutput>(parsed).ok()?;
    if drafts
        .iter()
        .all(|draft| draft_payload_is_valid(draft, act_descriptors))
    {
        return Some(drafts);
    }
    None
}

fn draft_payload_is_valid(draft: &ActDraft, act_descriptors: &[NeuralSignalDescriptor]) -> bool {
    let Some(descriptor) = act_descriptors.iter().find(|descriptor| {
        descriptor.r#type == NeuralSignalType::Act
            && descriptor.endpoint_id == draft.endpoint_id
            && descriptor_fq_act_id(descriptor) == draft.fq_act_id
    }) else {
        return false;
    };
    payload_matches_schema(&draft.payload, &descriptor.payload_schema)
}

fn payload_matches_schema(payload: &serde_json::Value, schema: &serde_json::Value) -> bool {
    let Ok(compiled) = jsonschema::JSONSchema::compile(schema) else {
        return false;
    };
    compiled.validate(payload).is_ok()
}

fn materialize_acts(
    cycle_id: u64,
    drafts: ActsHelperOutput,
    act_descriptors: &[NeuralSignalDescriptor],
    sense_descriptors: &[NeuralSignalDescriptor],
) -> Vec<Act> {
    let endpoint_emitted_sense_catalog = build_endpoint_emitted_sense_catalog(sense_descriptors);
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
        let might_emit_sense_ids = endpoint_emitted_sense_catalog
            .get(&draft.endpoint_id)
            .cloned()
            .unwrap_or_default();
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
            might_emit_sense_ids,
            payload,
        });
    }
    acts
}

fn build_endpoint_emitted_sense_catalog(
    sense_descriptors: &[NeuralSignalDescriptor],
) -> HashMap<String, Vec<String>> {
    let mut catalog: HashMap<String, Vec<String>> = HashMap::new();
    for descriptor in sense_descriptors {
        let fq_sense_id = build_fq_neural_signal_id(
            &descriptor.endpoint_id,
            &descriptor.neural_signal_descriptor_id,
        );
        catalog
            .entry(descriptor.endpoint_id.clone())
            .or_default()
            .push(fq_sense_id);
    }
    for might_emit_sense_ids in catalog.values_mut() {
        might_emit_sense_ids.sort();
        might_emit_sense_ids.dedup();
    }
    catalog
}

fn descriptor_fq_act_id(descriptor: &NeuralSignalDescriptor) -> String {
    build_fq_neural_signal_id(
        &descriptor.endpoint_id,
        &descriptor.neural_signal_descriptor_id,
    )
}
