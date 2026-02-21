use crate::types::{
    CognitionState, NeuralSignalDescriptor, NeuralSignalType, PhysicalState, Sense,
};

pub(crate) fn fallback_senses_section(
    senses: &[Sense],
    sense_descriptors: &[NeuralSignalDescriptor],
) -> String {
    format!(
        "## raw senses\n{}\n\n## sense descriptors\n{}",
        serde_json::to_string_pretty(senses).unwrap_or_else(|_| "[]".to_string()),
        serde_json::to_string_pretty(sense_descriptors).unwrap_or_else(|_| "[]".to_string()),
    )
}

pub(crate) fn fallback_act_descriptor_catalog_section(
    act_descriptors: &[NeuralSignalDescriptor],
) -> String {
    serde_json::to_string_pretty(act_descriptors).unwrap_or_else(|_| "[]".to_string())
}

pub(crate) fn goal_stack_section(cognition_state: &CognitionState) -> String {
    serde_json::to_string_pretty(&cognition_state.goal_stack).unwrap_or_else(|_| "[]".to_string())
}

pub(crate) fn context_section(
    physical_state: &PhysicalState,
    cognition_state: &CognitionState,
) -> String {
    format!(
        "## physical_state\n{}\n\n## cognition_meta\n{}",
        serde_json::to_string_pretty(physical_state).unwrap_or_else(|_| "{}".to_string()),
        serde_json::to_string_pretty(&serde_json::json!({ "revision": cognition_state.revision }))
            .unwrap_or_else(|_| "{}".to_string())
    )
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
    entries
}

pub(crate) fn act_descriptor_cache_key(act_descriptors: &[NeuralSignalDescriptor]) -> String {
    let canonical = serde_json::to_string(act_descriptors).unwrap_or_else(|_| "[]".to_string());
    format!("{:x}", md5::compute(canonical.as_bytes()))
}
