use std::path::PathBuf;

use crate::{
    continuity::{
        error::ContinuityError,
        persistence::ContinuityPersistence,
        state::{ContinuityState, validate_cognition_state},
        types::DispatchContext,
    },
    cortex::CognitionState,
    types::{Act, DispatchDecision},
};

#[derive(Clone)]
pub struct ContinuityEngine {
    state: ContinuityState,
    persistence: ContinuityPersistence,
}

impl ContinuityEngine {
    pub fn new(state: ContinuityState, persistence: ContinuityPersistence) -> Self {
        Self { state, persistence }
    }

    pub fn with_defaults_at(path: PathBuf) -> Result<Self, ContinuityError> {
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

    pub fn on_act(
        &self,
        act: &Act,
        _ctx: &DispatchContext,
    ) -> Result<DispatchDecision, ContinuityError> {
        Ok(self.state.on_act(act))
    }

    pub fn flush(&mut self) -> Result<(), ContinuityError> {
        self.persistence
            .save(&self.state.cognition_state_snapshot())
    }
}
