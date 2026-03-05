use std::sync::atomic::{AtomicU64, Ordering};

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

static NEXT_INTERNAL_SENSE_ID: AtomicU64 = AtomicU64::new(1);

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
                endpoint_id: event.endpoint_id,
                sense_id: event.sense_id,
                fq_sense_id: event.fq_sense_id,
                payload: event.payload,
                payload_schema,
                weight: event.weight,
            });
        }
        Self { entries }
    }

    pub(crate) fn entries(&self) -> &[SenseToolContextEntry] {
        &self.entries
    }

    pub(crate) fn merged(prior: &Self, incoming: &Self) -> Self {
        if prior.entries.is_empty() {
            return incoming.clone();
        }
        if incoming.entries.is_empty() {
            return prior.clone();
        }

        let mut merged = Vec::with_capacity(prior.entries.len() + incoming.entries.len());
        merged.extend(prior.entries.iter().cloned());
        merged.extend(incoming.entries.iter().cloned());
        Self { entries: merged }
    }

    fn entry_by_ref_id(&self, sense_ref_id: &str) -> Option<&SenseToolContextEntry> {
        let normalized = normalize_sense_ref_id(sense_ref_id);
        self.entries
            .iter()
            .find(|entry| entry.sense_ref_id == normalized.as_str())
    }
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct SenseToolContextEntry {
    pub sense_instance_id: u64,
    pub sense_ref_id: String,
    pub endpoint_id: String,
    pub sense_id: String,
    pub fq_sense_id: String,
    pub payload: String,
    pub payload_schema: serde_json::Value,
    pub weight: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct SenseSubAgentTask {
    pub sense_id: String,
    #[serde(default)]
    pub instruction: Option<String>,
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
        let context = SenseToolContext::from_inputs(senses, sense_descriptors);
        self.to_input_ir_section_from_context(
            cycle_id,
            &context,
            runtime.limits().sense_passthrough_max_bytes,
        )
        .await
    }

    pub(crate) async fn to_input_ir_section_from_context(
        &self,
        cycle_id: u64,
        context: &SenseToolContext,
        sense_passthrough_max_bytes: usize,
    ) -> String {
        let stage = CognitionOrgan::Sense.stage();
        if context.entries().is_empty() {
            let input_payload = helpers::pretty_json(&serde_json::json!({
                "sense_context": [],
                "sense_passthrough_max_bytes": sense_passthrough_max_bytes,
            }));
            helpers::log_organ_input(cycle_id, stage, &input_payload);
            let output = "[]".to_string();
            helpers::log_organ_output(cycle_id, stage, &output);
            return output;
        }

        let input_payload = helpers::pretty_json(&serde_json::json!({
            "sense_context": context.entries(),
            "sense_passthrough_max_bytes": sense_passthrough_max_bytes,
        }));
        helpers::log_organ_input(cycle_id, stage, &input_payload);
        let output = render_sense_lines(context, sense_passthrough_max_bytes);
        helpers::log_organ_output(cycle_id, stage, &output);
        output
    }
}

pub(crate) fn fallback_senses_section(
    senses: &[Sense],
    sense_descriptors: &[NeuralSignalDescriptor],
    sense_passthrough_max_bytes: usize,
) -> String {
    let context = SenseToolContext::from_inputs(senses, sense_descriptors);
    if context.entries().is_empty() {
        return "Empty".to_string();
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
                "endpoint_id": entry.endpoint_id,
                "neural_signal_descriptor_id": entry.sense_id,
                "fq_sense_id": entry.fq_sense_id,
                "weight": entry.weight,
                "payload": entry.payload,
                "payload_schema": entry.payload_schema,
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
            "endpoint_id": entry.endpoint_id,
            "neural_signal_descriptor_id": entry.sense_id,
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
    endpoint_id: String,
    sense_id: String,
    fq_sense_id: String,
    payload: String,
    weight: f64,
}

#[derive(Debug, Clone, Serialize)]
struct SenseDescriptorInput {
    fq_sense_id: String,
    payload_schema: serde_json::Value,
}

fn project_domain_sense_events(senses: &[Sense]) -> Vec<SenseInputEvent> {
    let mut events = Vec::new();
    for sense in senses {
        let sense_instance_id = NEXT_INTERNAL_SENSE_ID.fetch_add(1, Ordering::Relaxed);
        let fq_sense_id =
            build_fq_neural_signal_id(&sense.endpoint_id, &sense.neural_signal_descriptor_id);
        events.push(SenseInputEvent {
            sense_instance_id,
            sense_ref_id: sense_instance_id.to_string(),
            endpoint_id: sense.endpoint_id.clone(),
            sense_id: sense.neural_signal_descriptor_id.clone(),
            fq_sense_id,
            payload: sense.payload.clone(),
            weight: sense.weight,
        });
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

fn normalize_sense_ref_id(raw: &str) -> String {
    let trimmed = raw.trim();
    if let Some(prefix) = parse_internal_monotonic_sense_id_prefix(trimmed) {
        return prefix;
    }
    trimmed.to_string()
}

fn parse_internal_monotonic_sense_id_prefix(raw: &str) -> Option<String> {
    let digits = raw
        .chars()
        .take_while(|ch| ch.is_ascii_digit())
        .collect::<String>();
    if digits.is_empty() {
        return None;
    }
    let rest = &raw[digits.len()..];
    if !rest.is_empty() {
        let is_supported_suffix = rest.starts_with('.')
            || rest.starts_with(' ')
            || rest.starts_with(',')
            || rest.starts_with(':');
        if !is_supported_suffix {
            return None;
        }
    }
    Some(digits)
}

fn render_sense_lines(context: &SenseToolContext, sense_passthrough_max_bytes: usize) -> String {
    context
        .entries()
        .iter()
        .map(|entry| {
            let truncated = truncate_payload(&entry.payload, sense_passthrough_max_bytes);
            format!(
                "- {}. {}; payload={}",
                entry.sense_ref_id,
                render_metadata(entry, truncated.truncated_ratio),
                serde_json::to_string(&truncated.payload).unwrap_or_else(|_| "\"\"".to_string())
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_metadata(entry: &SenseToolContextEntry, truncated_ratio: Option<f64>) -> String {
    let mut metadata = format!(
        "endpoint_id={}, sense_id={}, weight={:.3}",
        entry.endpoint_id, entry.sense_id, entry.weight
    );
    if let Some(ratio) = truncated_ratio {
        metadata.push_str(&format!(", truncated_ratio={ratio:.3}"));
    }
    metadata
}

#[derive(Debug, Clone)]
struct TruncatedPayload {
    payload: String,
    truncated_ratio: Option<f64>,
}

fn truncate_payload(payload: &str, max_bytes: usize) -> TruncatedPayload {
    if payload.len() <= max_bytes {
        return TruncatedPayload {
            payload: payload.to_string(),
            truncated_ratio: None,
        };
    }
    if max_bytes == 0 {
        return TruncatedPayload {
            payload: "...(truncated)".to_string(),
            truncated_ratio: Some(1.0),
        };
    }

    let mut end = max_bytes.min(payload.len());
    while end > 0 && !payload.is_char_boundary(end) {
        end -= 1;
    }
    if end == 0 {
        return TruncatedPayload {
            payload: "...(truncated)".to_string(),
            truncated_ratio: Some(1.0),
        };
    }
    let omitted_bytes = payload.len().saturating_sub(end);
    let ratio = (omitted_bytes as f64) / (payload.len() as f64);
    TruncatedPayload {
        payload: format!("{}...(truncated)", &payload[..end]),
        truncated_ratio: Some(ratio),
    }
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
