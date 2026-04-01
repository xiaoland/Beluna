use beluna::ai_gateway::{
    router::BackendRouter,
    types::{
        AIGatewayConfig, BackendDialect, BackendProfile, ChatConfig, ChatRouteAlias, ChatRouteKey,
        ChatRouteRef, CredentialRef, ModelProfile, ResilienceConfig,
    },
};

fn gateway_config() -> AIGatewayConfig {
    AIGatewayConfig {
        backends: vec![
            BackendProfile {
                id: "primary".to_string(),
                dialect: BackendDialect::OpenAiCompatible,
                endpoint: Some("https://example.com/v1".to_string()),
                credential: CredentialRef::None,
                models: vec![ModelProfile {
                    id: "m1".to_string(),
                    aliases: vec!["default".to_string()],
                }],
                capabilities: None,
                copilot: None,
            },
            BackendProfile {
                id: "secondary".to_string(),
                dialect: BackendDialect::Ollama,
                endpoint: Some("http://localhost:11434".to_string()),
                credential: CredentialRef::None,
                models: vec![ModelProfile {
                    id: "m2".to_string(),
                    aliases: vec!["low-cost".to_string()],
                }],
                capabilities: None,
                copilot: None,
            },
        ],
        chat: ChatConfig::default(),
        resilience: ResilienceConfig::default(),
    }
}

#[test]
fn missing_route_selects_default_alias() {
    let router = BackendRouter::new(&gateway_config()).expect("router should build");
    let selected = router
        .select_route_ref(None)
        .expect("selection should succeed");
    assert_eq!(selected.backend_id, "primary");
    assert_eq!(selected.resolved_model, "m1");
}

#[test]
fn unknown_alias_fails_fast() {
    let router = BackendRouter::new(&gateway_config()).expect("router should build");
    let err = router
        .select_route_ref(Some(&ChatRouteRef::Alias(ChatRouteAlias {
            capability: "chat".to_string(),
            alias: "unknown".to_string(),
        })))
        .expect_err("selection should fail");
    assert!(err.message.contains("unknown route alias"));
}

#[test]
fn chat_route_ref_alias_selects_backend() {
    let router = BackendRouter::new(&gateway_config()).expect("router should build");
    let selected = router
        .select_route_ref(Some(&ChatRouteRef::Alias(ChatRouteAlias {
            capability: "chat".to_string(),
            alias: "default".to_string(),
        })))
        .expect("selection should succeed");
    assert_eq!(selected.backend_id, "primary");
    assert_eq!(selected.resolved_model, "m1");
}

#[test]
fn chat_route_ref_key_selects_backend() {
    let router = BackendRouter::new(&gateway_config()).expect("router should build");
    let selected = router
        .select_route_ref(Some(&ChatRouteRef::Key(ChatRouteKey {
            capability: "chat".to_string(),
            binding_id: "low-cost".to_string(),
        })))
        .expect("selection should succeed");
    assert_eq!(selected.backend_id, "secondary");
    assert_eq!(selected.resolved_model, "m2");
}

#[test]
fn non_chat_capability_route_ref_is_rejected() {
    let router = BackendRouter::new(&gateway_config()).expect("router should build");
    let err = router
        .select_route_ref(Some(&ChatRouteRef::Alias(ChatRouteAlias {
            capability: "asr".to_string(),
            alias: "default".to_string(),
        })))
        .expect_err("selection should fail");
    assert!(err.message.contains("unsupported capability"));
}
