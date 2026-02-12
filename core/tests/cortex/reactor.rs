use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use async_trait::async_trait;
use beluna::cortex::{
    AttemptDraft, AttemptExtractorPort, AttemptExtractorRequest, CapabilityCatalog,
    ConstitutionalIntent, CortexPipeline, CortexReactor, DeterministicAttemptClamp,
    EnvironmentalIntentSignal, IntentContext, NoopTelemetryPort, PayloadFillerPort,
    PayloadFillerRequest, PrimaryReasonerPort, PrimaryReasonerRequest, ProseIr, ReactionInput,
    ReactionLimits, SenseDelta,
};
use tokio::sync::mpsc;

struct StaticPrimary {
    calls: Arc<AtomicUsize>,
}

#[async_trait]
impl PrimaryReasonerPort for StaticPrimary {
    async fn infer_ir(&self, _req: PrimaryReasonerRequest) -> Result<ProseIr, beluna::cortex::CortexError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(ProseIr {
            text: "intent: plan action".to_string(),
        })
    }
}

struct StaticExtractor {
    calls: Arc<AtomicUsize>,
    drafts: Vec<AttemptDraft>,
}

#[async_trait]
impl AttemptExtractorPort for StaticExtractor {
    async fn extract(
        &self,
        _req: AttemptExtractorRequest,
    ) -> Result<Vec<AttemptDraft>, beluna::cortex::CortexError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(self.drafts.clone())
    }
}

struct StaticFiller {
    calls: Arc<AtomicUsize>,
    drafts: Vec<AttemptDraft>,
}

#[async_trait]
impl PayloadFillerPort for StaticFiller {
    async fn fill(
        &self,
        _req: PayloadFillerRequest,
    ) -> Result<Vec<AttemptDraft>, beluna::cortex::CortexError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(self.drafts.clone())
    }
}

fn base_catalog() -> CapabilityCatalog {
    CapabilityCatalog {
        version: "v1".to_string(),
        affordances: vec![beluna::cortex::AffordanceCapability {
            affordance_key: "deliberate.plan".to_string(),
            allowed_capability_handles: vec!["cap.core".to_string()],
            payload_schema: serde_json::json!({"type":"object"}),
            max_payload_bytes: 4096,
            default_resources: Default::default(),
        }],
    }
}

fn base_input(reaction_id: u64, limits: ReactionLimits) -> ReactionInput {
    ReactionInput {
        reaction_id,
        sense_window: vec![SenseDelta {
            sense_id: "s1".to_string(),
            source: "sensor".to_string(),
            payload: serde_json::json!({"v": 1}),
        }],
        env_snapshots: vec![],
        admission_feedback: vec![],
        capability_catalog: base_catalog(),
        limits,
        context: IntentContext {
            constitutional: vec![ConstitutionalIntent {
                intent_key: "survive".to_string(),
                description: "preserve runtime".to_string(),
            }],
            environmental: vec![EnvironmentalIntentSignal {
                signal_key: "budget".to_string(),
                constraint_code: "ok".to_string(),
                payload: serde_json::json!({}),
            }],
            emergent_candidates: vec![],
        },
    }
}

fn valid_draft() -> AttemptDraft {
    AttemptDraft {
        intent_span: "plan a safe action".to_string(),
        based_on: vec!["s1".to_string()],
        attention_tags: vec!["planning".to_string()],
        affordance_key: "deliberate.plan".to_string(),
        capability_handle: "cap.core".to_string(),
        payload_draft: serde_json::json!({"task":"ok"}),
        requested_resources: Default::default(),
        commitment_hint: Some("c1".to_string()),
        goal_hint: Some("g1".to_string()),
    }
}

#[tokio::test]
async fn given_valid_input_when_reacting_then_primary_is_called_once_and_attempts_are_emitted() {
    let primary_calls = Arc::new(AtomicUsize::new(0));
    let extractor_calls = Arc::new(AtomicUsize::new(0));
    let filler_calls = Arc::new(AtomicUsize::new(0));

    let pipeline = CortexPipeline::new(
        Arc::new(StaticPrimary {
            calls: Arc::clone(&primary_calls),
        }),
        Arc::new(StaticExtractor {
            calls: Arc::clone(&extractor_calls),
            drafts: vec![valid_draft()],
        }),
        Arc::new(StaticFiller {
            calls: Arc::clone(&filler_calls),
            drafts: vec![valid_draft()],
        }),
        Arc::new(DeterministicAttemptClamp),
        Arc::new(NoopTelemetryPort),
    );
    let reactor = CortexReactor::new(pipeline);

    let result = reactor.react_once(base_input(1, ReactionLimits::default())).await;

    assert_eq!(primary_calls.load(Ordering::SeqCst), 1);
    assert_eq!(extractor_calls.load(Ordering::SeqCst), 1);
    assert_eq!(filler_calls.load(Ordering::SeqCst), 0);
    assert!(!result.attempts.is_empty());
    assert_eq!(result.attempts[0].based_on, vec!["s1".to_string()]);
}

