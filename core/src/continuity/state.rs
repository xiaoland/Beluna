use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::{
    continuity::types::{ContinuityDispatchRecord, DispatchContext},
    cortex::{AffordanceCapability, CapabilityCatalog},
    runtime_types::{
        Act, CapabilityDropPatch, CapabilityPatch, CognitionState, DispatchDecision,
        RequestedResources,
    },
    spine::types::{EndpointCapabilityDescriptor, RouteKey, SpineEvent},
};

const MAX_DISPATCH_RECORDS: usize = 256;

#[derive(Debug, Clone, Default)]
pub struct ContinuityState {
    pub cognition_state: CognitionState,
    capability_version: u64,
    capability_entries: BTreeMap<RouteKey, EndpointCapabilityDescriptor>,
    tombstoned_routes: BTreeSet<RouteKey>,
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

    pub fn apply_capability_patch(&mut self, patch: &CapabilityPatch) {
        for descriptor in &patch.entries {
            let route = descriptor.route.clone();
            self.tombstoned_routes.remove(&route);
            self.capability_entries.insert(route, descriptor.clone());
            self.capability_version = self.capability_version.saturating_add(1);
        }
    }

    pub fn apply_capability_drop(&mut self, drop_patch: &CapabilityDropPatch) {
        for route in &drop_patch.routes {
            self.capability_entries.remove(route);
            self.tombstoned_routes.insert(route.clone());
            self.capability_version = self.capability_version.saturating_add(1);
        }
    }

    pub fn capabilities_snapshot(&self) -> CapabilityCatalog {
        let mut grouped: BTreeMap<String, Vec<&EndpointCapabilityDescriptor>> = BTreeMap::new();
        for descriptor in self.capability_entries.values() {
            grouped
                .entry(descriptor.route.endpoint_id.clone())
                .or_default()
                .push(descriptor);
        }

        let mut affordances = Vec::with_capacity(grouped.len());
        for (endpoint_id, entries) in grouped {
            let representative = entries
                .first()
                .expect("grouped capability entries should never be empty");

            let mut capability_ids = BTreeSet::new();
            for entry in entries.iter() {
                capability_ids.insert(entry.route.capability_id.clone());
            }

            affordances.push(AffordanceCapability {
                endpoint_id,
                allowed_capability_ids: capability_ids.into_iter().collect(),
                payload_schema: representative.payload_schema.clone(),
                max_payload_bytes: representative.max_payload_bytes,
                default_resources: RequestedResources {
                    survival_micro: representative.default_cost.survival_micro,
                    time_ms: representative.default_cost.time_ms,
                    io_units: representative.default_cost.io_units,
                    token_units: representative.default_cost.token_units,
                },
            });
        }

        CapabilityCatalog {
            version: format!("continuity:v{}", self.capability_version),
            affordances,
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
