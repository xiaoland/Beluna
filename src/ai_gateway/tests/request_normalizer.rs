use std::collections::BTreeMap;

use crate::ai_gateway::{
    request_normalizer::RequestNormalizer,
    types::{
        BelunaContentPart, BelunaInferenceRequest, BelunaMessage, BelunaRole, BelunaToolDefinition,
        OutputMode, RequestLimitOverrides, ToolChoice,
    },
};

fn base_request() -> BelunaInferenceRequest {
    BelunaInferenceRequest {
        request_id: None,
        backend_id: Some("openai-default".to_string()),
        model: Some("gpt-4.1-mini".to_string()),
        messages: vec![BelunaMessage {
            role: BelunaRole::User,
            parts: vec![BelunaContentPart::Text {
                text: "hello".to_string(),
            }],
            tool_call_id: None,
            tool_name: None,
        }],
        tools: vec![],
        tool_choice: ToolChoice::Auto,
        output_mode: OutputMode::Text,
        limits: RequestLimitOverrides::default(),
        metadata: BTreeMap::new(),
        stream: true,
    }
}

#[test]
fn generates_request_id_when_missing() {
    let normalizer = RequestNormalizer;
    let request = base_request();
    let normalized = normalizer
        .normalize(request)
        .expect("normalization should succeed");
    assert!(!normalized.request_id.is_empty());
}

#[test]
fn rejects_tool_message_missing_tool_call_id() {
    let normalizer = RequestNormalizer;
    let mut request = base_request();
    request.messages = vec![BelunaMessage {
        role: BelunaRole::Tool,
        parts: vec![BelunaContentPart::Text {
            text: "result".to_string(),
        }],
        tool_call_id: None,
        tool_name: Some("my_tool".to_string()),
    }];

    let err = normalizer
        .normalize(request)
        .expect_err("normalization should fail");
    assert!(err.message.contains("tool_call_id"));
}

#[test]
fn rejects_non_tool_message_with_tool_linkage() {
    let normalizer = RequestNormalizer;
    let mut request = base_request();
    request.messages[0].tool_call_id = Some("abc".to_string());

    let err = normalizer
        .normalize(request)
        .expect_err("normalization should fail");
    assert!(err.message.contains("non-tool"));
}

#[test]
fn rejects_tool_schema_unknown_keyword() {
    let normalizer = RequestNormalizer;
    let mut request = base_request();
    request.tools = vec![BelunaToolDefinition {
        name: "test_tool".to_string(),
        description: None,
        input_schema: serde_json::json!({
            "type": "object",
            "my_custom_keyword": true
        }),
    }];

    let err = normalizer
        .normalize(request)
        .expect_err("normalization should fail");
    assert!(err.message.contains("unsupported keyword"));
}
