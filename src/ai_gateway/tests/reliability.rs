use crate::ai_gateway::{
    error::{GatewayError, GatewayErrorKind},
    reliability::ReliabilityLayer,
    types::{BackendCapabilities, ReliabilityConfig},
};

#[tokio::test]
async fn retries_before_first_output_only() {
    let reliability = ReliabilityLayer::new(ReliabilityConfig::default());
    let err =
        GatewayError::new(GatewayErrorKind::BackendTransient, "transient").with_retryable(true);

    let can_retry_before_output = reliability.can_retry(
        &err,
        0,
        false,
        false,
        &BackendCapabilities::default(),
        false,
    );
    assert!(can_retry_before_output);

    let can_retry_after_output =
        reliability.can_retry(&err, 0, true, false, &BackendCapabilities::default(), false);
    assert!(!can_retry_after_output);
}

#[tokio::test]
async fn does_not_retry_after_tool_event_when_not_safe() {
    let reliability = ReliabilityLayer::new(ReliabilityConfig::default());
    let err =
        GatewayError::new(GatewayErrorKind::BackendTransient, "transient").with_retryable(true);

    let can_retry =
        reliability.can_retry(&err, 0, false, true, &BackendCapabilities::default(), false);
    assert!(!can_retry);
}

#[tokio::test]
async fn opens_circuit_after_threshold_failures() {
    let mut config = ReliabilityConfig::default();
    config.breaker_failure_threshold = 2;
    config.breaker_open_ms = 5000;
    let reliability = ReliabilityLayer::new(config);
    let backend = "b1".to_string();

    reliability.record_failure(&backend, true).await;
    reliability.record_failure(&backend, true).await;

    let err = reliability
        .ensure_backend_allowed(&backend)
        .await
        .expect_err("circuit should be open");
    assert_eq!(err.kind, GatewayErrorKind::CircuitOpen);
}
