use crate::{
    cortex::cognition::GoalNode,
    types::{NeuralSignalDescriptor, NeuralSignalType, Sense},
};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PrimarySenseEvent {
    pub sense_id: String,
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

const GOAL_TREE_EMPTY_PURSUITS_ONE_SHOT: &str = concat!(
    "1 (w=0.60) Build an environment baseline from current senses and constraints.\n",
    "1.1 (w=0.50) Verify executable acts and their payload boundaries.\n",
    "1.2 (w=0.40) Extract high-signal observations and unresolved unknowns.\n",
    "2 (w=0.50) Preserve compact cross-tick heuristics.\n",
    "2.1 (w=0.40) Keep focal-awareness as styleless bullet point statements."
);

const L1_MEMORY_EMPTY_ONE_SHOT: &str = concat!(
    "- Record concrete observations, not narrative.\n",
    "- Keep one bullet per actionable heuristic.\n",
    "- Prefer short statements that can guide the next tick directly."
);

pub(crate) fn fallback_senses_section(
    senses: &[Sense],
    sense_descriptors: &[NeuralSignalDescriptor],
) -> String {
    let semantic_senses = semantic_sense_events(senses);
    if semantic_senses.is_empty() {
        return "[]".to_string();
    }

    let semantic_catalog = semantic_sense_catalog(sense_descriptors);
    let mut entries = Vec::with_capacity(semantic_senses.len());
    for sense in &semantic_senses {
        let payload_json =
            serde_json::to_string_pretty(&sense.payload).unwrap_or_else(|_| "{}".to_string());
        let payload_schema_json = semantic_catalog
            .iter()
            .find(|descriptor| {
                descriptor.endpoint == sense.endpoint && descriptor.sense == sense.sense
            })
            .and_then(|descriptor| serde_json::to_string_pretty(&descriptor.payload_schema).ok())
            .unwrap_or_else(|| "{}".to_string());
        entries.push(format!(
            "<sense endpoint-id=\"{}\" sense-id=\"{}\" sense-name=\"{}\">\n## payload\n{}\n\n## payload-schema\n{}\n</sense>",
            escape_xml_attr(&sense.endpoint),
            escape_xml_attr(&sense.sense_id),
            escape_xml_attr(&sense.sense),
            payload_json,
            payload_schema_json
        ));
    }

    entries.join("\n")
}

pub(crate) fn fallback_act_descriptor_catalog_section(
    act_descriptors: &[NeuralSignalDescriptor],
) -> String {
    if act_descriptors.is_empty() {
        return "[]".to_string();
    }

    let mut entries = Vec::with_capacity(act_descriptors.len());
    for descriptor in act_descriptors {
        let payload_schema_json = serde_json::to_string_pretty(&descriptor.payload_schema)
            .unwrap_or_else(|_| "{}".to_string());
        entries.push(format!(
            "<act-descriptor endpoint-id=\"{}\" act-id=\"{}\">\n## payload-schema\n{}\n</act-descriptor>",
            escape_xml_attr(&descriptor.endpoint_id),
            escape_xml_attr(&descriptor.neural_signal_descriptor_id),
            payload_schema_json
        ));
    }
    entries.join("\n")
}

pub(crate) fn fallback_goal_tree_section(user_partition: &[GoalNode]) -> String {
    serde_json::to_string_pretty(user_partition).unwrap_or_else(|_| "[]".to_string())
}

pub(crate) fn instincts_section(root_partition: &[String]) -> String {
    serde_json::to_string_pretty(root_partition).unwrap_or_else(|_| "[]".to_string())
}

pub(crate) fn l1_memory_section(l1_memory: &[String]) -> String {
    if l1_memory.is_empty() {
        return l1_memory_empty_one_shot().to_string();
    }
    serde_json::to_string_pretty(l1_memory).unwrap_or_else(|_| "[]".to_string())
}

pub(crate) fn goal_tree_user_partition_json(user_partition: &[GoalNode]) -> String {
    serde_json::to_string_pretty(user_partition).unwrap_or_else(|_| "[]".to_string())
}

pub(crate) fn l1_memory_json(l1_memory: &[String]) -> String {
    serde_json::to_string_pretty(l1_memory).unwrap_or_else(|_| "[]".to_string())
}

pub(crate) fn semantic_sense_events(senses: &[Sense]) -> Vec<PrimarySenseEvent> {
    senses
        .iter()
        .filter_map(|sense| match sense {
            Sense::Domain(domain) => Some(PrimarySenseEvent {
                sense_id: domain.sense_id.clone(),
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

pub(crate) fn sense_descriptors(
    capability_entries: &[NeuralSignalDescriptor],
) -> Vec<NeuralSignalDescriptor> {
    let mut entries: Vec<_> = capability_entries
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

pub(crate) fn act_descriptors(
    capability_entries: &[NeuralSignalDescriptor],
) -> Vec<NeuralSignalDescriptor> {
    let mut entries: Vec<_> = capability_entries
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

pub(crate) fn goal_tree_cache_key(user_partition: &[GoalNode]) -> String {
    let canonical = serde_json::to_string(user_partition).unwrap_or_else(|_| "[]".to_string());
    format!("{:x}", md5::compute(canonical.as_bytes()))
}

pub(crate) fn goal_tree_empty_pursuits_one_shot() -> &'static str {
    GOAL_TREE_EMPTY_PURSUITS_ONE_SHOT
}

pub(crate) fn l1_memory_empty_one_shot() -> &'static str {
    L1_MEMORY_EMPTY_ONE_SHOT
}

fn escape_xml_attr(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
