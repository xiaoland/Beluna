use std::sync::Arc;

use tokio::sync::{Mutex, mpsc};

use beluna::{
    afferent_pathway::SenseAfferentPathway,
    config::SpineRuntimeConfig,
    continuity::ContinuityEngine,
    cortex::{AttemptExtractorHook, Cortex, PrimaryReasonerHook, ProseIr, ReactionLimits},
    ledger::LedgerStage,
    spine::Spine,
    stem::Stem,
    types::{
        NeuralSignalDescriptor, NeuralSignalDescriptorPatch, NeuralSignalType, PhysicalState, Sense,
    },
};

fn test_spine() -> Arc<Spine> {
    let config = SpineRuntimeConfig { adapters: vec![] };
    Spine::new(&config, SenseAfferentPathway::new(4).0)
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
        .send(Sense::NewNeuralSignalDescriptors(
            NeuralSignalDescriptorPatch {
                entries: vec![NeuralSignalDescriptor {
                    r#type: NeuralSignalType::Act,
                    endpoint_id: "ep.patch".to_string(),
                    neural_signal_descriptor_id: "cap.patch".to_string(),
                    payload_schema: serde_json::json!({"type":"object"}),
                }],
            },
        ))
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
    let descriptor = captured[0]
        .capabilities
        .entries
        .iter()
        .find(|entry| {
            entry.r#type == NeuralSignalType::Act
                && entry.endpoint_id == "ep.patch"
                && entry.neural_signal_descriptor_id == "cap.patch"
        })
        .expect("patched capability should be visible to cortex");
    assert_eq!(
        descriptor.payload_schema["type"],
        serde_json::json!("object")
    );
}
