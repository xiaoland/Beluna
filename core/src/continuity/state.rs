use std::collections::{BTreeMap, BTreeSet};

use crate::{
    continuity::error::{ContinuityError, invariant_violation},
    cortex::{CognitionState, GoalNode, ROOT_PARTITION},
    types::{
        Act, DispatchDecision, NeuralSignalDescriptor, NeuralSignalDescriptorCatalog,
        NeuralSignalDescriptorDropPatch, NeuralSignalDescriptorPatch,
        NeuralSignalDescriptorRouteKey,
    },
};

#[derive(Debug, Clone)]
pub struct ContinuityState {
    pub cognition_state: CognitionState,
    neural_signal_descriptor_version: u64,
    neural_signal_descriptor_entries:
        BTreeMap<NeuralSignalDescriptorRouteKey, NeuralSignalDescriptor>,
    tombstoned_routes: BTreeSet<NeuralSignalDescriptorRouteKey>,
}

impl Default for ContinuityState {
    fn default() -> Self {
        Self {
            cognition_state: CognitionState::default(),
            neural_signal_descriptor_version: 0,
            neural_signal_descriptor_entries: BTreeMap::new(),
            tombstoned_routes: BTreeSet::new(),
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

    pub fn apply_neural_signal_descriptor_patch(&mut self, patch: &NeuralSignalDescriptorPatch) {
        for descriptor in &patch.entries {
            let route = NeuralSignalDescriptorRouteKey {
                r#type: descriptor.r#type,
                endpoint_id: descriptor.endpoint_id.clone(),
                neural_signal_descriptor_id: descriptor.neural_signal_descriptor_id.clone(),
            };
            self.tombstoned_routes.remove(&route);
            self.neural_signal_descriptor_entries
                .insert(route, descriptor.clone());
            self.neural_signal_descriptor_version =
                self.neural_signal_descriptor_version.saturating_add(1);
        }
    }

    pub fn apply_neural_signal_descriptor_drop(
        &mut self,
        drop_patch: &NeuralSignalDescriptorDropPatch,
    ) {
        for route in &drop_patch.routes {
            self.neural_signal_descriptor_entries.remove(route);
            self.tombstoned_routes.insert(route.clone());
            self.neural_signal_descriptor_version =
                self.neural_signal_descriptor_version.saturating_add(1);
        }
    }

    pub fn neural_signal_descriptor_snapshot(&self) -> NeuralSignalDescriptorCatalog {
        let mut entries: Vec<_> = self
            .neural_signal_descriptor_entries
            .values()
            .cloned()
            .collect();
        entries.sort_by(|lhs, rhs| {
            lhs.r#type
                .cmp(&rhs.r#type)
                .then_with(|| lhs.endpoint_id.cmp(&rhs.endpoint_id))
                .then_with(|| {
                    lhs.neural_signal_descriptor_id
                        .cmp(&rhs.neural_signal_descriptor_id)
                })
        });
        NeuralSignalDescriptorCatalog {
            version: format!("continuity:v{}", self.neural_signal_descriptor_version),
            entries,
        }
    }

    pub fn on_act(&self, _act: &Act) -> DispatchDecision {
        DispatchDecision::Continue
    }
}

pub fn validate_cognition_state(state: &CognitionState) -> Result<(), ContinuityError> {
    let root_expected: Vec<String> = ROOT_PARTITION.iter().map(|s| (*s).to_string()).collect();
    if state.goal_tree.root_partition != root_expected {
        return Err(invariant_violation(
            "goal-tree root partition must match compile-time constants",
        ));
    }

    if state.goal_tree.user_partition.node_id != "user-root" {
        return Err(invariant_violation(
            "goal-tree user partition root node_id must be 'user-root'",
        ));
    }

    let mut ids = BTreeSet::new();
    collect_node_ids(&state.goal_tree.user_partition, &mut ids)?;

    Ok(())
}

fn collect_node_ids(node: &GoalNode, ids: &mut BTreeSet<String>) -> Result<(), ContinuityError> {
    if !ids.insert(node.node_id.clone()) {
        return Err(invariant_violation(format!(
            "duplicate goal node id '{}'",
            node.node_id
        )));
    }
    for child in &node.children {
        collect_node_ids(child, ids)?;
    }
    Ok(())
}
