use std::sync::Arc;

use beluna::{
    admission::{
        AdmissionResolver, AdmissionResolverConfig, AffordanceProfile, AffordanceRegistry,
        CostAdmissionPolicy, IntentAttempt,
    },
    continuity::{
        ContinuityEngine, ContinuityState, ExternalDebitObservation, InMemoryDebitSource,
        SpinePortAdapter,
    },
    spine::DeterministicNoopSpine,
};

fn attempt() -> IntentAttempt {
    IntentAttempt {
        attempt_id: "att:1".to_string(),
        cycle_id: 1,
        commitment_id: "c1".to_string(),
        goal_id: "g1".to_string(),
        planner_slot: 0,
        affordance_key: "deliberate.plan".to_string(),
        capability_handle: "cap.core".to_string(),
        normalized_payload: serde_json::json!({"k": "v"}),
        requested_resources: Default::default(),
        cost_attribution_id: "cat:ok".to_string(),
    }
}

fn engine_with_debit_source(debit_source: Arc<InMemoryDebitSource>) -> ContinuityEngine {
    let resolver = AdmissionResolver::new(
        AffordanceRegistry::new(vec![AffordanceProfile::default()]),
        CostAdmissionPolicy::default(),
        AdmissionResolverConfig::default(),
    );

    ContinuityEngine::new(
        ContinuityState::new(10_000),
        resolver,
        Arc::new(SpinePortAdapter::new(Arc::new(
            DeterministicNoopSpine::default(),
        ))),
        debit_source,
    )
}

#[test]
fn given_unmatched_attribution_when_applying_external_debit_then_observation_is_ignored() {
    let debit_source = Arc::new(InMemoryDebitSource::default());
    let mut engine = engine_with_debit_source(Arc::clone(&debit_source));

    engine
        .process_attempts(1, vec![attempt()])
        .expect("first cycle should succeed");

    debit_source.push(ExternalDebitObservation {
        reference_id: "obs:1".to_string(),
        cost_attribution_id: "cat:other".to_string(),
        action_id: None,
        cycle_id: Some(1),
        debit_survival_micro: 99,
    });

    let output = engine
        .process_attempts(2, Vec::new())
        .expect("second cycle should succeed");

    assert_eq!(output.external_debit_applied_count, 0);
}

#[test]
fn given_duplicate_external_reference_when_applied_then_debit_is_deduped() {
    let debit_source = Arc::new(InMemoryDebitSource::default());
    let mut engine = engine_with_debit_source(Arc::clone(&debit_source));

    engine
        .process_attempts(1, vec![attempt()])
        .expect("first cycle should succeed");

    debit_source.push(ExternalDebitObservation {
        reference_id: "obs:1".to_string(),
        cost_attribution_id: "cat:ok".to_string(),
        action_id: None,
        cycle_id: Some(1),
        debit_survival_micro: 42,
    });
    debit_source.push(ExternalDebitObservation {
        reference_id: "obs:1".to_string(),
        cost_attribution_id: "cat:ok".to_string(),
        action_id: None,
        cycle_id: Some(1),
        debit_survival_micro: 42,
    });

    let output = engine
        .process_attempts(2, Vec::new())
        .expect("second cycle should succeed");

    assert_eq!(output.external_debit_applied_count, 1);
}
