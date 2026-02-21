use crate::types::{
    CognitionState, NeuralSignalDescriptor, NeuralSignalType, PhysicalState, Sense,
};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PrimarySenseEvent {
    pub endpoint: String,
    pub sense: String,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PrimarySenseDescriptor {
    pub endpoint: String,
    pub sense: String,
    pub payload_schema: serde_json::Value,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PrimaryActDescriptor {
    pub endpoint: String,
    pub act: String,
    pub payload_schema: serde_json::Value,
}

pub(crate) fn fallback_senses_section(
    senses: &[Sense],
    sense_descriptors: &[NeuralSignalDescriptor],
) -> String {
    let semantic_senses = semantic_sense_events(senses);
    let semantic_catalog = semantic_sense_catalog(sense_descriptors);
    format!(
        "## senses\n{}\n\n## sense catalog\n{}",
        serde_json::to_string_pretty(&semantic_senses).unwrap_or_else(|_| "[]".to_string()),
        serde_json::to_string_pretty(&semantic_catalog).unwrap_or_else(|_| "[]".to_string()),
    )
}

pub(crate) fn fallback_act_descriptor_catalog_section(
    act_descriptors: &[NeuralSignalDescriptor],
) -> String {
    serde_json::to_string_pretty(&semantic_act_catalog(act_descriptors))
        .unwrap_or_else(|_| "[]".to_string())
}

pub(crate) fn goal_stack_section(cognition_state: &CognitionState) -> String {
    serde_json::to_string_pretty(&cognition_state.goal_stack).unwrap_or_else(|_| "[]".to_string())
}

pub(crate) fn context_section(
    physical_state: &PhysicalState,
    cognition_state: &CognitionState,
) -> String {
    let (sense_count, act_count) = physical_state.capabilities.entries.iter().fold(
        (0usize, 0usize),
        |(sense_count, act_count), entry| match entry.r#type {
            NeuralSignalType::Sense => (sense_count + 1, act_count),
            NeuralSignalType::Act => (sense_count, act_count + 1),
        },
    );

    format!(
        "## runtime_context\n{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "cycle_id": physical_state.cycle_id,
            "ledger": physical_state.ledger,
            "capability_summary": {
                "version": physical_state.capabilities.version,
                "sense_count": sense_count,
                "act_count": act_count
            },
            "cognition_revision": cognition_state.revision
        }))
        .unwrap_or_else(|_| "{}".to_string())
    )
}

pub(crate) fn semantic_sense_events(senses: &[Sense]) -> Vec<PrimarySenseEvent> {
    senses
        .iter()
        .filter_map(|sense| match sense {
            Sense::Domain(domain) => Some(PrimarySenseEvent {
                endpoint: domain.endpoint_id.clone(),
                sense: domain.neural_signal_descriptor_id.clone(),
                payload: domain.payload.clone(),
            }),
            _ => None,
        })
        .collect()
}

pub(crate) fn semantic_sense_catalog(
    sense_descriptors: &[NeuralSignalDescriptor],
) -> Vec<PrimarySenseDescriptor> {
    sense_descriptors
        .iter()
        .map(|descriptor| PrimarySenseDescriptor {
            endpoint: descriptor.endpoint_id.clone(),
            sense: descriptor.neural_signal_descriptor_id.clone(),
            payload_schema: descriptor.payload_schema.clone(),
        })
        .collect()
}

pub(crate) fn semantic_act_catalog(
    act_descriptors: &[NeuralSignalDescriptor],
) -> Vec<PrimaryActDescriptor> {
    act_descriptors
        .iter()
        .map(semantic_act_descriptor)
        .collect()
}

pub(crate) fn semantic_act_descriptor(
    act_descriptor: &NeuralSignalDescriptor,
) -> PrimaryActDescriptor {
    PrimaryActDescriptor {
        endpoint: act_descriptor.endpoint_id.clone(),
        act: act_descriptor.neural_signal_descriptor_id.clone(),
        payload_schema: act_descriptor.payload_schema.clone(),
    }
}

pub(crate) fn sense_descriptors(physical_state: &PhysicalState) -> Vec<NeuralSignalDescriptor> {
    let mut entries: Vec<_> = physical_state
        .capabilities
        .entries
        .iter()
        .filter(|entry| entry.r#type == NeuralSignalType::Sense)
        .cloned()
        .collect();
    entries.sort_by(|lhs, rhs| {
        lhs.endpoint_id.cmp(&rhs.endpoint_id).then_with(|| {
            lhs.neural_signal_descriptor_id
                .cmp(&rhs.neural_signal_descriptor_id)
        })
    });
    entries.dedup_by(|lhs, rhs| {
        lhs.endpoint_id == rhs.endpoint_id
            && lhs.neural_signal_descriptor_id == rhs.neural_signal_descriptor_id
            && lhs.payload_schema == rhs.payload_schema
    });
    entries
}

pub(crate) fn act_descriptors(physical_state: &PhysicalState) -> Vec<NeuralSignalDescriptor> {
    let mut entries: Vec<_> = physical_state
        .capabilities
        .entries
        .iter()
        .filter(|entry| entry.r#type == NeuralSignalType::Act)
        .cloned()
        .collect();
    entries.sort_by(|lhs, rhs| {
        lhs.endpoint_id.cmp(&rhs.endpoint_id).then_with(|| {
            lhs.neural_signal_descriptor_id
                .cmp(&rhs.neural_signal_descriptor_id)
        })
    });
    entries.dedup_by(|lhs, rhs| {
        lhs.endpoint_id == rhs.endpoint_id
            && lhs.neural_signal_descriptor_id == rhs.neural_signal_descriptor_id
            && lhs.payload_schema == rhs.payload_schema
    });
    entries
}

pub(crate) fn act_descriptor_cache_key(act_descriptors: &[NeuralSignalDescriptor]) -> String {
    let canonical = serde_json::to_string(act_descriptors).unwrap_or_else(|_| "[]".to_string());
    format!("{:x}", md5::compute(canonical.as_bytes()))
}
