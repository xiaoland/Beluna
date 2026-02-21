use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::{Mutex, mpsc};

use beluna::{
    afferent_pathway::SenseAfferentPathway,
    config::SpineRuntimeConfig,
    continuity::ContinuityEngine,
    cortex::{
        ReactionLimits,
        testing::{
            TestActDraft, TestActsHelperOutput, TestGoalStackPatch, TestHooks, boxed,
            cortex_with_hooks,
        },
    },
    ledger::LedgerStage,
    spine::{ActDispatchResult, Endpoint, EndpointBinding, Spine},
    stem::Stem,
    types::{Act, NeuralSignalDescriptor, NeuralSignalType, Sense, SenseDatum},
};

#[derive(Default)]
struct SpyEndpoint {
    requests: Mutex<Vec<Act>>,
}

#[async_trait]
impl Endpoint for SpyEndpoint {
    async fn invoke(&self, act: Act) -> Result<ActDispatchResult, beluna::spine::SpineError> {
        self.requests.lock().await.push(act.clone());
        Ok(ActDispatchResult::Acknowledged {
            reference_id: format!("spy:settle:{}", act.act_id),
        })
    }
}

fn test_spine_with_spy() -> (Arc<Spine>, Arc<SpyEndpoint>, String) {
    let config = SpineRuntimeConfig { adapters: vec![] };
    let spine = Spine::new(&config, SenseAfferentPathway::new(4).0);
    let spy_endpoint = Arc::new(SpyEndpoint::default());

    let handle = spine
        .add_endpoint(
            "ep.demo",
            EndpointBinding::Inline(spy_endpoint.clone()),
            vec![NeuralSignalDescriptor {
                r#type: NeuralSignalType::Act,
                endpoint_id: "placeholder".to_string(),
                neural_signal_descriptor_id: "cap.demo".to_string(),
                payload_schema: serde_json::json!({"type":"object"}),
            }],
        )
        .expect("spy endpoint registration should succeed");

    (spine, spy_endpoint, handle.body_endpoint_id)
}

fn valid_output_ir() -> String {
    "<output-ir><acts>body</acts><goal-stack-patch>body</goal-stack-patch></output-ir>".to_string()
}

fn two_act_cortex(endpoint_id: String) -> Arc<beluna::cortex::Cortex> {
    let sense_helper = Arc::new(|_req| boxed(async { Ok("senses".to_string()) }));
    let act_descriptor_helper = Arc::new(|_req| boxed(async { Ok("catalog".to_string()) }));
    let primary = Arc::new(|_req| boxed(async { Ok(valid_output_ir()) }));
    let acts_helper = Arc::new(move |_req| {
        let endpoint_id = endpoint_id.clone();
        boxed(async move {
            Ok(TestActsHelperOutput {
                acts: vec![
                    TestActDraft {
                        endpoint_id: endpoint_id.clone(),
                        neural_signal_descriptor_id: "cap.demo".to_string(),
                        payload: serde_json::json!({"draft":"first"}),
                    },
                    TestActDraft {
                        endpoint_id,
                        neural_signal_descriptor_id: "cap.demo".to_string(),
                        payload: serde_json::json!({"draft":"second"}),
                    },
                ],
            })
        })
    });
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
async fn dispatches_all_acts_when_ledger_has_zero_reservation_policy() {
    let (sense_tx, sense_rx) = mpsc::channel(4);
    sense_tx
        .send(Sense::Domain(SenseDatum {
            sense_id: "sense:1".to_string(),
            endpoint_id: "ep.demo".to_string(),
            neural_signal_descriptor_id: "sense.demo".to_string(),
            payload: serde_json::json!({}),
        }))
        .await
        .expect("domain sense should enqueue");
    sense_tx
        .send(Sense::Sleep)
        .await
        .expect("sleep should enqueue");
    drop(sense_tx);

    let (spine, spy_endpoint, endpoint_id) = test_spine_with_spy();
    let continuity = Arc::new(Mutex::new(ContinuityEngine::with_defaults()));
    let ledger = Arc::new(Mutex::new(LedgerStage::new(1_000)));
    let runtime = Stem::new(
        two_act_cortex(endpoint_id),
        continuity,
        ledger,
        spine,
        sense_rx,
    );
    runtime.run().await.expect("stem should run");

    let requests = spy_endpoint.requests.lock().await;
    assert_eq!(requests.len(), 2, "both acts should be dispatched");
    assert!(requests.iter().any(|act| act.payload["draft"] == "first"));
    assert!(requests.iter().any(|act| act.payload["draft"] == "second"));
}
