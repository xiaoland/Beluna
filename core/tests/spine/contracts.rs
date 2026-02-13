use beluna::spine::{
    DeterministicNoopSpine, SpineErrorKind, SpineExecutionMode, SpineExecutorPort,
    types::{AdmittedAction, AdmittedActionBatch, CostVector, SpineEvent},
};

fn action(neural_signal_id: &str, reserve_entry_id: &str) -> AdmittedAction {
    AdmittedAction {
        neural_signal_id: neural_signal_id.to_string(),
        capability_instance_id: format!("instance:{neural_signal_id}"),
        source_attempt_id: "att:1".to_string(),
        reserve_entry_id: reserve_entry_id.to_string(),
        cost_attribution_id: "cat:1".to_string(),
        endpoint_id: "deliberate.plan".to_string(),
        capability_id: "cap.core".to_string(),
        normalized_payload: serde_json::json!({"k": "v"}),
        reserved_cost: CostVector {
            survival_micro: 10,
            time_ms: 1,
            io_units: 1,
            token_units: 1,
        },
        degraded: false,
        degradation_profile_id: None,
        admission_cycle: 1,
        metadata: Default::default(),
    }
}

#[tokio::test]
async fn given_invalid_admitted_action_when_execute_then_batch_is_rejected() {
    let spine = DeterministicNoopSpine::default();
    let err = spine
        .execute_admitted(AdmittedActionBatch {
            cycle_id: 1,
            actions: vec![action("", "")],
        })
        .await
        .expect_err("batch should be rejected");

    assert_eq!(err.kind, SpineErrorKind::InvalidBatch);
}

#[tokio::test]
async fn given_admitted_actions_when_execute_then_events_are_ordered_and_settlement_linked() {
    let spine = DeterministicNoopSpine::new(SpineExecutionMode::SerializedDeterministic);
    let report = spine
        .execute_admitted(AdmittedActionBatch {
            cycle_id: 1,
            actions: vec![action("act:1", "resv:1"), action("act:2", "resv:2")],
        })
        .await
        .expect("execution should succeed");

    assert_eq!(report.mode, SpineExecutionMode::SerializedDeterministic);
    assert!(
        report
            .events
            .windows(2)
            .all(|window| window[0].seq_no < window[1].seq_no)
    );

    for ordered in report.events {
        match ordered.event {
            SpineEvent::ActionApplied {
                reserve_entry_id,
                reference_id,
                ..
            } => {
                assert!(!reserve_entry_id.is_empty());
                assert!(reference_id.starts_with("noop:settle:"));
            }
            _ => panic!("deterministic noop spine should only emit ActionApplied"),
        }
    }
}

#[tokio::test]
async fn given_best_effort_mode_when_execute_then_mode_is_reported_without_losing_ordering() {
    let spine = DeterministicNoopSpine::new(SpineExecutionMode::BestEffortReplayable);
    let report = spine
        .execute_admitted(AdmittedActionBatch {
            cycle_id: 2,
            actions: vec![action("act:1", "resv:1")],
        })
        .await
        .expect("execution should succeed");

    assert_eq!(report.mode, SpineExecutionMode::BestEffortReplayable);
    assert_eq!(report.events[0].seq_no, 1);
    assert!(report.replay_cursor.is_some());
}
