use beluna::ai_gateway::{
    error::{GatewayError, GatewayErrorKind},
    reliability::ReliabilityLayer,
    types::{BackendCapabilities, ReliabilityConfig},
};

#[tokio::test]
async fn given_retryable_error_before_any_event_when_can_retry_then_retry_is_allowed() {
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
}

#[tokio::test]
async fn given_output_already_emitted_when_can_retry_then_retry_is_denied() {
    let reliability = ReliabilityLayer::new(ReliabilityConfig::default());
    let err =
        GatewayError::new(GatewayErrorKind::BackendTransient, "transient").with_retryable(true);

    let can_retry =
        reliability.can_retry(&err, 0, true, false, &BackendCapabilities::default(), false);
    assert!(!can_retry);
}

#[tokio::test]
async fn given_tool_event_and_unsafe_adapter_when_can_retry_then_retry_is_denied() {
    let reliability = ReliabilityLayer::new(ReliabilityConfig::default());
    let err =
        GatewayError::new(GatewayErrorKind::BackendTransient, "transient").with_retryable(true);

    let can_retry =
        reliability.can_retry(&err, 0, false, true, &BackendCapabilities::default(), false);
    assert!(!can_retry);
}

#[tokio::test]
async fn given_failure_threshold_reached_when_backend_allowed_checked_then_circuit_is_open() {
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

#[tokio::test]
async fn given_failure_not_counted_when_recorded_then_backend_remains_allowed() {
    let mut config = ReliabilityConfig::default();
    config.breaker_failure_threshold = 1;
    config.breaker_open_ms = 5000;
    let reliability = ReliabilityLayer::new(config);
    let backend = "b1".to_string();

    reliability.record_failure(&backend, false).await;

    reliability
        .ensure_backend_allowed(&backend)
        .await
        .expect("non-counted failures must not open the circuit");
}
