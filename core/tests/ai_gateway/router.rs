use beluna::ai_gateway::{
    router::BackendRouter,
    types::{
        AIGatewayConfig, BackendDialect, BackendProfile, BudgetConfig, CanonicalContentPart,
        CanonicalLimits, CanonicalMessage, CanonicalOutputMode, CanonicalRequest, CanonicalRole,
        CanonicalToolChoice, CredentialRef, ReliabilityConfig,
    },
};

fn gateway_config() -> AIGatewayConfig {
    AIGatewayConfig {
        default_backend: "primary".to_string(),
        backends: vec![
            BackendProfile {
                id: "primary".to_string(),
                dialect: BackendDialect::OpenAiCompatible,
                endpoint: Some("https://example.com/v1".to_string()),
                credential: CredentialRef::None,
                default_model: "m1".to_string(),
                capabilities: None,
                copilot: None,
            },
            BackendProfile {
                id: "secondary".to_string(),
                dialect: BackendDialect::Ollama,
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

fn request_with_backend(backend_id: Option<&str>) -> CanonicalRequest {
    CanonicalRequest {
        request_id: "req".to_string(),
        backend_hint: backend_id.map(str::to_string),
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
        limits: CanonicalLimits::default(),
        metadata: Default::default(),
        cost_attribution_id: None,
        stream: true,
    }
}

#[test]
fn given_backend_hint_is_missing_when_select_then_default_backend_is_chosen() {
    let router = BackendRouter::new(&gateway_config()).expect("router should build");
    let selected = router
        .select(&request_with_backend(None))
        .expect("selection should succeed");
    assert_eq!(selected.backend_id, "primary");
}

#[test]
fn given_known_backend_hint_when_select_then_requested_backend_is_chosen() {
    let router = BackendRouter::new(&gateway_config()).expect("router should build");
    let selected = router
        .select(&request_with_backend(Some("secondary")))
        .expect("selection should succeed");
    assert_eq!(selected.backend_id, "secondary");
}

#[test]
fn given_unknown_backend_hint_when_select_then_selection_fails_without_fallback() {
    let router = BackendRouter::new(&gateway_config()).expect("router should build");
    let err = match router.select(&request_with_backend(Some("unknown"))) {
        Ok(_) => panic!("selection should fail"),
        Err(err) => err,
    };
    assert!(err.message.contains("no fallback"));
}
