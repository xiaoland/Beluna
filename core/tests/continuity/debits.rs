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

fn attempt_with(
    attempt_id: &str,
    cost_attribution_id: &str,
    capability_instance_id: &str,
) -> IntentAttempt {
    IntentAttempt {
        attempt_id: attempt_id.to_string(),
        cycle_id: 1,
        commitment_id: "c1".to_string(),
        goal_id: "g1".to_string(),
        planner_slot: 0,
        based_on: vec!["s1".to_string()],
        endpoint_id: "core.mind".to_string(),
        capability_id: "deliberate.plan".to_string(),
        capability_instance_id: capability_instance_id.to_string(),
        normalized_payload: serde_json::json!({"k": "v"}),
        requested_resources: Default::default(),
        cost_attribution_id: cost_attribution_id.to_string(),
    }
}

fn attempt() -> IntentAttempt {
    attempt_with("att:1", "cat:ok", "instance:att:1")
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

#[tokio::test]
async fn given_unmatched_attribution_when_applying_external_debit_then_observation_is_ignored() {
    let debit_source = Arc::new(InMemoryDebitSource::default());
    let mut engine = engine_with_debit_source(Arc::clone(&debit_source));

    engine
        .process_attempts(1, vec![attempt()])
        .await
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
        .await
        .expect("second cycle should succeed");

    assert_eq!(output.external_debit_applied_count, 0);
}

#[tokio::test]
async fn given_duplicate_external_reference_when_applied_then_debit_is_deduped() {
    let debit_source = Arc::new(InMemoryDebitSource::default());
    let mut engine = engine_with_debit_source(Arc::clone(&debit_source));

    engine
        .process_attempts(1, vec![attempt()])
        .await
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
        .await
        .expect("second cycle should succeed");

    assert_eq!(output.external_debit_applied_count, 1);
}

#[tokio::test]
async fn given_duplicate_capability_instance_id_when_applying_external_debit_then_neural_signal_id_correlates()
 {
    let debit_source = Arc::new(InMemoryDebitSource::default());
    let mut engine = engine_with_debit_source(Arc::clone(&debit_source));

    let output = engine
        .process_attempts(
            1,
            vec![
                attempt_with("att:1", "cat:shared", "instance:same"),
                attempt_with("att:2", "cat:shared", "instance:same"),
            ],
        )
        .await
        .expect("first cycle should succeed");

    let admitted_neural_signal_ids: Vec<_> = output
        .admission_report
        .outcomes
        .iter()
        .filter_map(|item| item.admitted_neural_signal_id.clone())
        .collect();
    assert_eq!(admitted_neural_signal_ids.len(), 2);
    assert_ne!(admitted_neural_signal_ids[0], admitted_neural_signal_ids[1]);

    debit_source.push(ExternalDebitObservation {
        reference_id: "obs:neural:1".to_string(),
        cost_attribution_id: "cat:shared".to_string(),
        action_id: Some(admitted_neural_signal_ids[0].clone()),
        cycle_id: Some(1),
        debit_survival_micro: 11,
    });

    let followup = engine
        .process_attempts(2, Vec::new())
        .await
        .expect("second cycle should succeed");

    assert_eq!(followup.external_debit_applied_count, 1);
}
