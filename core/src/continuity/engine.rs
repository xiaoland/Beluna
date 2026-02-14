use crate::{
    continuity::{
        error::ContinuityError,
        state::ContinuityState,
        types::{ContinuityDispatchRecord, DispatchContext},
    },
    cortex::CapabilityCatalog,
    runtime_types::{Act, CapabilityDropPatch, CapabilityPatch, CognitionState, DispatchDecision},
    spine::types::SpineEvent,
};

#[derive(Debug, Clone, Default)]
pub struct ContinuityEngine {
    state: ContinuityState,
}

impl ContinuityEngine {
    pub fn new(state: ContinuityState) -> Self {
        Self { state }
    }

    pub fn with_defaults() -> Self {
        Self {
            state: ContinuityState::new(),
        }
    }

    pub fn state(&self) -> &ContinuityState {
        &self.state
    }

    pub fn cognition_state_snapshot(&self) -> CognitionState {
        self.state.cognition_state_snapshot()
    }

    pub fn persist_cognition_state(&mut self, state: CognitionState) -> Result<(), ContinuityError> {
        self.state.persist_cognition_state(state);
        Ok(())
    }

    pub fn apply_capability_patch(&mut self, patch: &CapabilityPatch) {
        self.state.apply_capability_patch(patch);
    }

    pub fn apply_capability_drop(&mut self, drop_patch: &CapabilityDropPatch) {
        self.state.apply_capability_drop(drop_patch);
    }

    pub fn capabilities_snapshot(&self) -> CapabilityCatalog {
        self.state.capabilities_snapshot()
    }

    pub fn pre_dispatch(
        &self,
        act: &Act,
        cognition_state: &CognitionState,
        ctx: &DispatchContext,
    ) -> Result<DispatchDecision, ContinuityError> {
        Ok(self.state.pre_dispatch(act, cognition_state, ctx))
    }

    pub fn on_spine_event(
        &mut self,
        act: &Act,
        event: &SpineEvent,
        ctx: &DispatchContext,
    ) -> Result<(), ContinuityError> {
        self.state.on_spine_event(act, event, ctx);
        Ok(())
    }

    pub fn dispatch_records(&self) -> Vec<ContinuityDispatchRecord> {
        self.state.dispatch_records()
    }

    pub fn flush(&mut self) -> Result<(), ContinuityError> {
        Ok(())
    }
}
