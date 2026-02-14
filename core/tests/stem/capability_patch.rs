use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::{Mutex, mpsc};

use beluna::{
    continuity::ContinuityEngine,
    cortex::{CortexError, CortexOutput, CortexPort},
    ledger::LedgerStage,
    runtime_types::{CapabilityPatch, CognitionState, PhysicalState, Sense},
    spine::{DeterministicNoopSpine, EndpointCapabilityDescriptor, RouteKey, SpineExecutorPort},
    stem::StemRuntime,
};

#[derive(Default)]
struct CaptureCortex {
    physical_states: Mutex<Vec<PhysicalState>>,
}

#[async_trait]
impl CortexPort for CaptureCortex {
    async fn cortex(
        &self,
        _senses: &[Sense],
        physical_state: &PhysicalState,
        cognition_state: &CognitionState,
    ) -> Result<CortexOutput, CortexError> {
        self.physical_states
            .lock()
            .await
            .push(physical_state.clone());
        Ok(CortexOutput {
            acts: Vec::new(),
            new_cognition_state: cognition_state.clone(),
        })
    }
}

#[tokio::test]
async fn new_capabilities_patch_takes_effect_before_cortex() {
    let (sense_tx, sense_rx) = mpsc::channel(4);
    sense_tx
        .send(Sense::NewCapabilities(CapabilityPatch {
            entries: vec![EndpointCapabilityDescriptor {
                route: RouteKey {
                    endpoint_id: "ep.patch".to_string(),
                    capability_id: "cap.patch".to_string(),
                },
                payload_schema: serde_json::json!({"type":"object"}),
                max_payload_bytes: 1024,
                default_cost: beluna::spine::CostVector::default(),
                metadata: Default::default(),
            }],
        }))
        .await
        .expect("patch should be enqueued");
    sense_tx
        .send(Sense::Sleep)
        .await
        .expect("sleep should be enqueued");
    drop(sense_tx);

    let cortex = Arc::new(CaptureCortex::default());
    let continuity = Arc::new(Mutex::new(ContinuityEngine::with_defaults()));
    let ledger = Arc::new(Mutex::new(LedgerStage::new(1_000)));
    let spine: Arc<dyn SpineExecutorPort> = Arc::new(DeterministicNoopSpine::default());

    let runtime = StemRuntime::new(cortex.clone(), continuity, ledger, spine, sense_rx);
    runtime.run().await.expect("stem should run");

    let captured = cortex.physical_states.lock().await;
    assert_eq!(captured.len(), 1);
    let affordance = captured[0]
        .capabilities
        .resolve("ep.patch")
        .expect("patched capability should be visible to cortex");
    assert!(
        affordance
            .allowed_capability_ids
            .iter()
            .any(|value| value == "cap.patch")
    );
}
