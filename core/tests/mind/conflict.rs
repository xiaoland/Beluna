use beluna::mind::{
    ConflictCase, ConflictResolution, ConflictResolver, DeterministicConflictResolver,
    EvaluationCriterion, EvaluationVerdict, Judgment, types::DelegationResult,
};

#[test]
fn given_helper_conflict_when_resolve_then_highest_confidence_selected() {
    let resolver = DeterministicConflictResolver;
    let cases = vec![ConflictCase::HelperOutputSameIntent {
        intent_id: "i1".to_string(),
        candidates: vec![
            DelegationResult {
                intent_id: "i1".to_string(),
                helper_id: "h1".to_string(),
                payload: serde_json::json!({}),
                confidence: 0.4,
            },
            DelegationResult {
                intent_id: "i1".to_string(),
                helper_id: "h2".to_string(),
                payload: serde_json::json!({}),
                confidence: 0.9,
            },
        ],
    }];

    let resolved = resolver.resolve(&cases).expect("resolution should succeed");
    assert_eq!(
        resolved[0],
        ConflictResolution::SelectedHelperResult {
            intent_id: "i1".to_string(),
            helper_id: "h2".to_string(),
        }
    );
}

#[test]
fn given_helper_confidence_tie_when_resolve_then_lexical_helper_id_selected() {
    let resolver = DeterministicConflictResolver;
    let cases = vec![ConflictCase::HelperOutputSameIntent {
        intent_id: "i1".to_string(),
        candidates: vec![
            DelegationResult {
                intent_id: "i1".to_string(),
                helper_id: "b-helper".to_string(),
                payload: serde_json::json!({}),
                confidence: 0.8,
            },
            DelegationResult {
                intent_id: "i1".to_string(),
                helper_id: "a-helper".to_string(),
                payload: serde_json::json!({}),
                confidence: 0.8,
            },
        ],
    }];

    let resolved = resolver.resolve(&cases).expect("resolution should succeed");
    assert_eq!(
        resolved[0],
        ConflictResolution::SelectedHelperResult {
            intent_id: "i1".to_string(),
            helper_id: "a-helper".to_string(),
        }
    );
}

#[test]
fn given_evaluator_conflict_when_resolve_then_most_conservative_verdict_selected() {
    let resolver = DeterministicConflictResolver;
    let cases = vec![ConflictCase::EvaluatorVerdictSameCriterion {
        criterion: EvaluationCriterion::SubsystemReliability,
        candidates: vec![
            Judgment {
                criterion: EvaluationCriterion::SubsystemReliability,
                verdict: EvaluationVerdict::Pass,
                confidence: 0.9,
                rationale: "ok".to_string(),
                evidence_refs: vec![],
            },
            Judgment {
                criterion: EvaluationCriterion::SubsystemReliability,
                verdict: EvaluationVerdict::Fail,
                confidence: 0.7,
                rationale: "bad".to_string(),
                evidence_refs: vec![],
            },
        ],
    }];

    let resolved = resolver.resolve(&cases).expect("resolution should succeed");
    assert_eq!(
        resolved[0],
        ConflictResolution::SelectedJudgment {
            criterion: EvaluationCriterion::SubsystemReliability,
            verdict: EvaluationVerdict::Fail,
        }
    );
}

#[test]
fn given_merge_conflict_when_resolve_then_deterministic_allow_or_reject() {
    let resolver = DeterministicConflictResolver;

    let allow = resolver
        .resolve(&[ConflictCase::MergeCompatibility {
            active_goal_id: "g1".to_string(),
            incoming_goal_id: "g2".to_string(),
            compatible: true,
        }])
        .expect("resolution should succeed");
    assert!(matches!(allow[0], ConflictResolution::MergeAllowed { .. }));

    let reject = resolver
        .resolve(&[ConflictCase::MergeCompatibility {
            active_goal_id: "g1".to_string(),
            incoming_goal_id: "g2".to_string(),
            compatible: false,
        }])
        .expect("resolution should succeed");
    assert_eq!(reject[0], ConflictResolution::MergeRejected);
}
