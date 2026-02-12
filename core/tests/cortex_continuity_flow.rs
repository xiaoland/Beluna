use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use beluna::{
    admission::{
        AdmissionDisposition, AdmissionResolver, AdmissionResolverConfig, AffordanceProfile,
        AffordanceRegistry, CostAdmissionPolicy, IntentAttempt,
    },
    continuity::{ContinuityEngine, ContinuityError, ContinuityState, NoopDebitSource, SpinePort},
    cortex::{
        AttemptDraft, AttemptExtractorPort, AttemptExtractorRequest, CapabilityCatalog,
        CortexPipeline, CortexReactor, DeterministicAttemptClamp, NoopTelemetryPort,
        PayloadFillerPort, PayloadFillerRequest, PrimaryReasonerPort, PrimaryReasonerRequest,
        ProseIr, ReactionInput, ReactionLimits, SenseDelta,
    },
    spine::types::{
        AdmittedActionBatch, OrderedSpineEvent, SpineEvent, SpineExecutionMode,
        SpineExecutionReport,
    },
};

#[derive(Default)]
struct RecordingScrambledSpine {
    batches: Mutex<Vec<AdmittedActionBatch>>,
}

impl RecordingScrambledSpine {
    fn admitted_batches(&self) -> Vec<AdmittedActionBatch> {
        self.batches.lock().expect("lock").clone()
    }
}

impl SpinePort for RecordingScrambledSpine {
    fn execute_admitted(
        &self,
        admitted: AdmittedActionBatch,
    ) -> Result<SpineExecutionReport, ContinuityError> {
        self.batches.lock().expect("lock").push(admitted.clone());

        let mut events: Vec<OrderedSpineEvent> = admitted
            .actions
            .iter()
            .enumerate()
            .map(|(index, action)| OrderedSpineEvent {
                seq_no: (index as u64) + 1,
                event: SpineEvent::ActionApplied {
                    action_id: action.action_id.clone(),
                    reserve_entry_id: action.reserve_entry_id.clone(),
                    cost_attribution_id: action.cost_attribution_id.clone(),
                    actual_cost_micro: action.reserved_cost.survival_micro,
                    reference_id: format!("scrambled:settle:{}", action.action_id),
                },
            })
            .collect();

        events.reverse();

        Ok(SpineExecutionReport {
            mode: SpineExecutionMode::BestEffortReplayable,
            events,
            replay_cursor: Some(format!("cursor:{}", admitted.cycle_id)),
        })
    }
}

struct StaticPrimary;

#[async_trait]
impl PrimaryReasonerPort for StaticPrimary {
    async fn infer_ir(&self, _req: PrimaryReasonerRequest) -> Result<ProseIr, beluna::cortex::CortexError> {
        Ok(ProseIr {
            text: "compose attempt".to_string(),
        })
    }
}

struct StaticExtractor;

#[async_trait]
impl AttemptExtractorPort for StaticExtractor {
    async fn extract(
        &self,
        _req: AttemptExtractorRequest,
    ) -> Result<Vec<AttemptDraft>, beluna::cortex::CortexError> {
        Ok(vec![AttemptDraft {
            intent_span: "attempt work".to_string(),
            based_on: vec!["s1".to_string()],
            attention_tags: vec!["plan".to_string()],
            affordance_key: "deliberate.plan".to_string(),
            capability_handle: "cap.core".to_string(),
            payload_draft: serde_json::json!({"ok": true}),
            requested_resources: Default::default(),
            commitment_hint: Some("c1".to_string()),
            goal_hint: Some("g1".to_string()),
        }])
    }
}

struct PassthroughFiller;

#[async_trait]
impl PayloadFillerPort for PassthroughFiller {
    async fn fill(
        &self,
        req: PayloadFillerRequest,
    ) -> Result<Vec<AttemptDraft>, beluna::cortex::CortexError> {
        Ok(req.drafts)
    }
}

