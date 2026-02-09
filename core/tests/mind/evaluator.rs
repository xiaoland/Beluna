use beluna::mind::{
    DeterministicEvaluator, GoalManager, MindCommand, MindState, NormativeEvaluator,
    types::{EvaluationCriterion, EvaluationVerdict},
};

use super::high_goal;

#[test]
fn given_alignment_evidence_when_evaluate_then_alignment_judgment_emitted() {
    let mut state = MindState::default();
    let goal = high_goal("g1", 1);
    GoalManager::register_goal(&mut state, goal.clone()).expect("goal should register");
    GoalManager::activate_goal(&mut state, &goal.id).expect("goal should activate");

    let evaluator = DeterministicEvaluator;
    let report = evaluator
        .evaluate(&state, &MindCommand::EvaluateNow)
        .expect("evaluation should succeed");

    let alignment = report
        .judgments
        .iter()
        .find(|j| j.criterion == EvaluationCriterion::GoalAlignment)
        .expect("alignment judgment should exist");
    assert_eq!(alignment.verdict, EvaluationVerdict::Pass);
}

#[test]
fn given_missing_evidence_when_evaluate_then_unknown_verdict_emitted() {
    let state = MindState::default();
    let evaluator = DeterministicEvaluator;
    let report = evaluator
        .evaluate(&state, &MindCommand::EvaluateNow)
        .expect("evaluation should succeed");

    let faithfulness = report
        .judgments
        .iter()
        .find(|j| j.criterion == EvaluationCriterion::SignalFaithfulness)
        .expect("faithfulness judgment should exist");
    assert_eq!(faithfulness.verdict, EvaluationVerdict::Unknown);
}

#[test]
fn given_non_pass_verdict_when_evaluate_then_rationale_is_non_empty() {
    let state = MindState::default();
    let evaluator = DeterministicEvaluator;

    let report = evaluator
        .evaluate(
            &state,
            &MindCommand::ObserveSignal {
                signal_id: "s1".to_string(),
                fidelity_hint: Some(0.2),
                payload: serde_json::json!({"value": 1}),
            },
        )
        .expect("evaluation should succeed");

    for judgment in report
        .judgments
        .iter()
        .filter(|j| !matches!(j.verdict, EvaluationVerdict::Pass))
    {
        assert!(
            !judgment.rationale.trim().is_empty(),
            "non-pass judgments must have rationale"
        );
    }
}
