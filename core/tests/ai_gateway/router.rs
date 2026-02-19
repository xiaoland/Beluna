use std::collections::BTreeMap;

use beluna::ai_gateway::{
    router::BackendRouter,
    types::{
        AIGatewayConfig, BackendDialect, BackendProfile, BudgetConfig, CredentialRef, ModelProfile,
        ModelTarget, ReliabilityConfig,
    },
    types_chat::{
        CanonicalContentPart, CanonicalLimits, CanonicalMessage, CanonicalOutputMode,
        CanonicalRequest, CanonicalRole, CanonicalToolChoice,
    },
};

fn gateway_config() -> AIGatewayConfig {
    let mut route_aliases = BTreeMap::new();
    route_aliases.insert(
        "default".to_string(),
        ModelTarget {
            backend_id: "primary".to_string(),
            model_id: "m1".to_string(),
        },
    );
    route_aliases.insert(
        "low-cost".to_string(),
        ModelTarget {
            backend_id: "secondary".to_string(),
            model_id: "m2".to_string(),
        },
    );

    AIGatewayConfig {
        backends: vec![
            BackendProfile {
                id: "primary".to_string(),
                dialect: BackendDialect::OpenAiCompatible,
                endpoint: Some("https://example.com/v1".to_string()),
                credential: CredentialRef::None,
                models: vec![ModelProfile {
                    id: "m1".to_string(),
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
                }],
                capabilities: None,
                copilot: None,
            },
        ],
        route_aliases,
        reliability: ReliabilityConfig::default(),
        budget: BudgetConfig::default(),
    }
}

fn request_with_route(route: Option<&str>) -> CanonicalRequest {
    CanonicalRequest {
        request_id: "req".to_string(),
        route_hint: route.map(str::to_string),
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
fn given_route_is_missing_when_select_then_default_alias_is_chosen() {
    let router = BackendRouter::new(&gateway_config()).expect("router should build");
    let selected = router
        .select(&request_with_route(None))
        .expect("selection should succeed");
    assert_eq!(selected.backend_id, "primary");
    assert_eq!(selected.resolved_model, "m1");
}

#[test]
fn given_known_alias_when_select_then_requested_alias_target_is_chosen() {
    let router = BackendRouter::new(&gateway_config()).expect("router should build");
    let selected = router
        .select(&request_with_route(Some("low-cost")))
        .expect("selection should succeed");
    assert_eq!(selected.backend_id, "secondary");
    assert_eq!(selected.resolved_model, "m2");
}

#[test]
fn given_direct_backend_model_route_when_select_then_direct_target_is_chosen() {
    let router = BackendRouter::new(&gateway_config()).expect("router should build");
    let selected = router
        .select(&request_with_route(Some("secondary/m2")))
        .expect("selection should succeed");
    assert_eq!(selected.backend_id, "secondary");
    assert_eq!(selected.resolved_model, "m2");
}

#[test]
fn given_unknown_alias_when_select_then_selection_fails() {
    let router = BackendRouter::new(&gateway_config()).expect("router should build");
    let err = match router.select(&request_with_route(Some("unknown"))) {
        Ok(_) => panic!("selection should fail"),
        Err(err) => err,
    };
    assert!(err.message.contains("unknown route alias"));
}

#[test]
fn given_unknown_model_on_known_backend_when_select_then_selection_fails() {
    let router = BackendRouter::new(&gateway_config()).expect("router should build");
    let err = match router.select(&request_with_route(Some("secondary/missing"))) {
        Ok(_) => panic!("selection should fail"),
        Err(err) => err,
    };
    assert!(err.message.contains("selected model"));
}
