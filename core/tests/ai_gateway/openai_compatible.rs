use beluna::ai_gateway::{
    adapters::{BackendAdapter, openai_compatible::OpenAiCompatibleAdapter},
    error::GatewayErrorKind,
    types::{AdapterContext, BackendDialect, BackendProfile, CredentialRef, ResolvedCredential},
    types_chat::{
        CanonicalContentPart, CanonicalLimits, CanonicalMessage, CanonicalOutputMode,
        CanonicalRequest, CanonicalRole, CanonicalToolChoice,
    },
};

fn request() -> CanonicalRequest {
    CanonicalRequest {
        request_id: "req-openai".to_string(),
        route_hint: Some("b1/gpt-4.1-mini".to_string()),
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
async fn given_missing_endpoint_when_openai_compatible_invoked_then_invalid_request_is_returned() {
    let adapter = OpenAiCompatibleAdapter::default();
    assert_eq!(adapter.dialect(), BackendDialect::OpenAiCompatible);

    let ctx = AdapterContext {
        backend_id: "b1".to_string(),
        model: "gpt-4.1-mini".to_string(),
        profile: BackendProfile {
            id: "b1".to_string(),
            dialect: BackendDialect::OpenAiCompatible,
            endpoint: None,
            credential: CredentialRef::None,
            models: vec![beluna::ai_gateway::types::ModelProfile {
                id: "gpt-4.1-mini".to_string(),
            }],
            capabilities: None,
            copilot: None,
        },
        credential: ResolvedCredential::none(),
        timeout: std::time::Duration::from_secs(5),
        request_id: "req-openai".to_string(),
    };

    let err = match adapter.invoke_stream(ctx, request()).await {
        Ok(_) => panic!("missing endpoint should fail"),
        Err(err) => err,
    };
    assert_eq!(err.kind, GatewayErrorKind::InvalidRequest);
}
