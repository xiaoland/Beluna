use std::sync::Arc;

use async_trait::async_trait;

use beluna::{
    cortex::{
        AttemptDraft, AttemptExtractorPort, AttemptExtractorRequest, CortexPipeline, CortexPort,
        NoopTelemetryPort, PrimaryReasonerPort, PrimaryReasonerRequest, ProseIr, ReactionLimits,
    },
    runtime_types::{
        CognitionState, PhysicalLedgerSnapshot, PhysicalState, RequestedResources, Sense,
        SenseDatum,
    },
};

#[derive(Clone)]
struct StaticPrimary;

#[async_trait]
impl PrimaryReasonerPort for StaticPrimary {
    async fn infer_ir(
        &self,
        _req: PrimaryReasonerRequest,
    ) -> Result<ProseIr, beluna::cortex::CortexError> {
        Ok(ProseIr {
            text: "ir".to_string(),
        })
    }
}

#[derive(Clone)]
struct StaticExtractor {
    drafts: Vec<AttemptDraft>,
}

#[async_trait]
impl AttemptExtractorPort for StaticExtractor {
    async fn extract(
        &self,
        _req: AttemptExtractorRequest,
    ) -> Result<Vec<AttemptDraft>, beluna::cortex::CortexError> {
        Ok(self.drafts.clone())
    }
}

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
        based_on: vec!["41f25f33-99f5-4250-99c3-020f8a92e199".to_string()],
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

#[tokio::test]
async fn cortex_pipeline_emits_acts_and_new_cognition() {
    let pipeline = CortexPipeline::new(
        Arc::new(StaticPrimary),
        Arc::new(StaticExtractor {
            drafts: vec![valid_draft()],
        }),
        Arc::new(NoopTelemetryPort),
        ReactionLimits::default(),
    );

    let output = pipeline
        .cortex(
            &[Sense::Domain(SenseDatum {
                sense_id: "41f25f33-99f5-4250-99c3-020f8a92e199".to_string(),
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
    let pipeline = CortexPipeline::new(
        Arc::new(StaticPrimary),
        Arc::new(StaticExtractor { drafts: vec![] }),
        Arc::new(NoopTelemetryPort),
        ReactionLimits::default(),
    );

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
    let limits = ReactionLimits::default();
    let pipeline = CortexPipeline::new(
        Arc::new(StaticPrimary),
        Arc::new(StaticExtractor { drafts: vec![] }),
        Arc::new(NoopTelemetryPort),
        limits,
    );

    let output = pipeline
        .cortex(
            &[Sense::Domain(SenseDatum {
                sense_id: "41f25f33-99f5-4250-99c3-020f8a92e199".to_string(),
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
