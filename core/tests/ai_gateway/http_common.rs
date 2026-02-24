use beluna::ai_gateway::{
    adapters::http_common::{canonical_messages_to_ollama, canonical_messages_to_openai},
    types_chat::{CanonicalContentPart, CanonicalMessage, CanonicalMessageToolCall, CanonicalRole},
};

#[test]
fn given_tool_json_message_when_mapped_to_openai_then_content_is_json_string() {
    let wire = canonical_messages_to_openai(&[CanonicalMessage {
        role: CanonicalRole::Tool,
        parts: vec![CanonicalContentPart::Json {
            value: serde_json::json!({"ok":true}),
        }],
        tool_call_id: Some("call_1".to_string()),
        tool_name: Some("expand-sense-raw".to_string()),
        tool_calls: vec![],
    }]);

    let content = wire[0]
        .get("content")
        .expect("tool message should include content");
    assert!(content.is_string());
    assert_eq!(content.as_str(), Some("{\"ok\":true}"));
}

#[test]
fn given_assistant_tool_calls_when_mapped_to_openai_then_rpc_fields_are_emitted() {
    let wire = canonical_messages_to_openai(&[CanonicalMessage {
        role: CanonicalRole::Assistant,
        parts: vec![],
        tool_call_id: None,
        tool_name: None,
        tool_calls: vec![CanonicalMessageToolCall {
            id: "call_1".to_string(),
            name: "expand-sense-raw".to_string(),
            arguments_json: "{\"sense_ids\":[1]}".to_string(),
        }],
    }]);

    assert_eq!(wire[0].get("content").and_then(|v| v.as_str()), Some(""));
    assert_eq!(
        wire[0]
            .get("tool_calls")
            .and_then(|v| v.as_array())
            .map(Vec::len),
        Some(1)
    );
    assert_eq!(
        wire[0]
            .get("tool_calls")
            .and_then(|v| v.get(0))
            .and_then(|v| v.get("id"))
            .and_then(|v| v.as_str()),
        Some("call_1")
    );
}

#[test]
fn given_tool_json_message_when_mapped_to_ollama_then_content_is_json_string() {
    let wire = canonical_messages_to_ollama(&[CanonicalMessage {
        role: CanonicalRole::Tool,
        parts: vec![CanonicalContentPart::Json {
            value: serde_json::json!({"ok":true}),
        }],
        tool_call_id: Some("call_1".to_string()),
        tool_name: Some("expand-sense-raw".to_string()),
        tool_calls: vec![],
    }]);

    assert_eq!(
        wire[0].get("content").and_then(|v| v.as_str()),
        Some("{\"ok\":true}")
    );
}