fn continuity_engine(
    spine: Arc<RecordingScrambledSpine>,
    state: ContinuityState,
) -> ContinuityEngine {
    ContinuityEngine::new(
        state,
        AdmissionResolver::new(
            AffordanceRegistry::new(vec![AffordanceProfile::default()]),
            CostAdmissionPolicy::default(),
            AdmissionResolverConfig::default(),
        ),
        spine,
        Arc::new(NoopDebitSource),
    )
}

fn reaction_input(reaction_id: u64) -> ReactionInput {
    ReactionInput {
        reaction_id,
        sense_window: vec![SenseDelta {
            sense_id: "s1".to_string(),
            source: "sensor".to_string(),
            payload: serde_json::json!({"v": 1}),
        }],
        env_snapshots: vec![],
        admission_feedback: vec![],
        capability_catalog: CapabilityCatalog {
            version: "v1".to_string(),
            affordances: vec![beluna::cortex::AffordanceCapability {
                affordance_key: "deliberate.plan".to_string(),
                allowed_capability_handles: vec!["cap.core".to_string()],
                payload_schema: serde_json::json!({"type":"object"}),
                max_payload_bytes: 4096,
                default_resources: Default::default(),
            }],
        },
        limits: ReactionLimits::default(),
        context: Default::default(),
    }
}

fn reactor() -> CortexReactor {
    CortexReactor::new(CortexPipeline::new(
        Arc::new(StaticPrimary),
        Arc::new(StaticExtractor),
        Arc::new(PassthroughFiller),
        Arc::new(DeterministicAttemptClamp),
        Arc::new(NoopTelemetryPort),
    ))
}

#[tokio::test]
async fn cortex_continuity_spine_flow_preserves_contracts() {
    let cycle_out = reactor().react_once(reaction_input(1)).await;

    let mut attempts = cycle_out.attempts;
    attempts.push(IntentAttempt {
        attempt_id: "att:denied".to_string(),
        cycle_id: cycle_out.reaction_id,
        commitment_id: "c1".to_string(),
        goal_id: "g1".to_string(),
        planner_slot: 999,
        based_on: vec!["s1".to_string()],
        affordance_key: "unknown.affordance".to_string(),
        capability_handle: "cap.core".to_string(),
        normalized_payload: serde_json::json!({"invalid": true}),
        requested_resources: Default::default(),
        cost_attribution_id: "cat:denied".to_string(),
    });

    let spine = Arc::new(RecordingScrambledSpine::default());
    let mut continuity = continuity_engine(Arc::clone(&spine), ContinuityState::new(100_000));

    let output = continuity
        .process_attempts(cycle_out.reaction_id, attempts)
        .expect("admission + reconciliation should succeed");

    assert!(output.admitted_action_count > 0);
    assert!(
        output
            .admission_report
            .outcomes
            .iter()
            .any(|item| matches!(item.disposition, AdmissionDisposition::DeniedHard { .. }))
    );

    let recorded = spine.admitted_batches();
    assert_eq!(recorded.len(), 1);
    assert_eq!(recorded[0].actions.len(), output.admitted_action_count);

    let first_action = &recorded[0].actions[0];
    let attribution_records = continuity
        .state()
        .attribution_index
        .get(&first_action.cost_attribution_id)
        .expect("attribution record should exist");
    assert!(
        attribution_records
            .iter()
            .any(|record| record.action_id == first_action.action_id)
    );

    let balance_after_first = continuity.state().ledger.balance_survival_micro();

    let replacement_cycle = reactor().react_once(reaction_input(2)).await;

    continuity
        .process_attempts(
            replacement_cycle.reaction_id + cycle_out.reaction_id,
            replacement_cycle.attempts,
        )
        .expect("continuity should hold across cortex replacement");

    assert!(continuity.state().ledger.balance_survival_micro() <= balance_after_first);
}
