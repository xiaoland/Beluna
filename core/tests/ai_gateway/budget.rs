use std::time::Duration;

use tokio::time::timeout;

use beluna::ai_gateway::{
    budget::BudgetEnforcer,
    error::GatewayErrorKind,
    types::{
        BudgetConfig, CanonicalContentPart, CanonicalLimits, CanonicalMessage, CanonicalOutputMode,
        CanonicalRequest, CanonicalRole, CanonicalToolChoice,
    },
};

fn request(max_output_tokens: Option<u64>) -> CanonicalRequest {
    CanonicalRequest {
        request_id: "req-budget".to_string(),
        backend_hint: Some("b1".to_string()),
        model_override: None,
        messages: vec![CanonicalMessage {
            role: CanonicalRole::User,
            parts: vec![CanonicalContentPart::Text {
                text: "hello".to_string(),
            }],
            tool_call_id: None,
            tool_name: None,
        }],
        tools: vec![],
        tool_choice: CanonicalToolChoice::Auto,
        output_mode: CanonicalOutputMode::Text,
        limits: CanonicalLimits {
            max_output_tokens,
            max_request_time_ms: None,
        },
        metadata: Default::default(),
        stream: true,
    }
}

#[tokio::test]
async fn given_output_token_limit_exceeded_when_pre_dispatch_then_budget_exceeded_is_returned() {
    let enforcer = BudgetEnforcer::new(BudgetConfig {
        max_request_time_ms: 45_000,
        max_usage_tokens_per_request: Some(10),
        max_concurrency_per_backend: 1,
        rate_smoothing_per_second: None,
    });

    let err = enforcer
        .pre_dispatch(&request(Some(20)), &"b1".to_string())
        .await
        .expect_err("should exceed budget");
    assert_eq!(err.kind, GatewayErrorKind::BudgetExceeded);
}

#[tokio::test]
async fn given_concurrency_limit_reached_when_pre_dispatch_then_second_request_blocks() {
    let enforcer = BudgetEnforcer::new(BudgetConfig {
        max_request_time_ms: 45_000,
        max_usage_tokens_per_request: None,
        max_concurrency_per_backend: 1,
        rate_smoothing_per_second: None,
    });

    let lease = enforcer
        .pre_dispatch(&request(None), &"b1".to_string())
        .await
        .expect("first permit");

    let second = timeout(
        Duration::from_millis(40),
        enforcer.pre_dispatch(&request(None), &"b1".to_string()),
    )
    .await;
    assert!(second.is_err(), "second request should block on semaphore");

    enforcer.release(lease);
}