#[tokio::test]
async fn given_first_clamp_empty_when_repair_allowed_then_filler_runs_once() {
    let primary_calls = Arc::new(AtomicUsize::new(0));
    let extractor_calls = Arc::new(AtomicUsize::new(0));
    let filler_calls = Arc::new(AtomicUsize::new(0));

    let mut limits = ReactionLimits::default();
    limits.max_sub_calls = 2;
    limits.max_repair_attempts = 1;

    let invalid_draft = AttemptDraft {
        affordance_key: "unknown.affordance".to_string(),
        ..valid_draft()
    };

    let pipeline = CortexPipeline::new(
        Arc::new(StaticPrimary {
            calls: Arc::clone(&primary_calls),
        }),
        Arc::new(StaticExtractor {
            calls: Arc::clone(&extractor_calls),
            drafts: vec![invalid_draft],
        }),
        Arc::new(StaticFiller {
            calls: Arc::clone(&filler_calls),
            drafts: vec![valid_draft()],
        }),
        Arc::new(DeterministicAttemptClamp),
        Arc::new(NoopTelemetryPort),
    );
    let reactor = CortexReactor::new(pipeline);

    let result = reactor.react_once(base_input(2, limits)).await;
    assert_eq!(primary_calls.load(Ordering::SeqCst), 1);
    assert_eq!(extractor_calls.load(Ordering::SeqCst), 1);
    assert_eq!(filler_calls.load(Ordering::SeqCst), 1);
    assert!(!result.attempts.is_empty());
}

#[tokio::test]
async fn given_subcall_budget_exhausted_when_repair_needed_then_cycle_falls_back_to_noop() {
    let mut limits = ReactionLimits::default();
    limits.max_sub_calls = 1;
    limits.max_repair_attempts = 1;

    let pipeline = CortexPipeline::new(
        Arc::new(StaticPrimary {
            calls: Arc::new(AtomicUsize::new(0)),
        }),
        Arc::new(StaticExtractor {
            calls: Arc::new(AtomicUsize::new(0)),
            drafts: vec![AttemptDraft {
                affordance_key: "unknown.affordance".to_string(),
                ..valid_draft()
            }],
        }),
        Arc::new(StaticFiller {
            calls: Arc::new(AtomicUsize::new(0)),
            drafts: vec![valid_draft()],
        }),
        Arc::new(DeterministicAttemptClamp),
        Arc::new(NoopTelemetryPort),
    );
    let reactor = CortexReactor::new(pipeline);

    let result = reactor.react_once(base_input(3, limits)).await;
    assert!(result.attempts.is_empty());
}

#[tokio::test]
async fn given_two_inputs_when_running_reactor_then_each_input_produces_one_output() {
    let pipeline = CortexPipeline::new(
        Arc::new(StaticPrimary {
            calls: Arc::new(AtomicUsize::new(0)),
        }),
        Arc::new(StaticExtractor {
            calls: Arc::new(AtomicUsize::new(0)),
            drafts: vec![valid_draft()],
        }),
        Arc::new(StaticFiller {
            calls: Arc::new(AtomicUsize::new(0)),
            drafts: vec![valid_draft()],
        }),
        Arc::new(DeterministicAttemptClamp),
        Arc::new(NoopTelemetryPort),
    );
    let reactor = CortexReactor::new(pipeline);

    let (in_tx, in_rx) = mpsc::channel(8);
    let (out_tx, mut out_rx) = mpsc::channel(8);

    let handle = tokio::spawn(async move {
        reactor.run(in_rx, out_tx).await;
    });

    in_tx
        .send(base_input(10, ReactionLimits::default()))
        .await
        .expect("send should succeed");
    in_tx
        .send(base_input(11, ReactionLimits::default()))
        .await
        .expect("send should succeed");
    drop(in_tx);

    let first = out_rx.recv().await.expect("first output");
    let second = out_rx.recv().await.expect("second output");
    assert_eq!(first.reaction_id, 10);
    assert_eq!(second.reaction_id, 11);

    handle.await.expect("reactor task should finish");
}
