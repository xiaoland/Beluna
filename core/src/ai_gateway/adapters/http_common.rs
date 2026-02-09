use std::collections::BTreeMap;

use serde_json::{Value, json};

use crate::ai_gateway::{
    error::{GatewayError, GatewayErrorKind},
    types::{
        CanonicalContentPart, CanonicalMessage, CanonicalRole, CanonicalToolChoice,
        CanonicalToolDefinition, FinishReason,
    },
};

pub fn role_to_wire(role: &CanonicalRole) -> &'static str {
    match role {
        CanonicalRole::System => "system",
        CanonicalRole::User => "user",
        CanonicalRole::Assistant => "assistant",
        CanonicalRole::Tool => "tool",
    }
}

pub fn canonical_messages_to_openai(messages: &[CanonicalMessage]) -> Vec<Value> {
    messages
        .iter()
        .map(|message| {
            let content = if message.parts.len() == 1 {
                part_to_simple_content(&message.parts[0])
            } else {
                Value::Array(
                    message
                        .parts
                        .iter()
                        .map(part_to_multi_content)
                        .collect::<Vec<_>>(),
                )
            };

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
            Value::Object(obj)
        })
        .collect()
}

pub fn canonical_messages_to_ollama(messages: &[CanonicalMessage]) -> Vec<Value> {
    messages
        .iter()
        .map(|message| {
            let text = message
                .parts
                .iter()
                .filter_map(|part| match part {
                    CanonicalContentPart::Text { text } => Some(text.as_str()),
                    _ => None,
                })
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
            Value::Object(map)
        })
        .collect()
}

pub fn tool_choice_to_openai(choice: &CanonicalToolChoice) -> Value {
    match choice {
        CanonicalToolChoice::Auto => Value::String("auto".to_string()),
        CanonicalToolChoice::None => Value::String("none".to_string()),
        CanonicalToolChoice::Required => Value::String("required".to_string()),
        CanonicalToolChoice::Specific { name } => json!({
            "type": "function",
            "function": {"name": name}
        }),
    }
}

pub fn tools_to_openai(tools: &[CanonicalToolDefinition]) -> Vec<Value> {
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

pub fn tools_to_ollama(tools: &[CanonicalToolDefinition]) -> Vec<Value> {
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

pub fn parse_finish_reason(value: Option<&str>) -> FinishReason {
    match value.unwrap_or("stop") {
        "stop" => FinishReason::Stop,
        "length" => FinishReason::Length,
        "tool_calls" => FinishReason::ToolCalls,
        other => FinishReason::Other(other.to_string()),
    }
}

pub fn map_http_error(status: u16, backend_id: &str, body: &str) -> GatewayError {
    let normalized_body = body.chars().take(240).collect::<String>();

    let mut err = if status == 401 {
        GatewayError::new(GatewayErrorKind::Authentication, "authentication failed")
            .with_retryable(false)
    } else if status == 403 {
        GatewayError::new(GatewayErrorKind::Authorization, "authorization failed")
            .with_retryable(false)
    } else if status == 408 || status == 429 {
        GatewayError::new(
            GatewayErrorKind::RateLimited,
            format!("backend returned status {}", status),
        )
        .with_retryable(true)
    } else if (400..500).contains(&status) {
        GatewayError::new(
            GatewayErrorKind::InvalidRequest,
            format!("backend returned status {}", status),
        )
        .with_retryable(false)
    } else {
        GatewayError::new(
            GatewayErrorKind::BackendTransient,
            format!("backend returned status {}", status),
        )
        .with_retryable(true)
    };

    err = err
        .with_backend_id(backend_id.to_string())
        .with_provider_http_status(status);

    if !normalized_body.is_empty() {
        err.message = format!("{}: {}", err.message, normalized_body);
    }

    err
}

fn part_to_simple_content(part: &CanonicalContentPart) -> Value {
    match part {
        CanonicalContentPart::Text { text } => Value::String(text.clone()),
        CanonicalContentPart::Json { value } => value.clone(),
        CanonicalContentPart::ImageUrl { url, .. } => Value::String(url.clone()),
    }
}

fn part_to_multi_content(part: &CanonicalContentPart) -> Value {
    match part {
        CanonicalContentPart::Text { text } => json!({"type": "text", "text": text}),
        CanonicalContentPart::Json { value } => json!({"type": "json", "json": value}),
        CanonicalContentPart::ImageUrl { url, mime_type } => {
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
