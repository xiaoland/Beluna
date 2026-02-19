use std::collections::BTreeMap;

use beluna::ai_gateway::{
    request_normalizer::RequestNormalizer,
    types_chat::{
        BelunaContentPart, BelunaMessage, BelunaRole, BelunaToolDefinition, ChatRequest,
        OutputMode, RequestLimitOverrides, ToolChoice,
    },
};

fn base_request() -> ChatRequest {
    ChatRequest {
        request_id: None,
        route: Some("default".to_string()),
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
        cost_attribution_id: None,
    }
}

#[test]
fn given_request_id_is_missing_when_normalized_then_non_empty_request_id_is_generated() {
    let normalizer = RequestNormalizer;
    let request = base_request();
    let normalized = normalizer
        .normalize_chat(request, true)
        .expect("normalization should succeed");
    assert!(!normalized.request_id.is_empty());
}

#[test]
fn given_tool_message_without_tool_call_id_when_normalized_then_invalid_request_is_returned() {
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
        .normalize_chat(request, true)
        .expect_err("normalization should fail");
    assert!(err.message.contains("tool_call_id"));
}

#[test]
fn given_tool_message_with_image_part_when_normalized_then_invalid_request_is_returned() {
    let normalizer = RequestNormalizer;
    let mut request = base_request();
    request.messages = vec![BelunaMessage {
        role: BelunaRole::Tool,
        parts: vec![BelunaContentPart::ImageUrl {
            url: "https://example.com/image.png".to_string(),
            mime_type: Some("image/png".to_string()),
        }],
        tool_call_id: Some("call-1".to_string()),
        tool_name: Some("my_tool".to_string()),
    }];

    let err = normalizer
        .normalize_chat(request, true)
        .expect_err("normalization should fail");
    assert!(err.message.contains("text/json"));
}

#[test]
fn given_non_tool_message_with_tool_linkage_when_normalized_then_invalid_request_is_returned() {
    let normalizer = RequestNormalizer;
    let mut request = base_request();
    request.messages[0].tool_call_id = Some("abc".to_string());

    let err = normalizer
        .normalize_chat(request, true)
        .expect_err("normalization should fail");
    assert!(err.message.contains("non-tool"));
}

#[test]
fn given_tool_schema_with_unknown_keyword_when_normalized_then_invalid_request_is_returned() {
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
        .normalize_chat(request, true)
        .expect_err("normalization should fail");
    assert!(err.message.contains("unsupported keyword"));
}
