use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

use async_trait::async_trait;
use beluna::{
    spine::{ActDispatchResult, Endpoint, SpineError},
    types::Act,
};
use serde_json::json;

use super::{case::EndpointResponseSpec, evidence::EvidenceJournal};

#[derive(Clone)]
pub struct TickClock {
    current: Arc<AtomicU64>,
}

impl TickClock {
    pub fn new(initial_tick: u64) -> Self {
        Self {
            current: Arc::new(AtomicU64::new(initial_tick)),
        }
    }

    pub fn set(&self, tick: u64) {
        self.current.store(tick, Ordering::Relaxed);
    }

    fn get(&self) -> u64 {
        self.current.load(Ordering::Relaxed)
    }
}

pub struct AckRecordingEndpoint {
    logical_endpoint_id: String,
    response: EndpointResponseSpec,
    journal: EvidenceJournal,
    tick_clock: TickClock,
}

impl AckRecordingEndpoint {
    pub fn new(
        logical_endpoint_id: String,
        response: EndpointResponseSpec,
        journal: EvidenceJournal,
        tick_clock: TickClock,
    ) -> Self {
        Self {
            logical_endpoint_id,
            response,
            journal,
            tick_clock,
        }
    }
}

#[async_trait]
impl Endpoint for AckRecordingEndpoint {
    async fn invoke(&self, act: Act) -> Result<ActDispatchResult, SpineError> {
        let tick = self.tick_clock.get();
        self.journal.record(
            "act.received",
            json!({
                "tick": tick,
                "endpoint_id": self.logical_endpoint_id,
                "runtime_endpoint_id": act.endpoint_id,
                "neural_signal_descriptor_id": act.neural_signal_descriptor_id,
                "act_instance_id": act.act_instance_id,
                "payload": act.payload,
            }),
        );

        let reference_id = self
            .response
            .reference_id_template
            .replace("{act_instance_id}", &act.act_instance_id);

        self.journal.record(
            "dispatch.outcome",
            json!({
                "tick": tick,
                "endpoint_id": self.logical_endpoint_id,
                "runtime_endpoint_id": act.endpoint_id,
                "neural_signal_descriptor_id": act.neural_signal_descriptor_id,
                "act_instance_id": act.act_instance_id,
                "outcome": self.response.outcome,
                "reference_id": reference_id,
            }),
        );

        Ok(ActDispatchResult::Acknowledged { reference_id })
    }
}
