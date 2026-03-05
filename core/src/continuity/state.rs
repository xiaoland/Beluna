use std::collections::BTreeSet;

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
        validate_goal_node(node, &mut id_set)?;
    }
    Ok(())
}

fn validate_goal_node(
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
        return Err(invariant_violation("goal id cannot be empty"));
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

    for child in &node.children {
        validate_goal_node(child, id_set)?;
    }

    Ok(())
}
