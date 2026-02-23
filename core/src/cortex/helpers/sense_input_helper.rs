use serde::Serialize;

use crate::{
    cortex::{
        error::CortexError,
        helpers::{self, CognitionOrgan, HelperRuntime},
        prompts,
        testing::SenseHelperRequest as TestSenseHelperRequest,
    },
    types::{NeuralSignalDescriptor, Sense, build_fq_neural_signal_id},
};

#[derive(Clone, Default)]
pub(crate) struct SenseInputHelper;

impl SenseInputHelper {
    pub(crate) async fn to_input_ir_section(
        &self,
        runtime: &impl HelperRuntime,
        cycle_id: u64,
        senses: &[Sense],
        sense_descriptors: &[NeuralSignalDescriptor],
    ) -> Result<String, CortexError> {
        let stage = CognitionOrgan::Sense.stage();
        let sense_events = project_domain_sense_events(senses);
        if sense_events.is_empty() {
            let input_payload = helpers::pretty_json(&serde_json::json!({
                "senses": senses,
                "sense_descriptors": sense_descriptors,
            }));
            helpers::log_organ_input(cycle_id, stage, &input_payload);
            let output = fallback_senses_section(senses, sense_descriptors);
            helpers::log_organ_output(cycle_id, stage, &output);
            return Ok(output);
        }

        if let Some(hooks) = runtime.hooks() {
            let input_payload = helpers::pretty_json(&serde_json::json!({
                "senses": senses,
                "sense_descriptors": sense_descriptors,
            }));
            helpers::log_organ_input(cycle_id, stage, &input_payload);
            let output = (hooks.sense_helper)(TestSenseHelperRequest {
                cycle_id,
                senses: senses.to_vec(),
                sense_descriptors: sense_descriptors.to_vec(),
            })
            .await?;
            helpers::log_organ_output(cycle_id, stage, &output);
            return Ok(output);
        }

        let sense_descriptor_catalog = project_sense_descriptor_inputs(sense_descriptors);
        let input_payload = format!(
            "sense_events:\n{}\n\nsense_descriptor_catalog:\n{}",
            helpers::pretty_json(&sense_events),
            helpers::pretty_json(&sense_descriptor_catalog)
        );
        helpers::log_organ_input(cycle_id, stage, &input_payload);
        let output = self
            .build_with_organ(runtime, cycle_id, &sense_events, &sense_descriptor_catalog)
            .await?;
        helpers::log_organ_output(cycle_id, stage, &output);
        Ok(output)
    }

    async fn build_with_organ(
        &self,
        runtime: &impl HelperRuntime,
        cycle_id: u64,
        sense_events: &[SenseInputEvent],
        sense_descriptor_catalog: &[SenseDescriptorInput],
    ) -> Result<String, CortexError> {
        let mut entries = Vec::with_capacity(sense_events.len());
        for sense_event in sense_events {
            let payload_json = serde_json::to_string_pretty(&sense_event.payload)
                .unwrap_or_else(|_| "{}".to_string());
            let payload_schema_json =
                payload_schema_json_for_sense(sense_event, sense_descriptor_catalog);
            let prompt = prompts::build_sense_helper_prompt(&payload_json, &payload_schema_json);
            let markdown = runtime
                .run_text_organ_with_system(
                    cycle_id,
                    CognitionOrgan::Sense,
                    runtime.limits().max_sub_output_tokens,
                    prompts::sense_helper_system_prompt(),
                    prompt,
                )
                .await?;
            entries.push(wrap_sense_entry(sense_event, markdown.trim()));
        }
        Ok(entries.join("\n"))
    }
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct SenseInputEvent {
    pub sense_instance_id: String,
    pub endpoint_id: String,
    pub neural_signal_descriptor_id: String,
    pub fq_sense_id: String,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct SenseDescriptorInput {
    pub endpoint_id: String,
    pub neural_signal_descriptor_id: String,
    pub fq_sense_id: String,
    pub payload_schema: serde_json::Value,
}

fn project_domain_sense_events(senses: &[Sense]) -> Vec<SenseInputEvent> {
    senses
        .iter()
        .filter_map(|sense| match sense {
            Sense::Domain(domain) => Some(SenseInputEvent {
                sense_instance_id: domain.sense_instance_id.clone(),
                endpoint_id: domain.endpoint_id.clone(),
                neural_signal_descriptor_id: domain.neural_signal_descriptor_id.clone(),
                fq_sense_id: build_fq_neural_signal_id(
                    &domain.endpoint_id,
                    &domain.neural_signal_descriptor_id,
                ),
                payload: domain.payload.clone(),
            }),
            _ => None,
        })
        .collect()
}

fn project_sense_descriptor_inputs(
    sense_descriptors: &[NeuralSignalDescriptor],
) -> Vec<SenseDescriptorInput> {
    sense_descriptors
        .iter()
        .map(|descriptor| SenseDescriptorInput {
            endpoint_id: descriptor.endpoint_id.clone(),
            neural_signal_descriptor_id: descriptor.neural_signal_descriptor_id.clone(),
            fq_sense_id: build_fq_neural_signal_id(
                &descriptor.endpoint_id,
                &descriptor.neural_signal_descriptor_id,
            ),
            payload_schema: descriptor.payload_schema.clone(),
        })
        .collect()
}

pub(crate) fn fallback_senses_section(
    senses: &[Sense],
    sense_descriptors: &[NeuralSignalDescriptor],
) -> String {
    let projected_senses = project_domain_sense_events(senses);
    if projected_senses.is_empty() {
        return "[]".to_string();
    }

    let projected_catalog = project_sense_descriptor_inputs(sense_descriptors);
    let mut entries = Vec::with_capacity(projected_senses.len());
    for sense_event in &projected_senses {
        let payload_json =
            serde_json::to_string_pretty(&sense_event.payload).unwrap_or_else(|_| "{}".to_string());
        let payload_schema_json = projected_catalog
            .iter()
            .find(|descriptor| descriptor.fq_sense_id == sense_event.fq_sense_id)
            .and_then(|descriptor| serde_json::to_string_pretty(&descriptor.payload_schema).ok())
            .unwrap_or_else(|| "{}".to_string());
        entries.push(format!(
            "<sense fq-sense-id=\"{}\">\n## payload\n{}\n\n## payload-schema\n{}\n</sense>",
            escape_xml_attr(&sense_event.fq_sense_id),
            payload_json,
            payload_schema_json
        ));
    }

    entries.join("\n")
}

fn payload_schema_json_for_sense(
    sense_event: &SenseInputEvent,
    sense_catalog: &[SenseDescriptorInput],
) -> String {
    sense_catalog
        .iter()
        .find(|descriptor| descriptor.fq_sense_id == sense_event.fq_sense_id)
        .and_then(|descriptor| serde_json::to_string_pretty(&descriptor.payload_schema).ok())
        .unwrap_or_else(|| "{}".to_string())
}

fn wrap_sense_entry(sense_event: &SenseInputEvent, markdown: &str) -> String {
    format!(
        "<sense fq-sense-id=\"{}\">\n{}\n</sense>",
        escape_xml_attr(&sense_event.fq_sense_id),
        markdown.trim()
    )
}

fn escape_xml_attr(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
