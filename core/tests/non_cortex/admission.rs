use std::sync::Arc;

use beluna::{
    non_cortex::{
        AdmissionDisposition, AdmissionResolver, AdmissionResolverConfig, AffordanceProfile,
        AffordanceRegistry, CostAdmissionPolicy, DegradationPreference, DegradationProfile,
        IntentAttempt, NonCortexFacade, NonCortexState, RequestedResources,
        noop::{NoopDebitSource, SpinePortAdapter},
    },
    spine::{DeterministicNoopSpine, SpineExecutionMode, types::CostVector},
};

fn attempt(
    attempt_id: &str,
    affordance_key: &str,
    capability_handle: &str,
    survival_micro: i64,
) -> IntentAttempt {
    IntentAttempt {
        attempt_id: attempt_id.to_string(),
        cycle_id: 1,
        commitment_id: "c1".to_string(),
        goal_id: "g1".to_string(),
        planner_slot: 0,
        affordance_key: affordance_key.to_string(),
        capability_handle: capability_handle.to_string(),
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

#[test]
fn given_unknown_affordance_when_admitting_then_hard_denial_code_is_returned() {
    let mut facade = NonCortexFacade::with_defaults(10_000);

    let output = facade
        .process_attempts(1, vec![attempt("a1", "unknown", "cap.core", 1)])
        .expect("processing should succeed");

    assert_eq!(output.admission_report.outcomes.len(), 1);
    assert!(matches!(
        output.admission_report.outcomes[0].disposition,
        AdmissionDisposition::DeniedHard { ref code } if code == "unknown_affordance"
    ));
}

#[test]
fn given_insufficient_budget_when_admitting_then_economic_denial_code_is_returned() {
    let mut facade = NonCortexFacade::with_defaults(100);

    let output = facade
        .process_attempts(1, vec![attempt("a1", "deliberate.plan", "cap.core", 10)])
        .expect("processing should succeed");

    assert_eq!(output.admission_report.outcomes.len(), 1);
    assert!(matches!(
        output.admission_report.outcomes[0].disposition,
        AdmissionDisposition::DeniedEconomic { ref code } if code == "insufficient_survival_budget"
    ));
}

#[test]
fn given_base_cost_unaffordable_when_degradation_is_affordable_then_admitted_with_degraded_true() {
    let mut facade = NonCortexFacade::with_defaults(300);

    let output = facade
        .process_attempts(1, vec![attempt("a1", "deliberate.plan", "cap.core", 10)])
        .expect("processing should succeed");

    assert!(matches!(
        output.admission_report.outcomes[0].disposition,
        AdmissionDisposition::Admitted { degraded: true }
    ));
}

#[test]
fn given_tied_degradation_candidates_when_admitting_then_stable_tiebreaker_and_cap_are_applied() {
    let profile = AffordanceProfile {
        profile_id: "p0".to_string(),
        affordance_key: "deliberate.plan".to_string(),
        capability_handle: "cap.core".to_string(),
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
                capability_handle_override: Some("cap.core.lite".to_string()),
            },
            DegradationProfile {
                profile_id: "b".to_string(),
                depth: 1,
                capability_loss_score: 5,
                cost_multiplier_milli: 100,
                capability_handle_override: Some("cap.core.lite".to_string()),
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

    let mut facade = NonCortexFacade::new(
        NonCortexState::new(1_000),
        resolver,
        Arc::new(SpinePortAdapter::new(Arc::new(
            DeterministicNoopSpine::new(SpineExecutionMode::SerializedDeterministic),
        ))),
        Arc::new(NoopDebitSource),
    );

    let output = facade
        .process_attempts(1, vec![attempt("a1", "deliberate.plan", "cap.core", 0)])
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
