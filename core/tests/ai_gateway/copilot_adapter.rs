use beluna::ai_gateway::{
    adapters::{BackendAdapter, github_copilot::GitHubCopilotAdapter},
    error::GatewayErrorKind,
    types::{
        AdapterContext, BackendDialect, BackendProfile, CanonicalContentPart, CanonicalLimits,
        CanonicalMessage, CanonicalOutputMode, CanonicalRequest, CanonicalRole,
        CanonicalToolChoice, CredentialRef, ResolvedCredential,
    },
};

fn request() -> CanonicalRequest {
    CanonicalRequest {
        request_id: "req-copilot".to_string(),
        backend_hint: Some("copilot".to_string()),
        model_override: Some("copilot-default".to_string()),
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
async fn given_missing_copilot_config_when_invoked_then_first_stream_item_is_invalid_request() {
    let adapter = GitHubCopilotAdapter;
    assert_eq!(adapter.dialect(), BackendDialect::GitHubCopilotSdk);

    let ctx = AdapterContext {
        backend_id: "copilot".to_string(),
        model: "copilot-default".to_string(),
        profile: BackendProfile {
            id: "copilot".to_string(),
            dialect: BackendDialect::GitHubCopilotSdk,
            endpoint: None,
            credential: CredentialRef::None,
            default_model: "copilot-default".to_string(),
            capabilities: None,
            copilot: None,
        },
        credential: ResolvedCredential::none(),
        timeout: std::time::Duration::from_secs(5),
        request_id: "req-copilot".to_string(),
    };

    let invocation = adapter
        .invoke_stream(ctx, request())
        .await
        .expect("adapter should build invocation");

    let mut stream = invocation.stream;
    let first = futures_util::StreamExt::next(&mut stream)
        .await
        .expect("first item should exist")
        .expect_err("first item should be error");
    assert_eq!(first.kind, GatewayErrorKind::InvalidRequest);
}
