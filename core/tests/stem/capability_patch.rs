use std::sync::Arc;

use tokio::sync::{Mutex, mpsc};

use beluna::{
    afferent_pathway::SenseAfferentPathway,
    config::SpineRuntimeConfig,
    continuity::ContinuityEngine,
    cortex::{
        ReactionLimits,
        testing::{TestActsHelperOutput, TestGoalStackPatch, TestHooks, boxed, cortex_with_hooks},
    },
    ledger::LedgerStage,
    spine::Spine,
    stem::Stem,
    types::{NeuralSignalDescriptor, NeuralSignalDescriptorPatch, NeuralSignalType, Sense},
};

fn test_spine() -> Arc<Spine> {
    let config = SpineRuntimeConfig { adapters: vec![] };
    Spine::new(&config, SenseAfferentPathway::new(4).0)
}

fn valid_output_ir() -> String {
    "<output-ir><acts>body</acts><goal-stack-patch>body</goal-stack-patch></output-ir>".to_string()
}

fn capture_cortex(
    act_descriptors_seen: Arc<Mutex<Vec<Vec<NeuralSignalDescriptor>>>>,
) -> Arc<beluna::cortex::Cortex> {
    let sense_helper = Arc::new(|_req| boxed(async { Ok("senses".to_string()) }));
    let act_descriptor_helper = Arc::new(
        move |req: beluna::cortex::testing::ActDescriptorHelperRequest| {
            let act_descriptors_seen = Arc::clone(&act_descriptors_seen);
            boxed(async move {
                act_descriptors_seen.lock().await.push(req.act_descriptors);
                Ok(valid_output_ir())
            })
        },
    );
    let primary = Arc::new(move |_req: beluna::cortex::testing::PrimaryRequest| {
        boxed(async move { Ok(valid_output_ir()) })
    });
    let acts_helper = Arc::new(|_req| boxed(async { Ok(TestActsHelperOutput::default()) }));
    let goal_stack_helper = Arc::new(|_req| boxed(async { Ok(TestGoalStackPatch::default()) }));

    Arc::new(cortex_with_hooks(
        TestHooks::new(
            sense_helper,
            act_descriptor_helper,
            primary,
            acts_helper,
            goal_stack_helper,
        ),
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

    let act_descriptors_seen = Arc::new(Mutex::new(Vec::new()));
    let continuity = Arc::new(Mutex::new(ContinuityEngine::with_defaults()));
    let ledger = Arc::new(Mutex::new(LedgerStage::new(1_000)));

    let runtime = Stem::new(
        capture_cortex(Arc::clone(&act_descriptors_seen)),
        continuity,
        ledger,
        test_spine(),
        sense_rx,
    );
    runtime.run().await.expect("stem should run");

    let captured = act_descriptors_seen.lock().await;
    assert_eq!(captured.len(), 1);
    let descriptor = captured[0]
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
