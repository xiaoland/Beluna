use async_trait::async_trait;

use crate::runtime_types::Act;
use crate::spine::{
    error::{SpineError, invalid_batch},
    ports::SpineExecutorPort,
    types::{EndpointExecutionOutcome, SpineCapabilityCatalog, SpineExecutionMode},
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

    async fn dispatch_act(&self, act: Act) -> Result<EndpointExecutionOutcome, SpineError> {
        if act.act_id.trim().is_empty() || act.body_endpoint_name.trim().is_empty() {
            return Err(invalid_batch("act dispatch is missing act_id/endpoint_id"));
        }

        Ok(EndpointExecutionOutcome::Applied {
            actual_cost_micro: act.requested_resources.survival_micro.max(0),
            reference_id: format!("noop:settle:{}", act.act_id),
        })
    }

    fn capability_catalog_snapshot(&self) -> SpineCapabilityCatalog {
        SpineCapabilityCatalog::default()
    }
}
