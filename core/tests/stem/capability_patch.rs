use std::sync::Arc;

use tokio::sync::{Mutex, mpsc};

use beluna::{
    config::SpineRuntimeConfig,
    continuity::ContinuityEngine,
    cortex::{AttemptExtractorHook, Cortex, PrimaryReasonerHook, ProseIr, ReactionLimits},
    ingress::SenseIngress,
    ledger::LedgerStage,
    spine::{EndpointCapabilityDescriptor, RouteKey, Spine},
    stem::Stem,
    types::{CapabilityPatch, PhysicalState, Sense},
};

fn test_spine() -> Arc<Spine> {
    let config = SpineRuntimeConfig { adapters: vec![] };
    Spine::new(&config, SenseIngress::new(mpsc::channel(4).0))
}

fn capture_cortex(physical_states: Arc<Mutex<Vec<PhysicalState>>>) -> Arc<Cortex> {
    let primary: PrimaryReasonerHook = Arc::new(move |req| {
        let physical_states = Arc::clone(&physical_states);
        Box::pin(async move {
            physical_states.lock().await.push(req.physical_state);
            Ok(ProseIr {
                text: "ir".to_string(),
            })
        })
    });
    let extractor: AttemptExtractorHook = Arc::new(|_req| Box::pin(async { Ok(Vec::new()) }));

    Arc::new(Cortex::for_test_with_hooks(
        primary,
        extractor,
        ReactionLimits::default(),
    ))
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

    let physical_states = Arc::new(Mutex::new(Vec::new()));
    let continuity = Arc::new(Mutex::new(ContinuityEngine::with_defaults()));
    let ledger = Arc::new(Mutex::new(LedgerStage::new(1_000)));

    let runtime = Stem::new(
        capture_cortex(Arc::clone(&physical_states)),
        continuity,
        ledger,
        test_spine(),
        sense_rx,
    );
    runtime.run().await.expect("stem should run");

    let captured = physical_states.lock().await;
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
