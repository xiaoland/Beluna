use std::path::PathBuf;

use crate::{
    afferent_pathway::SenseAfferentPathway,
    continuity::{
        error::ContinuityError,
        persistence::ContinuityPersistence,
        state::{ContinuityState, validate_cognition_state},
        types::DispatchContext,
    },
    cortex::CognitionState,
    types::{
        Act, DispatchDecision, NeuralSignalDescriptorCatalog, NeuralSignalDescriptorDropPatch,
        NeuralSignalDescriptorPatch,
    },
};

#[derive(Clone)]
pub struct ContinuityEngine {
    state: ContinuityState,
    persistence: ContinuityPersistence,
    afferent_pathway: SenseAfferentPathway,
}

impl ContinuityEngine {
    pub fn new(
        state: ContinuityState,
        persistence: ContinuityPersistence,
        afferent_pathway: SenseAfferentPathway,
    ) -> Self {
        Self {
            state,
            persistence,
            afferent_pathway,
        }
    }

    pub fn with_defaults_at(
        path: PathBuf,
        afferent_pathway: SenseAfferentPathway,
    ) -> Result<Self, ContinuityError> {
        let persistence = ContinuityPersistence::new(path);
        let cognition_state = match persistence.load()? {
            Some(state) => {
                validate_cognition_state(&state)?;
                state
            }
            None => CognitionState::default(),
        };

        Ok(Self {
            state: ContinuityState::with_cognition_state(cognition_state),
            persistence,
            afferent_pathway,
        })
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
        self.state.persist_cognition_state(state)?;
        self.persistence
            .save(&self.state.cognition_state_snapshot())
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

    pub fn on_act(
        &self,
        act: &Act,
        _ctx: &DispatchContext,
    ) -> Result<DispatchDecision, ContinuityError> {
        let _ = &self.afferent_pathway;
        Ok(self.state.on_act(act))
    }

    pub fn flush(&mut self) -> Result<(), ContinuityError> {
        self.persistence
            .save(&self.state.cognition_state_snapshot())
    }
}
