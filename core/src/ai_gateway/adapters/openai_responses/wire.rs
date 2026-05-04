//! OpenAI Responses API wire-format serialization.

use serde_json::{Value, json};

use crate::ai_gateway::{
    chat::{
        tool::ChatToolDefinition,
        types::{ChatMessage, ChatRole, ContentPart, MessageToolCall, OutputMode, TurnPayload},
    },
    error::{GatewayError, GatewayErrorKind},
};

pub(crate) fn build_request_body(
    model: &str,
    payload: &TurnPayload,
    allow_parallel_tool_calls: bool,
) -> Result<Value, GatewayError> {
    let (instructions, input) = messages_to_responses(&payload.messages)?;
    let mut body = json!({
        "model": model,
        "input": input,
        "store": false,
    });

    if let Some(instructions) = instructions {
        body["instructions"] = Value::String(instructions);
    }

    if !payload.tools.is_empty() {
        body["tools"] = Value::Array(tools_to_responses(&payload.tools));
        body["tool_choice"] = Value::String("auto".to_string());
        body["parallel_tool_calls"] = Value::Bool(allow_parallel_tool_calls);
    }

    match &payload.output_mode {
        OutputMode::JsonObject => {
            body["text"] = json!({
                "format": {
                    "type": "json_object"
                }
            });
        }
        OutputMode::JsonSchema {
            name,
            schema,
            strict,
        } => {
            body["text"] = json!({
                "format": {
                    "type": "json_schema",
                    "name": name,
                    "schema": schema,
                    "strict": strict
                }
            });
        }
        OutputMode::Text => {}
    }

    if let Some(max_tokens) = payload.limits.max_output_tokens {
        body["max_output_tokens"] = Value::Number(max_tokens.into());
    }

    Ok(body)
}

fn messages_to_responses(
    messages: &[ChatMessage],
) -> Result<(Option<String>, Vec<Value>), GatewayError> {
    let system_count = messages
        .iter()
        .filter(|message| message.role == ChatRole::System)
        .count();
    let mut instructions = None;
    let mut input = Vec::new();

    for message in messages {
        if message.role == ChatRole::System && system_count == 1 {
            instructions = Some(parts_to_text(&message.parts));
            continue;
        }
        input.extend(message_to_input_items(message)?);
    }

    Ok((instructions, input))
}

fn message_to_input_items(message: &ChatMessage) -> Result<Vec<Value>, GatewayError> {
    match message.role {
        ChatRole::System | ChatRole::User => Ok(vec![message_item(
            role_to_responses(&message.role),
            parts_to_text(&message.parts),
        )]),
        ChatRole::Assistant => {
            let mut items = Vec::new();
            let text = parts_to_text(&message.parts);
            if !text.is_empty() {
                items.push(message_item("assistant", text));
            }
            items.extend(message.tool_calls.iter().map(function_call_item));
            Ok(items)
        }
        ChatRole::Tool => {
            let call_id = message.tool_call_id.clone().ok_or_else(|| {
                GatewayError::new(
                    GatewayErrorKind::InvalidRequest,
                    "responses tool message requires tool_call_id",
                )
                .with_retryable(false)
            })?;
            Ok(vec![json!({
                "type": "function_call_output",
                "call_id": call_id,
                "output": parts_to_text(&message.parts),
            })])
        }
    }
}

fn message_item(role: &str, content: String) -> Value {
    json!({
        "type": "message",
        "role": role,
        "content": content,
    })
}

fn function_call_item(call: &MessageToolCall) -> Value {
    json!({
        "type": "function_call",
        "call_id": call.id,
        "name": call.name,
        "arguments": call.arguments_json,
    })
}

fn tools_to_responses(tools: &[ChatToolDefinition]) -> Vec<Value> {
    tools
        .iter()
        .map(|tool| {
            let mut map = serde_json::Map::new();
            map.insert("type".to_string(), Value::String("function".to_string()));
            map.insert("name".to_string(), Value::String(tool.name.clone()));
            if let Some(description) = &tool.description {
                map.insert(
                    "description".to_string(),
                    Value::String(description.clone()),
                );
            }
            map.insert("parameters".to_string(), tool.input_schema.clone());
            Value::Object(map)
        })
        .collect()
}

fn role_to_responses(role: &ChatRole) -> &'static str {
    match role {
        ChatRole::System => "system",
        ChatRole::User => "user",
        ChatRole::Assistant => "assistant",
        ChatRole::Tool => "tool",
    }
}

fn parts_to_text(parts: &[ContentPart]) -> String {
    parts
        .iter()
        .map(|part| match part {
            ContentPart::Text { text } => text.clone(),
            ContentPart::Json { value } => value.to_string(),
            ContentPart::ImageUrl { url, .. } => url.clone(),
        })
        .collect::<Vec<_>>()
        .join("\n")
}
