use beluna::ai_gateway::{
    adapters::{BackendAdapter, ollama::OllamaAdapter},
    error::GatewayErrorKind,
    types::{AdapterContext, BackendDialect, BackendProfile, CredentialRef, ResolvedCredential},
    types_chat::{
        CanonicalContentPart, CanonicalLimits, CanonicalMessage, CanonicalOutputMode,
        CanonicalRequest, CanonicalRole, CanonicalToolChoice,
    },
};

fn request() -> CanonicalRequest {
    CanonicalRequest {
        request_id: "req-ollama".to_string(),
        route_hint: Some("b1/qwen2.5-coder:7b".to_string()),
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

#[tokio::test]
async fn given_missing_endpoint_when_ollama_invoked_then_invalid_request_is_returned() {
    let adapter = OllamaAdapter::default();
    assert_eq!(adapter.dialect(), BackendDialect::Ollama);

    let ctx = AdapterContext {
        backend_id: "b1".to_string(),
        model: "qwen2.5-coder:7b".to_string(),
        profile: BackendProfile {
            id: "b1".to_string(),
            dialect: BackendDialect::Ollama,
            endpoint: None,
            credential: CredentialRef::None,
            models: vec![beluna::ai_gateway::types::ModelProfile {
                id: "qwen2.5-coder:7b".to_string(),
            }],
            capabilities: None,
            copilot: None,
        },
        credential: ResolvedCredential::none(),
        timeout: std::time::Duration::from_secs(5),
        request_id: "req-ollama".to_string(),
    };

    let err = match adapter.invoke_stream(ctx, request()).await {
        Ok(_) => panic!("missing endpoint should fail"),
        Err(err) => err,
    };
    assert_eq!(err.kind, GatewayErrorKind::InvalidRequest);
}
