use serde::{Deserialize, Serialize};
use tokio::time::{Duration, timeout};

use crate::{
    ai_gateway::types_chat::OutputMode,
    cortex::{
        error::{CortexError, extractor_failed},
        helpers::{self, CognitionOrgan, HelperRuntime},
        prompts,
        testing::SenseHelperRequest as TestSenseHelperRequest,
    },
    types::{NeuralSignalDescriptor, Sense, build_fq_neural_signal_id},
};

#[derive(Clone, Default)]
pub(crate) struct SenseInputHelper;

#[derive(Debug, Clone, Default)]
pub(crate) struct SenseToolContext {
    entries: Vec<SenseToolContextEntry>,
}

impl SenseToolContext {
    pub(crate) fn from_inputs(
        senses: &[Sense],
        sense_descriptors: &[NeuralSignalDescriptor],
    ) -> Self {
        let sense_events = project_domain_sense_events(senses);
        if sense_events.is_empty() {
            return Self::default();
        }

        let descriptor_catalog = project_sense_descriptor_inputs(sense_descriptors);
        let mut entries = Vec::with_capacity(sense_events.len());
        for event in sense_events {
            let payload_schema = descriptor_catalog
                .iter()
                .find(|descriptor| descriptor.fq_sense_id == event.fq_sense_id)
                .map(|descriptor| descriptor.payload_schema.clone())
                .unwrap_or_else(|| serde_json::json!({}));
            entries.push(SenseToolContextEntry {
                sense_instance_id: event.sense_instance_id,
                fq_sense_id: event.fq_sense_id,
                payload: event.payload,
                payload_schema,
                original_size_in_bytes: event.original_size_in_bytes,
            });
        }
        Self { entries }
    }

    pub(crate) fn entries(&self) -> &[SenseToolContextEntry] {
        &self.entries
    }

