use crate::{
    continuity::{
        error::ContinuityError,
        state::ContinuityState,
        types::{ContinuityDispatchRecord, DispatchContext},
    },
    spine::types::SpineEvent,
    types::{
        Act, CognitionState, DispatchDecision, NeuralSignalDescriptorCatalog,
        NeuralSignalDescriptorDropPatch, NeuralSignalDescriptorPatch,
    },
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

    pub fn persist_cognition_state(
        &mut self,
        state: CognitionState,
    ) -> Result<(), ContinuityError> {
        self.state.persist_cognition_state(state);
        Ok(())
    }

    pub fn apply_neural_signal_descriptor_patch(&mut self, patch: &NeuralSignalDescriptorPatch) {
        self.state.apply_neural_signal_descriptor_patch(patch);
    }

    pub fn apply_neural_signal_descriptor_drop(
        &mut self,
        drop_patch: &NeuralSignalDescriptorDropPatch,
    ) {
        self.state.apply_neural_signal_descriptor_drop(drop_patch);
    }

    pub fn neural_signal_descriptor_snapshot(&self) -> NeuralSignalDescriptorCatalog {
        self.state.neural_signal_descriptor_snapshot()
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
