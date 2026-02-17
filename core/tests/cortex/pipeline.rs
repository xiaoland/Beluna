use std::sync::Arc;

use beluna::{
    cortex::{
        AttemptDraft, AttemptExtractorHook, Cortex, PrimaryReasonerHook, ProseIr, ReactionLimits,
    },
    types::{
        CognitionState, PhysicalLedgerSnapshot, PhysicalState, RequestedResources, Sense,
        SenseDatum,
    },
};

fn base_physical_state() -> PhysicalState {
    PhysicalState {
        cycle_id: 1,
        ledger: PhysicalLedgerSnapshot::default(),
        capabilities: beluna::cortex::CapabilityCatalog {
            version: "v1".to_string(),
            affordances: vec![beluna::cortex::AffordanceCapability {
                endpoint_id: "ep.demo".to_string(),
                allowed_capability_ids: vec!["cap.demo".to_string()],
                payload_schema: serde_json::json!({"type":"object"}),
                max_payload_bytes: 1024,
                default_resources: RequestedResources::default(),
            }],
        },
    }
}

fn valid_draft() -> AttemptDraft {
    AttemptDraft {
        intent_span: "run".to_string(),
        based_on: vec!["sense:1".to_string()],
        attention_tags: vec![],
        endpoint_id: "ep.demo".to_string(),
        capability_id: "cap.demo".to_string(),
        capability_instance_id: "".to_string(),
        payload_draft: serde_json::json!({"ok":true}),
        requested_resources: RequestedResources {
            survival_micro: 10,
            time_ms: 0,
            io_units: 0,
            token_units: 0,
        },
        goal_hint: None,
    }
}

fn build_pipeline(drafts: Vec<AttemptDraft>) -> Cortex {
    let primary: PrimaryReasonerHook = Arc::new(|_req| {
        Box::pin(async {
            Ok(ProseIr {
                text: "ir".to_string(),
            })
        })
    });
    let extractor: AttemptExtractorHook = Arc::new(move |_req| {
        let drafts = drafts.clone();
        Box::pin(async move { Ok(drafts) })
    });

    Cortex::for_test_with_hooks(primary, extractor, ReactionLimits::default())
}

#[tokio::test]
async fn cortex_pipeline_emits_acts_and_new_cognition() {
    let pipeline = build_pipeline(vec![valid_draft()]);

    let output = pipeline
        .cortex(
            &[Sense::Domain(SenseDatum {
                sense_id: "sense:1".to_string(),
                source: "test".to_string(),
                payload: serde_json::json!({"x":1}),
            })],
            &base_physical_state(),
            &CognitionState::default(),
        )
        .await
        .expect("pipeline should succeed");

    assert_eq!(output.acts.len(), 1);
    assert_eq!(output.new_cognition_state.revision, 1);
}

#[tokio::test]
async fn cortex_pipeline_rejects_sleep_sense() {
    let pipeline = build_pipeline(vec![]);

    let err = pipeline
        .cortex(
            &[Sense::Sleep],
            &base_physical_state(),
            &CognitionState::default(),
        )
        .await
        .expect_err("sleep should not reach cortex");
    assert_eq!(
        err.kind,
        beluna::cortex::CortexErrorKind::InvalidReactionInput
    );
}

#[tokio::test]
async fn cortex_pipeline_allows_noop_when_extractor_empty() {
    let pipeline = build_pipeline(vec![]);

    let output = pipeline
        .cortex(
            &[Sense::Domain(SenseDatum {
                sense_id: "sense:1".to_string(),
                source: "test".to_string(),
                payload: serde_json::json!({"x":1}),
            })],
            &base_physical_state(),
            &CognitionState::default(),
        )
        .await
        .expect("pipeline should succeed");

    assert!(output.acts.is_empty());
}
