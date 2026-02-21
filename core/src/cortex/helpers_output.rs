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

fn apply_goal_tree_op(user_root: &mut GoalNode, op: &GoalTreePatchOp) -> bool {
    match op {
        GoalTreePatchOp::Sprout {
            parent_node_id,
            node_id,
            summary,
            weight,
        } => {
            if node_exists(user_root, node_id) {
                return false;
            }
            let Some(parent) = find_node_mut(user_root, parent_node_id) else {
                return false;
            };
            parent.children.push(GoalNode {
                node_id: node_id.clone(),
                summary: summary.clone(),
                weight: clamp_weight(*weight),
                children: Vec::new(),
            });
            true
        }
        GoalTreePatchOp::Prune { node_id } => {
            if node_id == "user-root" {
                return false;
            }
            prune_node(user_root, node_id)
        }
        GoalTreePatchOp::Tilt { node_id, weight } => {
            let Some(node) = find_node_mut(user_root, node_id) else {
                return false;
            };
            let clamped = clamp_weight(*weight);
            if node.weight == clamped {
                return false;
            }
            node.weight = clamped;
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

fn node_exists(node: &GoalNode, node_id: &str) -> bool {
    if node.node_id == node_id {
        return true;
    }
    node.children
        .iter()
        .any(|child| node_exists(child, node_id))
}

fn find_node_mut<'a>(node: &'a mut GoalNode, node_id: &str) -> Option<&'a mut GoalNode> {
    if node.node_id == node_id {
        return Some(node);
    }
    for child in &mut node.children {
        if let Some(found) = find_node_mut(child, node_id) {
            return Some(found);
        }
    }
    None
}

fn prune_node(node: &mut GoalNode, target: &str) -> bool {
    if let Some(idx) = node
        .children
        .iter()
        .position(|child| child.node_id == target)
    {
        node.children.remove(idx);
        return true;
    }
    for child in &mut node.children {
        if prune_node(child, target) {
            return true;
        }
    }
    false
}

fn clamp_weight(weight: i32) -> i32 {
    weight.clamp(-1000, 1000)
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
                        "parent_node_id": { "type": "string" },
                        "node_id": { "type": "string" },
                        "summary": { "type": "string" },
                        "weight": { "type": "integer" }
                    },
                    "required": ["op", "parent_node_id", "node_id", "summary", "weight"],
                    "additionalProperties": false
                },
                {
                    "type": "object",
                    "properties": {
                        "op": { "type": "string", "const": "prune" },
                        "node_id": { "type": "string" }
                    },
                    "required": ["op", "node_id"],
                    "additionalProperties": false
                },
                {
                    "type": "object",
                    "properties": {
                        "op": { "type": "string", "const": "tilt" },
                        "node_id": { "type": "string" },
                        "weight": { "type": "integer" }
                    },
                    "required": ["op", "node_id", "weight"],
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
