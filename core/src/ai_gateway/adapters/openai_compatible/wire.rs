//! OpenAI-compatible wire-format serialization.

use std::collections::BTreeMap;

use serde_json::{Value, json};

use crate::ai_gateway::{
    adapters::wire::role_to_wire,
    chat::{
        tool::ChatToolDefinition,
        types::{ChatMessage, ContentPart, MessageToolCall},
    },
};

pub(crate) fn messages_to_openai(messages: &[ChatMessage]) -> Vec<Value> {
    messages
        .iter()
        .map(|message| {
            let content = message_content_to_openai(message);

            let mut obj = serde_json::Map::new();
            obj.insert(
                "role".to_string(),
                Value::String(role_to_wire(&message.role).to_string()),
            );
            obj.insert("content".to_string(), content);
            if let Some(tool_call_id) = &message.tool_call_id {
                obj.insert(
                    "tool_call_id".to_string(),
                    Value::String(tool_call_id.clone()),
                );
            }
            if let Some(tool_name) = &message.tool_name {
                obj.insert("name".to_string(), Value::String(tool_name.clone()));
            }
            if !message.tool_calls.is_empty() {
                obj.insert(
                    "tool_calls".to_string(),
                    Value::Array(
                        message
                            .tool_calls
                            .iter()
                            .map(tool_call_to_openai)
                            .collect::<Vec<_>>(),
                    ),
                );
            }
            Value::Object(obj)
        })
        .collect()
}

pub(crate) fn tools_to_openai(tools: &[ChatToolDefinition]) -> Vec<Value> {
    tools
        .iter()
        .map(|tool| {
            json!({
                "type": "function",
                "function": {
                    "name": tool.name,
                    "description": tool.description,
                    "parameters": tool.input_schema,
                }
            })
        })
        .collect()
}

fn message_content_to_openai(message: &ChatMessage) -> Value {
    match message.parts.len() {
        0 => Value::String(String::new()),
        1 => part_to_simple_content(&message.parts[0]),
        _ => Value::Array(
            message
                .parts
                .iter()
                .map(part_to_multi_content)
                .collect::<Vec<_>>(),
        ),
    }
}

fn tool_call_to_openai(call: &MessageToolCall) -> Value {
    json!({
        "id": call.id,
        "type": "function",
        "function": {
            "name": call.name,
            "arguments": call.arguments_json,
        }
    })
}

fn part_to_simple_content(part: &ContentPart) -> Value {
    match part {
        ContentPart::Text { text } => Value::String(text.clone()),
        ContentPart::Json { value } => Value::String(value.to_string()),
        ContentPart::ImageUrl { url, .. } => Value::String(url.clone()),
    }
}

fn part_to_multi_content(part: &ContentPart) -> Value {
    match part {
        ContentPart::Text { text } => json!({"type": "text", "text": text}),
        ContentPart::Json { value } => {
            json!({"type": "text", "text": value.to_string()})
        }
        ContentPart::ImageUrl { url, mime_type } => {
            let mut map = BTreeMap::new();
            map.insert("type".to_string(), Value::String("input_image".to_string()));
            map.insert("image_url".to_string(), Value::String(url.clone()));
            if let Some(mime_type) = mime_type {
                map.insert("mime_type".to_string(), Value::String(mime_type.clone()));
            }
            Value::Object(map.into_iter().collect())
        }
    }
}
