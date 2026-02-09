use beluna::mind::{
    DeterministicPreemptionDecider, GoalManager, MindState, PreemptionContext, PreemptionDecider,
    SafePoint,
    error::MindErrorKind,
    types::{
        EvaluationCriterion, EvaluationReport, EvaluationVerdict, Judgment, PreemptionDisposition,
    },
};

use super::high_goal;

fn state_with_active_goal() -> (MindState, beluna::mind::Goal) {
    let mut state = MindState::default();
    let active = high_goal("active", 1);
    GoalManager::register_goal(&mut state, active.clone()).expect("goal should register");
    GoalManager::activate_goal(&mut state, &active.id).expect("goal should activate");
    (state, active)
}

#[test]
fn given_non_preemptable_safe_point_when_new_goal_then_continue() {
    let (state, active) = state_with_active_goal();
    let mut incoming = high_goal("incoming", 9);
    incoming.parent_goal_id = Some("other".to_string());
    let decider = DeterministicPreemptionDecider::default();

    let decision = decider
        .decide(PreemptionContext {
            state: &state,
            active_goal: state.goals.get(&active.id),
            incoming_goal: &incoming,
            safe_point: SafePoint {
                preemptable: false,
                checkpoint_token: None,
                rationale: "not preemptable".to_string(),
            },
        })
        .expect("decision should succeed");

    assert_eq!(decision.disposition, PreemptionDisposition::Continue);
}

#[test]
fn given_higher_priority_and_preemptable_when_new_goal_then_pause() {
    let (state, active) = state_with_active_goal();
    let mut incoming = high_goal("incoming", 9);
    incoming.parent_goal_id = Some("different-scope".to_string());
    let decider = DeterministicPreemptionDecider::default();

    let decision = decider
        .decide(PreemptionContext {
            state: &state,
            active_goal: state.goals.get(&active.id),
            incoming_goal: &incoming,
            safe_point: SafePoint {
                preemptable: true,
                checkpoint_token: Some("cp:ok".to_string()),
                rationale: "safe".to_string(),
            },
        })
        .expect("decision should succeed");

    assert_eq!(decision.disposition, PreemptionDisposition::Pause);
}

#[test]
fn given_reliability_failures_when_new_goal_then_cancel() {
    let (mut state, active) = state_with_active_goal();
    let mut incoming = high_goal("incoming", 1);
    incoming.parent_goal_id = Some("different-scope".to_string());

    for _ in 0..2 {
        state.push_evaluation(EvaluationReport {
            goal_id: Some(active.id.clone()),
            judgments: vec![Judgment {
                criterion: EvaluationCriterion::SubsystemReliability,
                verdict: EvaluationVerdict::Fail,
                confidence: 0.9,
                rationale: "reliability fail".to_string(),
                evidence_refs: vec!["r1".to_string()],
            }],
        });
    }

    let decider = DeterministicPreemptionDecider {
        reliability_failure_window: 2,
    };

    let decision = decider
        .decide(PreemptionContext {
            state: &state,
            active_goal: state.goals.get(&active.id),
            incoming_goal: &incoming,
            safe_point: SafePoint {
                preemptable: true,
                checkpoint_token: Some("cp:ok".to_string()),
                rationale: "safe".to_string(),
            },
        })
        .expect("decision should succeed");

    assert_eq!(decision.disposition, PreemptionDisposition::Cancel);
}

#[test]
fn given_merge_compatible_goals_when_new_goal_then_merge() {
    let (state, active) = state_with_active_goal();
    let incoming = high_goal("incoming", 1);
    let decider = DeterministicPreemptionDecider::default();

    let decision = decider
        .decide(PreemptionContext {
            state: &state,
            active_goal: state.goals.get(&active.id),
            incoming_goal: &incoming,
            safe_point: SafePoint {
                preemptable: true,
                checkpoint_token: Some("cp:ok".to_string()),
                rationale: "safe".to_string(),
            },
        })
        .expect("decision should succeed");

    assert_eq!(decision.disposition, PreemptionDisposition::Merge);
    assert!(decision.merge_goal_id.is_some());
}

#[test]
fn given_checkpoint_without_preemptable_when_decide_then_invalid_request() {
    let (state, active) = state_with_active_goal();
    let incoming = high_goal("incoming", 1);
    let decider = DeterministicPreemptionDecider::default();

    let err = decider
        .decide(PreemptionContext {
            state: &state,
            active_goal: state.goals.get(&active.id),
            incoming_goal: &incoming,
            safe_point: SafePoint {
                preemptable: false,
                checkpoint_token: Some("cp:invalid".to_string()),
                rationale: "bad".to_string(),
            },
        })
        .expect_err("decision should fail");

    assert_eq!(err.kind, MindErrorKind::PolicyViolation);
}
