use beluna::mind::{
    DeterministicEvolutionDecider, EvolutionDecider, EvolutionDecision, EvolutionTarget,
    GoalManager, MindState,
    types::{EvaluationCriterion, EvaluationReport, EvaluationVerdict, Judgment},
};

use super::high_goal;

fn report(
    goal_id: &str,
    criterion: EvaluationCriterion,
    verdict: EvaluationVerdict,
    confidence: f32,
) -> EvaluationReport {
    EvaluationReport {
        goal_id: Some(goal_id.to_string()),
        judgments: vec![Judgment {
            criterion,
            verdict,
            confidence,
            rationale: "evidence".to_string(),
            evidence_refs: vec!["ref1".to_string()],
        }],
    }
}

#[test]
fn given_single_failure_when_decide_then_no_change() {
    let mut state = MindState::default();
    let goal = high_goal("g1", 1);
    GoalManager::register_goal(&mut state, goal.clone()).expect("goal should register");
    GoalManager::activate_goal(&mut state, &goal.id).expect("goal should activate");

    state.push_evaluation(report(
        &goal.id,
        EvaluationCriterion::SubsystemReliability,
        EvaluationVerdict::Fail,
        0.8,
    ));

    let decider = DeterministicEvolutionDecider::default();
    let decision = decider
        .decide(
            &state,
            &report(
                &goal.id,
                EvaluationCriterion::SubsystemReliability,
                EvaluationVerdict::Fail,
                0.8,
            ),
        )
        .expect("decision should succeed");

    assert!(matches!(decision, EvolutionDecision::NoChange { .. }));
}

#[test]
fn given_repeated_reliability_failures_when_decide_then_change_proposal() {
    let mut state = MindState::default();
    let goal = high_goal("g1", 1);
    GoalManager::register_goal(&mut state, goal.clone()).expect("goal should register");
    GoalManager::activate_goal(&mut state, &goal.id).expect("goal should activate");

    state.push_evaluation(report(
        &goal.id,
        EvaluationCriterion::SubsystemReliability,
        EvaluationVerdict::Fail,
        0.9,
    ));
    state.push_evaluation(report(
        &goal.id,
        EvaluationCriterion::SubsystemReliability,
        EvaluationVerdict::Fail,
        0.9,
    ));

    let decider = DeterministicEvolutionDecider::default();
    let decision = decider
        .decide(
            &state,
            &report(
                &goal.id,
                EvaluationCriterion::SubsystemReliability,
                EvaluationVerdict::Fail,
                0.9,
            ),
        )
        .expect("decision should succeed");

    assert!(matches!(decision, EvolutionDecision::ChangeProposal(_)));
}

#[test]
fn given_faithfulness_failures_when_decide_then_perception_target_proposed() {
    let mut state = MindState::default();
    let goal = high_goal("g1", 1);
    GoalManager::register_goal(&mut state, goal.clone()).expect("goal should register");
    GoalManager::activate_goal(&mut state, &goal.id).expect("goal should activate");

    state.push_evaluation(report(
        &goal.id,
        EvaluationCriterion::SignalFaithfulness,
        EvaluationVerdict::Fail,
        0.9,
    ));
    state.push_evaluation(report(
        &goal.id,
        EvaluationCriterion::SignalFaithfulness,
        EvaluationVerdict::Fail,
        0.9,
    ));

    let decider = DeterministicEvolutionDecider::default();
    let decision = decider
        .decide(
            &state,
            &report(
                &goal.id,
                EvaluationCriterion::SignalFaithfulness,
                EvaluationVerdict::Fail,
                0.9,
            ),
        )
        .expect("decision should succeed");

    match decision {
        EvolutionDecision::ChangeProposal(proposal) => {
            assert!(matches!(
                proposal.target,
                EvolutionTarget::PerceptionPipeline { .. }
            ));
        }
        _ => panic!("expected change proposal"),
    }
}

#[test]
fn given_low_confidence_when_decide_then_no_change() {
    let mut state = MindState::default();
    let goal = high_goal("g1", 1);
    GoalManager::register_goal(&mut state, goal.clone()).expect("goal should register");
    GoalManager::activate_goal(&mut state, &goal.id).expect("goal should activate");

    state.push_evaluation(report(
        &goal.id,
        EvaluationCriterion::SubsystemReliability,
        EvaluationVerdict::Fail,
        0.2,
    ));
    state.push_evaluation(report(
        &goal.id,
        EvaluationCriterion::SubsystemReliability,
        EvaluationVerdict::Fail,
        0.2,
    ));

    let decider = DeterministicEvolutionDecider::default();
    let decision = decider
        .decide(
            &state,
            &report(
                &goal.id,
                EvaluationCriterion::SubsystemReliability,
                EvaluationVerdict::Fail,
                0.2,
            ),
        )
        .expect("decision should succeed");

    assert!(matches!(decision, EvolutionDecision::NoChange { .. }));
}
