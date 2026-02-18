use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::{Mutex, mpsc};

use beluna::{
    afferent_pathway::SenseAfferentPathway,
    config::SpineRuntimeConfig,
    continuity::ContinuityEngine,
    cortex::{
        AttemptDraft, AttemptExtractorHook, Cortex, PrimaryReasonerHook, ProseIr, ReactionLimits,
    },
    ledger::LedgerStage,
    spine::{
        CostVector, Endpoint, EndpointBinding, EndpointCapabilityDescriptor,
        EndpointExecutionOutcome, RouteKey, Spine,
    },
    stem::Stem,
    types::{Act, RequestedResources, Sense, SenseDatum},
};

#[derive(Default)]
struct SpyEndpoint {
    requests: Mutex<Vec<Act>>,
}

#[async_trait]
impl Endpoint for SpyEndpoint {
    async fn invoke(
        &self,
        act: Act,
    ) -> Result<EndpointExecutionOutcome, beluna::spine::SpineError> {
        self.requests.lock().await.push(act.clone());
        Ok(EndpointExecutionOutcome::Applied {
            actual_cost_micro: act.requested_resources.survival_micro.max(0),
            reference_id: format!("spy:settle:{}", act.act_id),
        })
    }
}

fn test_spine_with_spy() -> (Arc<Spine>, Arc<SpyEndpoint>) {
    let config = SpineRuntimeConfig { adapters: vec![] };
    let spine = Spine::new(&config, SenseAfferentPathway::new(4).0);
    let spy_endpoint = Arc::new(SpyEndpoint::default());

    spine
        .add_endpoint(
            "ep.demo",
            EndpointBinding::Inline(spy_endpoint.clone()),
            vec![EndpointCapabilityDescriptor {
                route: RouteKey {
                    endpoint_id: "placeholder".to_string(),
                    capability_id: "cap.demo".to_string(),
                },
                payload_schema: serde_json::json!({"type":"object"}),
                max_payload_bytes: 1024,
                default_cost: CostVector::default(),
                metadata: Default::default(),
            }],
        )
        .expect("spy endpoint registration should succeed");

    (spine, spy_endpoint)
}

fn two_act_cortex() -> Arc<Cortex> {
    let primary: PrimaryReasonerHook = Arc::new(|_req| {
        Box::pin(async {
            Ok(ProseIr {
                text: "ir".to_string(),
            })
        })
    });
    let extractor: AttemptExtractorHook = Arc::new(|_req| {
        Box::pin(async {
            Ok(vec![
                AttemptDraft {
                    intent_span: "run".to_string(),
                    based_on: vec!["sense:1".to_string()],
                    attention_tags: vec![],
                    endpoint_id: "ep.demo".to_string(),
                    capability_id: "cap.demo".to_string(),
                    capability_instance_id: "instance:1".to_string(),
                    payload_draft: serde_json::json!({}),
                    requested_resources: RequestedResources {
                        survival_micro: 2_000_000,
                        time_ms: 1,
                        io_units: 1,
                        token_units: 0,
                    },
                    goal_hint: None,
                },
                AttemptDraft {
                    intent_span: "run".to_string(),
                    based_on: vec!["sense:1".to_string()],
                    attention_tags: vec![],
                    endpoint_id: "ep.demo".to_string(),
                    capability_id: "cap.demo".to_string(),
                    capability_instance_id: "instance:2".to_string(),
                    payload_draft: serde_json::json!({}),
                    requested_resources: RequestedResources {
                        survival_micro: 10,
                        time_ms: 1,
                        io_units: 1,
                        token_units: 0,
                    },
                    goal_hint: None,
                },
            ])
        })
    });

    Arc::new(Cortex::for_test_with_hooks(
        primary,
        extractor,
        ReactionLimits::default(),
    ))
}

#[tokio::test]
async fn break_stops_current_act_only_and_keeps_next_act() {
    let (sense_tx, sense_rx) = mpsc::channel(4);
    sense_tx
        .send(Sense::Domain(SenseDatum {
            sense_id: "sense:1".to_string(),
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

    let (spine, spy_endpoint) = test_spine_with_spy();
    let continuity = Arc::new(Mutex::new(ContinuityEngine::with_defaults()));
    let ledger = Arc::new(Mutex::new(LedgerStage::new(1_000)));
    let runtime = Stem::new(two_act_cortex(), continuity, ledger, spine, sense_rx);
    runtime.run().await.expect("stem should run");

    let requests = spy_endpoint.requests.lock().await;
    assert_eq!(requests.len(), 1, "only one act should be dispatched");
    assert_eq!(requests[0].requested_resources.survival_micro, 10);
}
