use std::time::Duration;

use tokio::time::timeout;

use crate::ai_gateway::{
    budget::BudgetEnforcer,
    types::{
        BudgetConfig, CanonicalLimits, CanonicalMessage, CanonicalOutputMode, CanonicalRequest,
        CanonicalRole, CanonicalToolChoice,
    },
};

fn request(max_output_tokens: Option<u64>) -> CanonicalRequest {
    CanonicalRequest {
        request_id: "req-budget".to_string(),
        backend_hint: Some("b1".to_string()),
        model_override: None,
        messages: vec![CanonicalMessage {
            role: CanonicalRole::User,
            parts: vec![crate::ai_gateway::types::CanonicalContentPart::Text {
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
async fn enforces_precheck_max_output_tokens() {
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
    assert_eq!(
        err.kind,
        crate::ai_gateway::error::GatewayErrorKind::BudgetExceeded
    );
}

#[tokio::test]
async fn enforces_per_backend_concurrency() {
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
