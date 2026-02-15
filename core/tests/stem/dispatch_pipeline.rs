use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::{Mutex, mpsc};

use beluna::{
    continuity::ContinuityEngine,
    cortex::{CortexError, CortexOutput, CortexPort},
    ledger::LedgerStage,
    runtime_types::{Act, CognitionState, PhysicalState, RequestedResources, Sense, SenseDatum},
    spine::{
        SpineCapabilityCatalog, SpineExecutionMode, SpineExecutorPort,
        types::EndpointExecutionOutcome,
    },
    stem::StemRuntime,
};

struct TwoActCortex;

#[async_trait]
impl CortexPort for TwoActCortex {
    async fn cortex(
        &self,
        _senses: &[Sense],
        _physical_state: &PhysicalState,
        cognition_state: &CognitionState,
    ) -> Result<CortexOutput, CortexError> {
        Ok(CortexOutput {
            acts: vec![
                Act {
                    act_id: "0194f1f3-cc2f-7aa7-8d4c-486f9f2f7c0a".to_string(),
                    based_on: vec!["41f25f33-99f5-4250-99c3-020f8a92e199".to_string()],
                    body_endpoint_name: "ep.demo".to_string(),
                    capability_id: "cap.demo".to_string(),
                    capability_instance_id: "instance:1".to_string(),
                    normalized_payload: serde_json::json!({}),
                    requested_resources: RequestedResources {
                        survival_micro: 2_000_000,
                        time_ms: 1,
                        io_units: 1,
                        token_units: 0,
                    },
                },
                Act {
                    act_id: "act:2".to_string(),
                    based_on: vec!["41f25f33-99f5-4250-99c3-020f8a92e199".to_string()],
                    body_endpoint_name: "ep.demo".to_string(),
                    capability_id: "cap.demo".to_string(),
                    capability_instance_id: "instance:2".to_string(),
                    normalized_payload: serde_json::json!({}),
                    requested_resources: RequestedResources {
                        survival_micro: 10,
                        time_ms: 1,
                        io_units: 1,
                        token_units: 0,
                    },
                },
            ],
            new_cognition_state: cognition_state.clone(),
        })
    }
}

#[derive(Default)]
struct SpySpine {
    requests: Mutex<Vec<Act>>,
}

#[async_trait]
impl SpineExecutorPort for SpySpine {
    fn mode(&self) -> SpineExecutionMode {
        SpineExecutionMode::SerializedDeterministic
    }

    async fn dispatch_act(
        &self,
        act: Act,
    ) -> Result<EndpointExecutionOutcome, beluna::spine::SpineError> {
        self.requests.lock().await.push(act.clone());
        Ok(EndpointExecutionOutcome::Applied {
            actual_cost_micro: act.requested_resources.survival_micro.max(0),
            reference_id: format!("spy:settle:{}", act.act_id),
        })
    }

    fn capability_catalog_snapshot(&self) -> SpineCapabilityCatalog {
        SpineCapabilityCatalog::default()
    }
}

#[tokio::test]
async fn break_stops_current_act_only_and_keeps_next_act() {
    let (sense_tx, sense_rx) = mpsc::channel(4);
    sense_tx
        .send(Sense::Domain(SenseDatum {
            sense_id: "41f25f33-99f5-4250-99c3-020f8a92e199".to_string(),
            source: "test".to_string(),
            payload: serde_json::json!({}),
        }))
        .await
        .expect("domain sense should enqueue");
    sense_tx
        .send(Sense::Sleep)
        .await
        .expect("sleep should enqueue");
    drop(sense_tx);

    let spy_spine = Arc::new(SpySpine::default());
    let continuity = Arc::new(Mutex::new(ContinuityEngine::with_defaults()));
    let ledger = Arc::new(Mutex::new(LedgerStage::new(1_000)));
    let runtime = StemRuntime::new(
        Arc::new(TwoActCortex),
        continuity,
        ledger,
        spy_spine.clone(),
        sense_rx,
    );
    runtime.run().await.expect("stem should run");

    let requests = spy_spine.requests.lock().await;
    assert_eq!(requests.len(), 1, "only second act should be dispatched");
    assert_eq!(requests[0].act_id, "act:2");
}
