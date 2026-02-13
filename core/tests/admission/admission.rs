use std::sync::Arc;

use beluna::{
    admission::{
        AdmissionDisposition, AdmissionResolver, AdmissionResolverConfig, AffordanceProfile,
        AffordanceRegistry, CostAdmissionPolicy, DegradationPreference, DegradationProfile,
        IntentAttempt, RequestedResources,
    },
    continuity::{ContinuityEngine, ContinuityState, NoopDebitSource, SpinePortAdapter},
    spine::{DeterministicNoopSpine, SpineExecutionMode, types::CostVector},
};
use uuid::Uuid;

fn attempt(
    attempt_id: &str,
    endpoint_id: &str,
    capability_id: &str,
    survival_micro: i64,
) -> IntentAttempt {
    IntentAttempt {
        attempt_id: attempt_id.to_string(),
        cycle_id: 1,
        commitment_id: "c1".to_string(),
        goal_id: "g1".to_string(),
        planner_slot: 0,
        based_on: vec!["s1".to_string()],
        endpoint_id: endpoint_id.to_string(),
        capability_id: capability_id.to_string(),
        capability_instance_id: format!("instance:{attempt_id}"),
        normalized_payload: serde_json::json!({"x": 1}),
        requested_resources: RequestedResources {
            survival_micro,
            time_ms: 0,
            io_units: 0,
            token_units: 0,
        },
        cost_attribution_id: format!("cat:{}", attempt_id),
    }
}

#[tokio::test]
async fn given_unknown_affordance_when_admitting_then_hard_denial_code_is_returned() {
    let mut engine = ContinuityEngine::with_defaults(10_000);

    let output = engine
        .process_attempts(1, vec![attempt("a1", "unknown", "cap.core", 1)])
        .await
        .expect("processing should succeed");

    assert_eq!(output.admission_report.outcomes.len(), 1);
    assert!(matches!(
        output.admission_report.outcomes[0].disposition,
        AdmissionDisposition::DeniedHard { ref code } if code == "unknown_endpoint_id"
    ));
}

#[tokio::test]
async fn given_insufficient_budget_when_admitting_then_economic_denial_code_is_returned() {
    let mut engine = ContinuityEngine::with_defaults(100);

    let output = engine
        .process_attempts(1, vec![attempt("a1", "core.mind", "deliberate.plan", 10)])
        .await
        .expect("processing should succeed");

    assert_eq!(output.admission_report.outcomes.len(), 1);
    assert!(matches!(
        output.admission_report.outcomes[0].disposition,
        AdmissionDisposition::DeniedEconomic { ref code } if code == "insufficient_survival_budget"
    ));
}

#[tokio::test]
async fn given_base_cost_unaffordable_when_degradation_is_affordable_then_admitted_with_degraded_true()
 {
    let mut engine = ContinuityEngine::with_defaults(300);

    let output = engine
        .process_attempts(1, vec![attempt("a1", "core.mind", "deliberate.plan", 10)])
        .await
        .expect("processing should succeed");

    assert!(matches!(
        output.admission_report.outcomes[0].disposition,
        AdmissionDisposition::Admitted { degraded: true }
    ));
}

#[tokio::test]
async fn given_tied_degradation_candidates_when_admitting_then_stable_tiebreaker_and_cap_are_applied()
 {
    let profile = AffordanceProfile {
        profile_id: "p0".to_string(),
        endpoint_id: "deliberate.plan".to_string(),
        capability_id: "cap.core".to_string(),
        max_payload_bytes: 16_384,
        base_cost: CostVector {
            survival_micro: 2_000,
            time_ms: 10,
            io_units: 1,
            token_units: 1,
        },
        degradation_profiles: vec![
            DegradationProfile {
                profile_id: "a".to_string(),
                depth: 1,
                capability_loss_score: 5,
                cost_multiplier_milli: 100,
                capability_id_override: Some("cap.core.lite".to_string()),
            },
            DegradationProfile {
                profile_id: "b".to_string(),
                depth: 1,
                capability_loss_score: 5,
                cost_multiplier_milli: 100,
                capability_id_override: Some("cap.core.lite".to_string()),
            },
        ],
    };

    let resolver = AdmissionResolver::new(
        AffordanceRegistry::new(vec![profile]),
        CostAdmissionPolicy::default(),
        AdmissionResolverConfig {
            reservation_ttl_cycles: 4,
            max_degradation_variants: 1,
            max_degradation_depth: 2,
            degradation_preference: DegradationPreference::CheapestFirst,
        },
    );

    let mut engine = ContinuityEngine::new(
        ContinuityState::new(1_000),
        resolver,
        Arc::new(SpinePortAdapter::new(Arc::new(
            DeterministicNoopSpine::new(SpineExecutionMode::SerializedDeterministic),
        ))),
        Arc::new(NoopDebitSource),
    );

    let output = engine
        .process_attempts(1, vec![attempt("a1", "deliberate.plan", "cap.core", 0)])
        .await
        .expect("processing should succeed");

    assert!(matches!(
        output.admission_report.outcomes[0].disposition,
        AdmissionDisposition::Admitted { degraded: true }
    ));
    assert_eq!(
        output.admission_report.outcomes[0]
            .degradation_profile_id
            .as_deref(),
        Some("a")
    );
}

#[tokio::test]
async fn given_admitted_attempt_when_processing_then_neural_signal_id_is_uuid_v7() {
    let mut engine = ContinuityEngine::with_defaults(10_000);

    let output = engine
        .process_attempts(1, vec![attempt("a1", "core.mind", "deliberate.plan", 1)])
        .await
        .expect("processing should succeed");

    let admitted_ids: Vec<_> = output
        .admission_report
        .outcomes
        .iter()
        .filter_map(|item| item.admitted_neural_signal_id.clone())
        .collect();
    assert_eq!(admitted_ids.len(), 1);

    let parsed = Uuid::parse_str(&admitted_ids[0]).expect("neural_signal_id should be UUID");
    assert_eq!(parsed.get_version_num(), 7);
}
