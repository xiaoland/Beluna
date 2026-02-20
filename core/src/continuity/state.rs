use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::{
    continuity::types::{ContinuityDispatchRecord, DispatchContext},
    spine::types::SpineEvent,
    types::{
        Act, CognitionState, DispatchDecision, NeuralSignalDescriptor,
        NeuralSignalDescriptorCatalog, NeuralSignalDescriptorDropPatch,
        NeuralSignalDescriptorPatch, NeuralSignalDescriptorRouteKey,
    },
};

const MAX_DISPATCH_RECORDS: usize = 256;

#[derive(Debug, Clone, Default)]
pub struct ContinuityState {
    pub cognition_state: CognitionState,
    neural_signal_descriptor_version: u64,
    neural_signal_descriptor_entries:
        BTreeMap<NeuralSignalDescriptorRouteKey, NeuralSignalDescriptor>,
    tombstoned_routes: BTreeSet<NeuralSignalDescriptorRouteKey>,
    dispatch_records: VecDeque<ContinuityDispatchRecord>,
}

impl ContinuityState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn cognition_state_snapshot(&self) -> CognitionState {
        self.cognition_state.clone()
    }

    pub fn persist_cognition_state(&mut self, state: CognitionState) {
        self.cognition_state = state;
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

    pub fn pre_dispatch(
        &self,
        _act: &Act,
        _cognition_state: &CognitionState,
        _ctx: &DispatchContext,
    ) -> DispatchDecision {
        DispatchDecision::Continue
    }

    pub fn on_spine_event(&mut self, act: &Act, event: &SpineEvent, ctx: &DispatchContext) {
        let (event_name, reference_id) = match event {
            SpineEvent::ActApplied { reference_id, .. } => {
                ("act_applied".to_string(), Some(reference_id.clone()))
            }
            SpineEvent::ActRejected { reference_id, .. } => {
                ("act_rejected".to_string(), Some(reference_id.clone()))
            }
            SpineEvent::ActDeferred { reference_id, .. } => {
                ("act_deferred".to_string(), Some(reference_id.clone()))
            }
        };

        self.dispatch_records.push_back(ContinuityDispatchRecord {
            cycle_id: ctx.cycle_id,
            seq_no: ctx.act_seq_no,
            act_id: act.act_id.clone(),
            event: event_name,
            reference_id,
        });
        while self.dispatch_records.len() > MAX_DISPATCH_RECORDS {
            self.dispatch_records.pop_front();
        }
    }

    pub fn dispatch_records(&self) -> Vec<ContinuityDispatchRecord> {
        self.dispatch_records.iter().cloned().collect()
    }
}
