use serde::{Deserialize, Serialize};
use tokio::time::Duration;

use crate::{
    ai_gateway::chat::OutputMode,
    cortex::{
        error::{CortexError, extractor_failed},
        helpers::{self, CognitionOrgan, HelperRuntime},
        prompts,
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
                sense_ref_id: event.sense_ref_id,
                fq_sense_id: event.fq_sense_id,
                payload: event.payload,
                payload_schema,
                original_size_in_bytes: event.original_size_in_bytes,
                weight: event.weight,
                act_instance_id: event.act_instance_id,
            });
        }
        Self { entries }
    }

    pub(crate) fn entries(&self) -> &[SenseToolContextEntry] {
        &self.entries
    }

    fn entry_by_ref_id(&self, sense_ref_id: &str) -> Option<&SenseToolContextEntry> {
        self.entries
            .iter()
            .find(|entry| entry.sense_ref_id == sense_ref_id)
    }
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct SenseToolContextEntry {
    pub sense_instance_id: u64,
    pub sense_ref_id: String,
    pub fq_sense_id: String,
    pub payload: String,
    pub payload_schema: serde_json::Value,
    pub original_size_in_bytes: usize,
    pub weight: f64,
    pub act_instance_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct SenseSubAgentTask {
    pub sense_id: String,
    #[serde(default)]
    pub instruction: Option<String>,
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
        _deadline: Duration,
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
        let output = render_sense_lines(&context, runtime.limits().sense_passthrough_max_bytes);
        helpers::log_organ_output(cycle_id, stage, &output);
        output
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
                        "Payload omitted by fallback; call expand-senses(mode=raw) for complete details."
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
        let payload_text = entry.payload.clone();
        let payload_schema_json = serde_json::to_string_pretty(&entry.payload_schema)
            .unwrap_or_else(|_| "{}".to_string());
        let prompt = prompts::build_sense_postman_envelope_prompt(
            &payload_text,
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
    render_sense_lines(&context, sense_passthrough_max_bytes)
}

pub(crate) fn expand_sense_raw(
    context: &SenseToolContext,
    sense_ids: &[String],
) -> serde_json::Value {
    let mut items = Vec::new();
    let mut not_found_sense_ids = Vec::new();

    for sense_id in sense_ids {
        if let Some(entry) = context.entry_by_ref_id(sense_id) {
            items.push(serde_json::json!({
                "sense_id": entry.sense_ref_id,
                "monotonic_internal_sense_id": entry.sense_instance_id,
                "fq_sense_id": entry.fq_sense_id,
                "payload": entry.payload,
                "payload_schema": entry.payload_schema,
                "original_size_in_bytes": entry.original_size_in_bytes,
            }));
        } else {
            not_found_sense_ids.push(sense_id.clone());
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
        let Some(entry) = context.entry_by_ref_id(&task.sense_id) else {
            not_found_sense_ids.push(task.sense_id.clone());
            continue;
        };

        let instruction = task.instruction.clone().unwrap_or_default();
        let payload_text = entry.payload.clone();
        let payload_schema_json = serde_json::to_string_pretty(&entry.payload_schema)
            .unwrap_or_else(|_| "{}".to_string());
        let prompt = prompts::build_sense_sub_agent_prompt(
            &payload_text,
            &payload_schema_json,
            &instruction,
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
            "sense_id": entry.sense_ref_id,
            "fq_sense_id": entry.fq_sense_id,
            "instruction": instruction,
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
    sense_ref_id: String,
    fq_sense_id: String,
    payload: String,
    original_size_in_bytes: usize,
    weight: f64,
    act_instance_id: Option<String>,
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
        let payload_bytes = sense.payload.as_bytes().len();
        let fq_sense_id =
            build_fq_neural_signal_id(&sense.endpoint_id, &sense.neural_signal_descriptor_id);
        events.push(SenseInputEvent {
            sense_instance_id: next_sense_instance_id,
            sense_ref_id: format!("{}. {}", next_sense_instance_id, fq_sense_id),
            fq_sense_id,
            payload: sense.payload.clone(),
            original_size_in_bytes: payload_bytes,
            weight: sense.weight,
            act_instance_id: sense.act_instance_id.clone(),
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
    let payload = truncate_payload(&entry.payload, usize::MAX);
    format!(
        "- {}: {}; {}",
        entry.sense_ref_id,
        render_metadata(entry, entry.original_size_in_bytes),
        payload
    )
}

fn wrap_postman_envelope_entry(
    entry: &SenseToolContextEntry,
    envelope: &SensePostmanEnvelope,
) -> String {
    let envelope_json = serde_json::to_string_pretty(envelope).unwrap_or_else(|_| "{}".to_string());
    format!(
        "- {}: {}; {}",
        entry.sense_ref_id,
        render_metadata(entry, envelope.original_size_in_bytes),
        envelope_json
    )
}

fn render_sense_lines(context: &SenseToolContext, sense_passthrough_max_bytes: usize) -> String {
    context
        .entries()
        .iter()
        .map(|entry| {
            let payload = truncate_payload(&entry.payload, sense_passthrough_max_bytes);
            format!(
                "- {}: {}; {}",
                entry.sense_ref_id,
                render_metadata(entry, entry.original_size_in_bytes),
                payload
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_metadata(entry: &SenseToolContextEntry, original_size_in_bytes: usize) -> String {
    let mut kv = Vec::with_capacity(3);
    kv.push(format!("weight={:.3}", entry.weight));
    if let Some(act_instance_id) = &entry.act_instance_id {
        kv.push(format!("act_instance_id={act_instance_id}"));
    }
    kv.push(format!("original_size_in_bytes={original_size_in_bytes}"));
    kv.join(",")
}

fn truncate_payload(payload: &str, max_bytes: usize) -> String {
    if payload.len() <= max_bytes {
        return payload.to_string();
    }
    if max_bytes == 0 {
        return "...(truncated)".to_string();
    }

    let mut end = max_bytes.min(payload.len());
    while end > 0 && !payload.is_char_boundary(end) {
        end -= 1;
    }
    if end == 0 {
        return "...(truncated)".to_string();
    }
    format!("{}...(truncated)", &payload[..end])
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

fn escape_xml_text(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
