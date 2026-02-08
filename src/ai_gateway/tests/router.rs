use crate::ai_gateway::{
    router::BackendRouter,
    types::{AIGatewayConfig, BackendProfile, BudgetConfig, CredentialRef, ReliabilityConfig},
};

fn gateway_config() -> AIGatewayConfig {
    AIGatewayConfig {
        default_backend: "primary".to_string(),
        backends: vec![
            BackendProfile {
                id: "primary".to_string(),
                dialect: crate::ai_gateway::types::BackendDialect::OpenAiCompatible,
                endpoint: Some("https://example.com/v1".to_string()),
                credential: CredentialRef::None,
                default_model: "m1".to_string(),
                capabilities: None,
                copilot: None,
            },
            BackendProfile {
                id: "secondary".to_string(),
                dialect: crate::ai_gateway::types::BackendDialect::Ollama,
                endpoint: Some("http://localhost:11434".to_string()),
                credential: CredentialRef::None,
                default_model: "m2".to_string(),
                capabilities: None,
                copilot: None,
            },
        ],
        reliability: ReliabilityConfig::default(),
        budget: BudgetConfig::default(),
    }
}

fn request_with_backend(backend_id: Option<&str>) -> crate::ai_gateway::types::CanonicalRequest {
    crate::ai_gateway::types::CanonicalRequest {
        request_id: "req".to_string(),
        backend_hint: backend_id.map(str::to_string),
        model_override: None,
        messages: vec![crate::ai_gateway::types::CanonicalMessage {
            role: crate::ai_gateway::types::CanonicalRole::User,
            parts: vec![crate::ai_gateway::types::CanonicalContentPart::Text {
                text: "hello".to_string(),
            }],
            tool_call_id: None,
            tool_name: None,
        }],
        tools: vec![],
        tool_choice: crate::ai_gateway::types::CanonicalToolChoice::Auto,
        output_mode: crate::ai_gateway::types::CanonicalOutputMode::Text,
        limits: crate::ai_gateway::types::CanonicalLimits::default(),
        metadata: Default::default(),
        stream: true,
    }
}

#[test]
fn selects_default_backend_deterministically() {
    let router = BackendRouter::new(&gateway_config()).expect("router should build");
    let selected = router
        .select(&request_with_backend(None))
        .expect("selection should succeed");
    assert_eq!(selected.backend_id, "primary");
}

#[test]
fn selects_requested_backend_without_fallback() {
    let router = BackendRouter::new(&gateway_config()).expect("router should build");
    let selected = router
        .select(&request_with_backend(Some("secondary")))
        .expect("selection should succeed");
    assert_eq!(selected.backend_id, "secondary");
}

#[test]
fn rejects_unknown_backend_without_fallback() {
    let router = BackendRouter::new(&gateway_config()).expect("router should build");
    let err = match router.select(&request_with_backend(Some("unknown"))) {
        Ok(_) => panic!("selection should fail"),
        Err(err) => err,
    };
    assert!(err.message.contains("no fallback"));
}