    fn entry_by_id(&self, sense_instance_id: u64) -> Option<&SenseToolContextEntry> {
        self.entries
            .iter()
            .find(|entry| entry.sense_instance_id == sense_instance_id)
    }
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct SenseToolContextEntry {
    pub sense_instance_id: u64,
    pub fq_sense_id: String,
    pub payload: serde_json::Value,
    pub payload_schema: serde_json::Value,
    pub original_size_in_bytes: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct SenseSubAgentTask {
    pub sense_id: u64,
    pub instruction: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SensePostmanEnvelope {
    brief: String,
    original_size_in_bytes: usize,
    confidence_score: f64,
    omitted_features: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SenseSubAgentEnvelope {
    result: String,
    confidence_score: f64,
}

impl SenseInputHelper {
    pub(crate) async fn to_input_ir_section(
        &self,
        runtime: &impl HelperRuntime,
        cycle_id: u64,
        deadline: Duration,
        senses: &[Sense],
        sense_descriptors: &[NeuralSignalDescriptor],
    ) -> String {
        let stage = CognitionOrgan::Sense.stage();
        let context = SenseToolContext::from_inputs(senses, sense_descriptors);
        if context.entries().is_empty() {
            let input_payload = helpers::pretty_json(&serde_json::json!({
                "senses": senses,
                "sense_descriptors": sense_descriptors,
            }));
            helpers::log_organ_input(cycle_id, stage, &input_payload);
            let output = "[]".to_string();
            helpers::log_organ_output(cycle_id, stage, &output);
            return output;
        }

        let input_payload = helpers::pretty_json(&serde_json::json!({
            "sense_context": context.entries(),
            "sense_passthrough_max_bytes": runtime.limits().sense_passthrough_max_bytes,
        }));
        helpers::log_organ_input(cycle_id, stage, &input_payload);
        let output_result = timeout(deadline, async {
            if let Some(hooks) = runtime.hooks() {
                (hooks.sense_helper)(TestSenseHelperRequest {
                    cycle_id,
                    senses: senses.to_vec(),
                    sense_descriptors: sense_descriptors.to_vec(),
                })
                .await
            } else {
                self.build_with_organ(
                    runtime,
                    cycle_id,
                    &context,
                    runtime.limits().sense_passthrough_max_bytes,
                )
                .await
            }
        })
        .await;
        match output_result {
            Ok(Ok(output)) if !output.trim().is_empty() => {
                helpers::log_organ_output(cycle_id, stage, &output);
                output
            }
            Ok(Ok(_)) => {
                let fallback = fallback_senses_section(
                    senses,
                    sense_descriptors,
                    runtime.limits().sense_passthrough_max_bytes,
                );
                helpers::log_organ_output(cycle_id, stage, &fallback);
                fallback
            }
            Ok(Err(err)) => {
                runtime.emit_stage_failed(cycle_id, stage);
                tracing::warn!(
                    target: "cortex",
                    cycle_id = cycle_id,
                    stage = stage,
                    error = %err,
                    "sense_helper_failed_fallback_raw"
                );
                let fallback = fallback_senses_section(
                    senses,
                    sense_descriptors,
                    runtime.limits().sense_passthrough_max_bytes,
                );
                helpers::log_organ_output(cycle_id, stage, &fallback);
                fallback
            }
            Err(_) => {
                runtime.emit_stage_failed(cycle_id, stage);
                tracing::warn!(
                    target: "cortex",
                    cycle_id = cycle_id,
                    stage = stage,
                    "sense_helper_timeout_fallback_raw"
                );
                let fallback = fallback_senses_section(
                    senses,
                    sense_descriptors,
                    runtime.limits().sense_passthrough_max_bytes,
                );
                helpers::log_organ_output(cycle_id, stage, &fallback);
                fallback
            }
        }
    }

    async fn build_with_organ(
        &self,
        runtime: &impl HelperRuntime,
        cycle_id: u64,
        context: &SenseToolContext,
        sense_passthrough_max_bytes: usize,
    ) -> Result<String, CortexError> {
        let mut entries = Vec::with_capacity(context.entries().len());
        for entry in context.entries() {
            if should_passthrough(entry.original_size_in_bytes, sense_passthrough_max_bytes) {
                entries.push(wrap_passthrough_entry(entry));
                continue;
            }

            let envelope = self
                .build_postman_envelope(runtime, cycle_id, entry)
                .await
                .unwrap_or_else(|_| SensePostmanEnvelope {
                    brief:
                        "Payload omitted by fallback; call expand-sense-raw for complete details."
                            .to_string(),
                    original_size_in_bytes: entry.original_size_in_bytes,
                    confidence_score: 0.0,
                    omitted_features: vec!["payload".to_string()],
                });
            entries.push(wrap_postman_envelope_entry(entry, &envelope));
        }

        Ok(entries.join("\n"))
    }

    async fn build_postman_envelope(
        &self,
        runtime: &impl HelperRuntime,
        cycle_id: u64,
        entry: &SenseToolContextEntry,
    ) -> Result<SensePostmanEnvelope, CortexError> {
        let payload_json =
            serde_json::to_string_pretty(&entry.payload).unwrap_or_else(|_| "{}".to_string());
        let payload_schema_json = serde_json::to_string_pretty(&entry.payload_schema)
            .unwrap_or_else(|_| "{}".to_string());
        let prompt = prompts::build_sense_postman_envelope_prompt(
            &payload_json,
            &payload_schema_json,
            entry.original_size_in_bytes,
        );
        let response = runtime
            .run_organ(
                cycle_id,
                CognitionOrgan::Sense,
                runtime.limits().max_sub_output_tokens,
                prompts::sense_helper_system_prompt(),
                prompt,
                OutputMode::JsonSchema {
                    name: "sense_postman_envelope".to_string(),
                    schema: sense_postman_envelope_json_schema(),
                    strict: true,
                },
            )
            .await?;
        let mut envelope: SensePostmanEnvelope = serde_json::from_str(&response.output_text)
            .map_err(|err| extractor_failed(err.to_string()))?;
        envelope.original_size_in_bytes = entry.original_size_in_bytes;
        envelope.confidence_score = envelope.confidence_score.clamp(0.0, 1.0);
        Ok(envelope)
    }
}

pub(crate) fn fallback_senses_section(
    senses: &[Sense],
    sense_descriptors: &[NeuralSignalDescriptor],
    sense_passthrough_max_bytes: usize,
) -> String {
    let context = SenseToolContext::from_inputs(senses, sense_descriptors);
    if context.entries().is_empty() {
        return "[]".to_string();
    }

    let mut entries = Vec::with_capacity(context.entries().len());
    for entry in context.entries() {
        if should_passthrough(entry.original_size_in_bytes, sense_passthrough_max_bytes) {
            entries.push(wrap_passthrough_entry(entry));
            continue;
        }

        let envelope = SensePostmanEnvelope {
            brief: "Payload omitted by deterministic fallback; call expand-sense-raw for complete details."
                .to_string(),
            original_size_in_bytes: entry.original_size_in_bytes,
            confidence_score: 0.0,
            omitted_features: vec!["payload".to_string()],
        };
        entries.push(wrap_postman_envelope_entry(entry, &envelope));
    }

    entries.join("\n")
}

pub(crate) fn expand_sense_raw(context: &SenseToolContext, sense_ids: &[u64]) -> serde_json::Value {
    let mut items = Vec::new();
    let mut not_found_sense_ids = Vec::new();

    for sense_id in sense_ids {
        if let Some(entry) = context.entry_by_id(*sense_id) {
            items.push(serde_json::json!({
                "sense_id": entry.sense_instance_id,
                "fq_sense_id": entry.fq_sense_id,
                "payload": entry.payload,
                "payload_schema": entry.payload_schema,
                "original_size_in_bytes": entry.original_size_in_bytes,
            }));
        } else {
            not_found_sense_ids.push(*sense_id);
        }
    }

    serde_json::json!({
        "items": items,
        "not_found_sense_ids": not_found_sense_ids,
    })
}

pub(crate) async fn expand_sense_with_sub_agent(
    runtime: &impl HelperRuntime,
    cycle_id: u64,
    context: &SenseToolContext,
    tasks: &[SenseSubAgentTask],
) -> Result<serde_json::Value, CortexError> {
    let mut results = Vec::new();
    let mut not_found_sense_ids = Vec::new();

    for task in tasks {
        let Some(entry) = context.entry_by_id(task.sense_id) else {
            not_found_sense_ids.push(task.sense_id);
            continue;
        };

        let payload_json =
            serde_json::to_string_pretty(&entry.payload).unwrap_or_else(|_| "{}".to_string());
        let payload_schema_json = serde_json::to_string_pretty(&entry.payload_schema)
            .unwrap_or_else(|_| "{}".to_string());
        let prompt = prompts::build_sense_sub_agent_prompt(
            &payload_json,
            &payload_schema_json,
            &task.instruction,
        );
        let response = runtime
            .run_organ(
                cycle_id,
                CognitionOrgan::Sense,
                runtime.limits().max_sub_output_tokens,
                prompts::sense_sub_agent_system_prompt(),
                prompt,
                OutputMode::JsonSchema {
                    name: "sense_sub_agent_output".to_string(),
                    schema: sense_sub_agent_output_json_schema(),
                    strict: true,
                },
            )
            .await?;
        let mut envelope: SenseSubAgentEnvelope = serde_json::from_str(&response.output_text)
            .map_err(|err| extractor_failed(err.to_string()))?;
        envelope.confidence_score = envelope.confidence_score.clamp(0.0, 1.0);

        results.push(serde_json::json!({
            "sense_id": entry.sense_instance_id,
            "fq_sense_id": entry.fq_sense_id,
            "instruction": task.instruction,
            "result": envelope.result,
            "confidence_score": envelope.confidence_score,
        }));
    }

    Ok(serde_json::json!({
        "results": results,
        "not_found_sense_ids": not_found_sense_ids,
    }))
}

#[derive(Debug, Clone, Serialize)]
struct SenseInputEvent {
    sense_instance_id: u64,
    fq_sense_id: String,
    payload: serde_json::Value,
    original_size_in_bytes: usize,
}

#[derive(Debug, Clone, Serialize)]
struct SenseDescriptorInput {
    fq_sense_id: String,
    payload_schema: serde_json::Value,
}

fn project_domain_sense_events(senses: &[Sense]) -> Vec<SenseInputEvent> {
    let mut next_sense_instance_id = 1_u64;
    let mut events = Vec::new();
    for sense in senses {
        let Sense::Domain(domain) = sense else {
            continue;
        };
        let payload_bytes = serde_json::to_vec(&domain.payload)
            .map(|bytes| bytes.len())
            .unwrap_or(0);
        events.push(SenseInputEvent {
            sense_instance_id: next_sense_instance_id,
            fq_sense_id: build_fq_neural_signal_id(
                &domain.endpoint_id,
                &domain.neural_signal_descriptor_id,
            ),
            payload: domain.payload.clone(),
            original_size_in_bytes: payload_bytes,
        });
        next_sense_instance_id += 1;
    }
    events
}

fn project_sense_descriptor_inputs(
    sense_descriptors: &[NeuralSignalDescriptor],
) -> Vec<SenseDescriptorInput> {
    sense_descriptors
        .iter()
        .map(|descriptor| SenseDescriptorInput {
            fq_sense_id: build_fq_neural_signal_id(
                &descriptor.endpoint_id,
                &descriptor.neural_signal_descriptor_id,
            ),
            payload_schema: descriptor.payload_schema.clone(),
        })
        .collect()
}

fn should_passthrough(original_size_in_bytes: usize, sense_passthrough_max_bytes: usize) -> bool {
    original_size_in_bytes <= sense_passthrough_max_bytes
}

fn wrap_passthrough_entry(entry: &SenseToolContextEntry) -> String {
    let payload_json = serde_json::to_string(&entry.payload).unwrap_or_else(|_| "{}".to_string());
    format!(
        "<somatic-sense sense-instance-id=\"{}\" fq-somatic-sense-id=\"{}\" body=\"full-payload\">\n{}\n</somatic-sense>",
        entry.sense_instance_id,
        escape_xml_attr(&entry.fq_sense_id),
        payload_json
    )
}

fn wrap_postman_envelope_entry(
    entry: &SenseToolContextEntry,
    envelope: &SensePostmanEnvelope,
) -> String {
    let envelope_json = serde_json::to_string_pretty(envelope).unwrap_or_else(|_| "{}".to_string());
    format!(
        "<somatic-sense sense-instance-id=\"{}\" fq-somatic-sense-id=\"{}\" body-format=\"postman-envelope-json\">\n{}\n</somatic-sense>",
        entry.sense_instance_id,
        escape_xml_attr(&entry.fq_sense_id),
        envelope_json
    )
}

fn sense_postman_envelope_json_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "brief": { "type": "string" },
            "original_size_in_bytes": { "type": "integer", "minimum": 0 },
            "confidence_score": { "type": "number", "minimum": 0, "maximum": 1 },
            "omitted_features": {
                "type": "array",
                "items": { "type": "string" }
            }
        },
        "required": ["brief", "original_size_in_bytes", "confidence_score", "omitted_features"],
        "additionalProperties": false
    })
}

fn sense_sub_agent_output_json_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "result": { "type": "string" },
            "confidence_score": { "type": "number", "minimum": 0, "maximum": 1 }
        },
        "required": ["result", "confidence_score"],
        "additionalProperties": false
    })
}

fn escape_xml_attr(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
