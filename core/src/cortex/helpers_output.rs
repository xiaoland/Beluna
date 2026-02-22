use crate::{
    cortex::{
        cognition::{CognitionState, GoalNode, GoalTreePatchOp, L1MemoryPatchOp},
        types::{ActDraft, ActsHelperOutput, GoalTreePatchHelperOutput, L1MemoryPatchHelperOutput},
    },
    types::Act,
};

pub(crate) fn apply_cognition_patches(
    previous: &CognitionState,
    goal_tree_ops: &[GoalTreePatchOp],
    l1_memory_ops: &[L1MemoryPatchOp],
) -> CognitionState {
    let mut next = previous.clone();
    let mut changed = false;

    for op in goal_tree_ops {
        if apply_goal_tree_op(&mut next.goal_tree.user_partition, op) {
            changed = true;
        }
    }

    for op in l1_memory_ops {
        if apply_l1_memory_op(&mut next.l1_memory.entries, op) {
            changed = true;
        }
    }

    if changed {
        next.revision = next.revision.saturating_add(1);
    }
    next
}

fn apply_goal_tree_op(user_partition: &mut Vec<GoalNode>, op: &GoalTreePatchOp) -> bool {
    match op {
        GoalTreePatchOp::Sprout {
            numbering,
            node_id,
            summary,
            weight,
        } => {
            if !is_valid_numbering(numbering) {
                return false;
            }
            if user_partition
                .iter()
                .any(|node| node.numbering == *numbering)
            {
                return false;
            }

            let Some(normalized_weight) = normalize_weight(*weight, user_partition) else {
                return false;
            };

            user_partition.push(GoalNode {
                numbering: numbering.clone(),
                node_id: node_id.clone(),
                summary: summary.clone(),
                weight: normalized_weight,
            });
            true
        }
        GoalTreePatchOp::Prune { numbering } => {
            if !is_valid_numbering(numbering) {
                return false;
            }

            let descendant_prefix = format!("{numbering}.");
            let original_len = user_partition.len();
            user_partition.retain(|node| {
                node.numbering != *numbering && !node.numbering.starts_with(&descendant_prefix)
            });
            user_partition.len() != original_len
        }
        GoalTreePatchOp::Tilt { numbering, weight } => {
            if !is_valid_numbering(numbering) {
                return false;
            }
            let Some(idx) = user_partition
                .iter()
                .position(|node| node.numbering == *numbering)
            else {
                return false;
            };

            let Some(normalized_weight) = normalize_weight(*weight, user_partition) else {
                return false;
            };

            if (user_partition[idx].weight - normalized_weight).abs() <= f64::EPSILON {
                return false;
            }
            user_partition[idx].weight = normalized_weight;
            true
        }
    }
}

fn apply_l1_memory_op(entries: &mut Vec<String>, op: &L1MemoryPatchOp) -> bool {
    match op {
        L1MemoryPatchOp::Append { value } => {
            entries.push(value.clone());
            true
        }
        L1MemoryPatchOp::Insert { index, value } => {
            if *index > entries.len() {
                return false;
            }
            entries.insert(*index, value.clone());
            true
        }
        L1MemoryPatchOp::Remove { index } => {
            if *index >= entries.len() {
                return false;
            }
            entries.remove(*index);
            true
        }
    }
}

fn normalize_weight(weight: f64, user_partition: &[GoalNode]) -> Option<f64> {
    if !weight.is_finite() {
        return None;
    }

    if user_partition.is_empty() {
        return Some(0.5);
    }

    let mut min_weight = user_partition[0].weight;
    let mut max_weight = user_partition[0].weight;
    for node in user_partition.iter().skip(1) {
        if node.weight < min_weight {
            min_weight = node.weight;
        }
        if node.weight > max_weight {
            max_weight = node.weight;
        }
    }

    let span = max_weight - min_weight;
    if span.abs() <= f64::EPSILON {
        return Some(0.5);
    }

    if weight < min_weight || weight > max_weight {
        return None;
    }

    Some((weight - min_weight) / span)
}

