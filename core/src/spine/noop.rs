use async_trait::async_trait;

use crate::spine::{
    error::{SpineError, invalid_batch},
    ports::SpineExecutorPort,
    types::{
        AdmittedActionBatch, OrderedSpineEvent, SpineCapabilityCatalog, SpineEvent,
        SpineExecutionMode, SpineExecutionReport,
    },
};

#[derive(Debug, Clone)]
pub struct DeterministicNoopSpine {
    mode: SpineExecutionMode,
}

impl DeterministicNoopSpine {
    pub fn new(mode: SpineExecutionMode) -> Self {
        Self { mode }
    }
}

impl Default for DeterministicNoopSpine {
    fn default() -> Self {
        Self {
            mode: SpineExecutionMode::SerializedDeterministic,
        }
    }
}

#[async_trait]
impl SpineExecutorPort for DeterministicNoopSpine {
    fn mode(&self) -> SpineExecutionMode {
        self.mode
    }

    async fn execute_admitted(
        &self,
        admitted: AdmittedActionBatch,
    ) -> Result<SpineExecutionReport, SpineError> {
        if admitted
            .actions
            .iter()
            .any(|action| action.neural_signal_id.is_empty() || action.reserve_entry_id.is_empty())
        {
            return Err(invalid_batch(
                "admitted action is missing neural_signal_id or reserve_entry_id",
            ));
        }

        let action_count = admitted.actions.len();
        let events = admitted
            .actions
            .into_iter()
            .enumerate()
            .map(|(index, action)| OrderedSpineEvent {
                seq_no: (index as u64) + 1,
                event: SpineEvent::ActionApplied {
                    neural_signal_id: action.neural_signal_id.clone(),
                    capability_instance_id: action.capability_instance_id.clone(),
                    reserve_entry_id: action.reserve_entry_id.clone(),
                    cost_attribution_id: action.cost_attribution_id.clone(),
                    actual_cost_micro: action.reserved_cost.survival_micro,
                    reference_id: format!("noop:settle:{}", action.neural_signal_id),
                },
            })
            .collect();

        Ok(SpineExecutionReport {
            mode: self.mode,
            events,
            replay_cursor: Some(format!("noop:{}:{}", admitted.cycle_id, action_count)),
        })
    }

    fn capability_catalog_snapshot(&self) -> SpineCapabilityCatalog {
        SpineCapabilityCatalog::default()
    }
}
