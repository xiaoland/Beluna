//! Ollama wire-format serialization.

use serde_json::{Value, json};

use crate::ai_gateway::{
    adapters::wire::role_to_wire,
    chat::{
        tool::ChatToolDefinition,
        types::{ChatMessage, ContentPart, MessageToolCall},
    },
};

pub(crate) fn messages_to_ollama(messages: &[ChatMessage]) -> Vec<Value> {
    messages
        .iter()
        .map(|message| {
            let text = message
                .parts
                .iter()
                .map(part_to_wire_text)
                .collect::<Vec<_>>()
                .join("");

            let mut map = serde_json::Map::new();
            map.insert(
                "role".into(),
                Value::String(role_to_wire(&message.role).to_string()),
            );
            map.insert("content".into(), Value::String(text));
            if let Some(tool_call_id) = &message.tool_call_id {
                map.insert("tool_call_id".into(), Value::String(tool_call_id.clone()));
            }
            if let Some(tool_name) = &message.tool_name {
                map.insert("name".into(), Value::String(tool_name.clone()));
            }
            if !message.tool_calls.is_empty() {
                map.insert(
                    "tool_calls".into(),
                    Value::Array(
                        message
                            .tool_calls
                            .iter()
                            .map(tool_call_to_ollama)
                            .collect::<Vec<_>>(),
                    ),
                );
            }
            Value::Object(map)
        })
        .collect()
}

pub(crate) fn tools_to_ollama(tools: &[ChatToolDefinition]) -> Vec<Value> {
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

fn tool_call_to_ollama(call: &MessageToolCall) -> Value {
    json!({
        "id": call.id,
        "type": "function",
        "function": {
            "name": call.name,
            "arguments": call.arguments_json,
        }
    })
}

fn part_to_wire_text(part: &ContentPart) -> String {
    match part {
        ContentPart::Text { text } => text.clone(),
        ContentPart::Json { value } => value.to_string(),
        ContentPart::ImageUrl { url, .. } => url.clone(),
    }
}
