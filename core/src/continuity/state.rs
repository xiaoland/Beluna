use std::collections::{BTreeMap, BTreeSet};

use crate::{
    continuity::error::{ContinuityError, invariant_violation},
    cortex::{CognitionState, GoalNode},
    types::{Act, DispatchDecision},
};

#[derive(Debug, Clone)]
pub struct ContinuityState {
    pub cognition_state: CognitionState,
}

impl Default for ContinuityState {
    fn default() -> Self {
        Self {
            cognition_state: CognitionState::default(),
        }
    }
}

impl ContinuityState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_cognition_state(cognition_state: CognitionState) -> Self {
        Self {
            cognition_state,
            ..Self::default()
        }
    }

    pub fn cognition_state_snapshot(&self) -> CognitionState {
        self.cognition_state.clone()
    }

    pub fn persist_cognition_state(
        &mut self,
        state: CognitionState,
    ) -> Result<(), ContinuityError> {
        validate_cognition_state(&state)?;
        self.cognition_state = state;
        Ok(())
    }

    pub fn on_act(&self, _act: &Act) -> DispatchDecision {
        DispatchDecision::Continue
    }
}

pub fn validate_cognition_state(state: &CognitionState) -> Result<(), ContinuityError> {
    let mut id_set = BTreeSet::new();
    for node in &state.goal_forest.nodes {
        validate_goal_node_fields(node, &mut id_set)?;
    }

    validate_goal_forest_topology(&state.goal_forest.nodes)?;

    Ok(())
}

fn validate_goal_node_fields(
    node: &GoalNode,
    id_set: &mut BTreeSet<String>,
) -> Result<(), ContinuityError> {
    if !node.weight.is_finite() || !(0.0..=1.0).contains(&node.weight) {
        return Err(invariant_violation(format!(
            "goal weight must be finite and in [0,1], got {} for id '{}'",
            node.weight, node.id
        )));
    }
    if node.id.trim().is_empty() {
        return Err(invariant_violation(format!("goal id cannot be empty")));
    }
    if !id_set.insert(node.id.clone()) {
        return Err(invariant_violation(format!(
            "duplicate goal id '{}'",
            node.id
        )));
    }
    if node.summary.trim().is_empty() {
        return Err(invariant_violation(format!(
            "goal summary cannot be empty for id '{}'",
            node.id
        )));
    }
    if node.status.trim().is_empty() {
        return Err(invariant_violation(format!(
            "goal status cannot be empty for id '{}'",
            node.id
        )));
    }

    Ok(())
}

fn validate_goal_forest_topology(nodes: &[GoalNode]) -> Result<(), ContinuityError> {
    let node_by_id: BTreeMap<&str, &GoalNode> =
        nodes.iter().map(|node| (node.id.as_str(), node)).collect();

    for node in nodes {
        validate_parent_chain_no_cycle(node, &node_by_id)?;
        validate_node_topology(node, &node_by_id)?;
    }

    let mut sibling_numberings = BTreeSet::new();
    for node in nodes {
        let Some(parent_id) = node.parent_id.as_deref() else {
            continue;
        };
        let Some(numbering) = node.numbering.as_deref() else {
            continue;
        };
        if !sibling_numberings.insert((parent_id.to_string(), numbering.to_string())) {
            return Err(invariant_violation(format!(
                "duplicate sibling numbering '{}' under parent '{}'",
                numbering, parent_id
            )));
        }
    }

    Ok(())
}

fn validate_parent_chain_no_cycle(
    node: &GoalNode,
    node_by_id: &BTreeMap<&str, &GoalNode>,
) -> Result<(), ContinuityError> {
    let mut seen = BTreeSet::new();
    let mut cursor = Some(node.id.as_str());
    while let Some(current_id) = cursor {
        if !seen.insert(current_id.to_string()) {
            return Err(invariant_violation(format!(
                "goal parent cycle detected at id '{}'",
                current_id
            )));
        }
        let Some(current_node) = node_by_id.get(current_id).copied() else {
            break;
        };
        cursor = current_node.parent_id.as_deref();
    }
    Ok(())
}

fn validate_node_topology(
    node: &GoalNode,
    node_by_id: &BTreeMap<&str, &GoalNode>,
) -> Result<(), ContinuityError> {
    match node.parent_id.as_deref() {
        None => {
            if node.numbering.is_some() {
                return Err(invariant_violation(format!(
                    "root goal '{}' must have null numbering",
                    node.id
                )));
            }
        }
        Some(parent_id) => {
            let Some(parent) = node_by_id.get(parent_id).copied() else {
                return Err(invariant_violation(format!(
                    "goal '{}' references missing parent '{}'",
                    node.id, parent_id
                )));
            };
            let Some(numbering) = node.numbering.as_deref() else {
                return Err(invariant_violation(format!(
                    "non-root goal '{}' must have numbering",
                    node.id
                )));
            };
            if !is_valid_numbering(numbering) {
                return Err(invariant_violation(format!(
                    "invalid goal numbering '{}' for id '{}'",
                    numbering, node.id
                )));
            }
            if !is_direct_child_numbering(numbering, parent.numbering.as_deref()) {
                return Err(invariant_violation(format!(
                    "goal '{}' numbering '{}' is not a direct child of parent '{}'",
                    node.id, numbering, parent.id
                )));
            }
        }
    }
    Ok(())
}

fn is_direct_child_numbering(numbering: &str, parent_numbering: Option<&str>) -> bool {
    direct_child_index(numbering, parent_numbering).is_some()
}

fn direct_child_index(numbering: &str, parent_numbering: Option<&str>) -> Option<u64> {
    if !is_valid_numbering(numbering) {
        return None;
    }

    match parent_numbering {
        None => {
            if numbering.contains('.') {
                return None;
            }
            numbering.parse::<u64>().ok()
        }
        Some(parent) => {
            let prefix = format!("{parent}.");
            let suffix = numbering.strip_prefix(prefix.as_str())?;
            if suffix.contains('.') {
                return None;
            }
            suffix.parse::<u64>().ok()
        }
    }
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
        if segment == "0" {
            return false;
        }
        if segment.starts_with('0') && segment.len() > 1 {
            return false;
        }
    }
    true
}
