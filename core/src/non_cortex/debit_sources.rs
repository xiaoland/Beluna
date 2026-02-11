use std::{collections::VecDeque, sync::Mutex};

use crate::{
    ai_gateway::telemetry::{GatewayTelemetryEvent, TelemetrySink},
    non_cortex::types::ExternalDebitObservation,
};

pub trait ExternalDebitSourcePort: Send + Sync {
    fn drain_observations(&self) -> Vec<ExternalDebitObservation>;
}

#[derive(Default)]
pub struct InMemoryDebitSource {
    observations: Mutex<VecDeque<ExternalDebitObservation>>,
}

impl InMemoryDebitSource {
    pub fn push(&self, observation: ExternalDebitObservation) {
        if let Ok(mut guard) = self.observations.lock() {
            guard.push_back(observation);
        }
    }
}

impl ExternalDebitSourcePort for InMemoryDebitSource {
    fn drain_observations(&self) -> Vec<ExternalDebitObservation> {
        let mut out = Vec::new();
        if let Ok(mut guard) = self.observations.lock() {
            while let Some(item) = guard.pop_front() {
                out.push(item);
            }
        }
        out
    }
}

pub struct AIGatewayApproxDebitSource {
    queue: Mutex<VecDeque<ExternalDebitObservation>>,
    token_to_survival_micro: i64,
}

impl AIGatewayApproxDebitSource {
    pub fn new(token_to_survival_micro: i64) -> Self {
        Self {
            queue: Mutex::new(VecDeque::new()),
            token_to_survival_micro: token_to_survival_micro.max(1),
        }
    }
}

impl Default for AIGatewayApproxDebitSource {
    fn default() -> Self {
        Self::new(1)
    }
}

impl ExternalDebitSourcePort for AIGatewayApproxDebitSource {
    fn drain_observations(&self) -> Vec<ExternalDebitObservation> {
        let mut out = Vec::new();
        if let Ok(mut guard) = self.queue.lock() {
            while let Some(item) = guard.pop_front() {
                out.push(item);
            }
        }
        out
    }
}

impl TelemetrySink for AIGatewayApproxDebitSource {
    fn on_event(&self, event: GatewayTelemetryEvent) {
        let GatewayTelemetryEvent::RequestCompleted {
            request_id,
            attempts,
            usage,
            cost_attribution_id,
        } = event
        else {
            return;
        };

        let Some(cost_attribution_id) = cost_attribution_id else {
            return;
        };

        let Some(usage) = usage else {
            return;
        };

        let total_tokens = usage
            .total_tokens
            .or(usage.output_tokens)
            .or(usage.input_tokens)
            .unwrap_or(0);

        if total_tokens == 0 {
            return;
        }

        let debit_survival_micro = total_tokens
            .saturating_mul(self.token_to_survival_micro as u64)
            .min(i64::MAX as u64) as i64;

        if let Ok(mut guard) = self.queue.lock() {
            guard.push_back(ExternalDebitObservation {
                reference_id: format!("gateway:{}:{}", request_id, attempts),
                cost_attribution_id,
                action_id: None,
                cycle_id: None,
                debit_survival_micro,
            });
        }
    }
}
