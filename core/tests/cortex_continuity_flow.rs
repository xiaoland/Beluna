use std::sync::{Arc, Mutex};

use beluna::{
    admission::{
        AdmissionDisposition, AdmissionResolver, AdmissionResolverConfig, AffordanceProfile,
        AffordanceRegistry, CostAdmissionPolicy, IntentAttempt,
    },
    continuity::{ContinuityEngine, ContinuityError, ContinuityState, NoopDebitSource, SpinePort},
    cortex::{CortexCommand, CortexFacade, Goal, GoalClass, GoalScope},
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

fn goal() -> Goal {
    Goal {
        id: "g1".to_string(),
        title: "Run full boundary loop".to_string(),
        class: GoalClass::Primary,
        scope: GoalScope::Strategic,
        parent_goal_id: None,
        metadata: Default::default(),
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

#[test]
fn cortex_continuity_spine_flow_preserves_contracts() {
    let mut cortex = CortexFacade::with_defaults();
    cortex
        .step(CortexCommand::ProposeGoal(goal()))
        .expect("goal proposal should succeed");
    cortex
        .step(CortexCommand::CommitGoal {
            goal_id: "g1".to_string(),
            commitment_id: Some("c1".to_string()),
        })
        .expect("commitment should succeed");

    let cycle_out = cortex
        .step(CortexCommand::PlanNow)
        .expect("planning should succeed");

    let mut attempts = cycle_out.attempts;
    attempts.push(IntentAttempt {
        attempt_id: "att:denied".to_string(),
        cycle_id: cycle_out.cycle_id,
        commitment_id: "c1".to_string(),
        goal_id: "g1".to_string(),
        planner_slot: 999,
        affordance_key: "unknown.affordance".to_string(),
        capability_handle: "cap.core".to_string(),
        normalized_payload: serde_json::json!({"invalid": true}),
        requested_resources: Default::default(),
        cost_attribution_id: "cat:denied".to_string(),
    });

    let spine = Arc::new(RecordingScrambledSpine::default());
    let mut continuity = continuity_engine(Arc::clone(&spine), ContinuityState::new(100_000));

    let output = continuity
        .process_attempts(cycle_out.cycle_id, attempts)
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

    let mut replacement_cortex = CortexFacade::with_defaults();
    replacement_cortex
        .step(CortexCommand::ProposeGoal(goal()))
        .expect("replacement cortex should still plan");
    replacement_cortex
        .step(CortexCommand::CommitGoal {
            goal_id: "g1".to_string(),
            commitment_id: Some("c1".to_string()),
        })
        .expect("replacement cortex should still commit");
    let replacement_cycle = replacement_cortex
        .step(CortexCommand::PlanNow)
        .expect("replacement cortex planning should succeed");

    continuity
        .process_attempts(
            replacement_cycle.cycle_id + cycle_out.cycle_id,
            replacement_cycle.attempts,
        )
        .expect("continuity should hold across cortex replacement");

    assert!(continuity.state().ledger.balance_survival_micro() <= balance_after_first);
}
