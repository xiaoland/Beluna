use async_trait::async_trait;

use crate::spine::{
    error::{SpineError, invalid_batch},
    ports::SpineExecutorPort,
    types::{ActDispatchRequest, SpineCapabilityCatalog, SpineEvent, SpineExecutionMode},
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

    async fn dispatch_act(&self, request: ActDispatchRequest) -> Result<SpineEvent, SpineError> {
        if request.act.act_id.trim().is_empty()
            || request.reserve_entry_id.trim().is_empty()
            || request.cost_attribution_id.trim().is_empty()
        {
            return Err(invalid_batch(
                "act dispatch request is missing act_id/reserve_entry_id/cost_attribution_id",
            ));
        }

        Ok(SpineEvent::ActApplied {
            cycle_id: request.cycle_id,
            seq_no: request.seq_no,
            act_id: request.act.act_id.clone(),
            capability_instance_id: request.act.capability_instance_id.clone(),
            reserve_entry_id: request.reserve_entry_id.clone(),
            cost_attribution_id: request.cost_attribution_id.clone(),
            actual_cost_micro: request.act.requested_resources.survival_micro.max(0),
            reference_id: format!("noop:settle:{}", request.act.act_id),
        })
    }

    fn capability_catalog_snapshot(&self) -> SpineCapabilityCatalog {
        SpineCapabilityCatalog::default()
    }
}
