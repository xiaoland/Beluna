use crate::ai_gateway::{
    adapters::{BackendAdapter, openai_compatible::OpenAiCompatibleAdapter},
    error::GatewayErrorKind,
    types::{
        AdapterContext, BackendDialect, BackendProfile, CanonicalContentPart, CanonicalLimits,
        CanonicalMessage, CanonicalOutputMode, CanonicalRequest, CanonicalRole,
        CanonicalToolChoice, CredentialRef, ResolvedCredential,
    },
};

fn request() -> CanonicalRequest {
    CanonicalRequest {
        request_id: "req-openai".to_string(),
        backend_hint: Some("b1".to_string()),
        model_override: Some("gpt-4.1-mini".to_string()),
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
        stream: true,
    }
}

#[tokio::test]
async fn requires_endpoint() {
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
            default_model: "gpt-4.1-mini".to_string(),
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