fn is_valid_numbering(numbering: &str) -> bool {
    if numbering.is_empty() {
        return false;
    }

    for segment in numbering.split('.') {
        if segment.is_empty() {
            return false;
        }
        if !segment.chars().all(|ch| ch.is_ascii_digit()) {
            return false;
        }
        if segment.starts_with('0') && segment.len() > 1 {
            return false;
        }
        if segment == "0" {
            return false;
        }
    }

    true
}

pub(crate) fn acts_json_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "array",
        "items": {
            "type": "object",
            "properties": {
                "endpoint_id": { "type": "string" },
                "neural_signal_descriptor_id": { "type": "string" },
                "payload": {}
            },
            "required": ["endpoint_id", "neural_signal_descriptor_id", "payload"],
            "additionalProperties": false
        }
    })
}

pub(crate) fn goal_tree_patch_ops_json_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "array",
        "items": {
            "oneOf": [
                {
                    "type": "object",
                    "properties": {
                        "op": { "type": "string", "const": "sprout" },
                        "numbering": { "type": "string" },
                        "node_id": { "type": "string" },
                        "summary": { "type": "string" },
                        "weight": { "type": "number" }
                    },
                    "required": ["op", "numbering", "node_id", "summary", "weight"],
                    "additionalProperties": false
                },
                {
                    "type": "object",
                    "properties": {
                        "op": { "type": "string", "const": "prune" },
                        "numbering": { "type": "string" }
                    },
                    "required": ["op", "numbering"],
                    "additionalProperties": false
                },
                {
                    "type": "object",
                    "properties": {
                        "op": { "type": "string", "const": "tilt" },
                        "numbering": { "type": "string" },
                        "weight": { "type": "number" }
                    },
                    "required": ["op", "numbering", "weight"],
                    "additionalProperties": false
                }
            ]
        }
    })
}

pub(crate) fn l1_memory_patch_ops_json_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "array",
        "items": {
            "oneOf": [
                {
                    "type": "object",
                    "properties": {
                        "op": { "type": "string", "const": "append" },
                        "value": { "type": "string" }
                    },
                    "required": ["op", "value"],
                    "additionalProperties": false
                },
                {
                    "type": "object",
                    "properties": {
                        "op": { "type": "string", "const": "insert" },
                        "index": { "type": "integer", "minimum": 0 },
                        "value": { "type": "string" }
                    },
                    "required": ["op", "index", "value"],
                    "additionalProperties": false
                },
                {
                    "type": "object",
                    "properties": {
                        "op": { "type": "string", "const": "remove" },
                        "index": { "type": "integer", "minimum": 0 }
                    },
                    "required": ["op", "index"],
                    "additionalProperties": false
                }
            ]
        }
    })
}

pub(crate) fn parse_acts_helper_output(text: &str) -> Result<ActsHelperOutput, serde_json::Error> {
    serde_json::from_str::<ActsHelperOutput>(text)
}

pub(crate) fn parse_goal_tree_patch_helper_output(
    text: &str,
) -> Result<GoalTreePatchHelperOutput, serde_json::Error> {
    serde_json::from_str::<GoalTreePatchHelperOutput>(text)
}

pub(crate) fn parse_l1_memory_patch_helper_output(
    text: &str,
) -> Result<L1MemoryPatchHelperOutput, serde_json::Error> {
    serde_json::from_str::<L1MemoryPatchHelperOutput>(text)
}

pub(crate) fn materialize_acts(
    cycle_id: u64,
    drafts: Vec<ActDraft>,
    derive_act_id: impl Fn(u64, &str, &str, &serde_json::Value) -> String,
) -> Vec<Act> {
    drafts
        .into_iter()
        .map(|draft| Act {
            act_id: derive_act_id(
                cycle_id,
                &draft.endpoint_id,
                &draft.neural_signal_descriptor_id,
                &draft.payload,
            ),
            endpoint_id: draft.endpoint_id,
            neural_signal_descriptor_id: draft.neural_signal_descriptor_id,
            payload: draft.payload,
        })
        .collect()
}
