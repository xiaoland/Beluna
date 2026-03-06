use std::time::{SystemTime, UNIX_EPOCH};

use super::{
    message::{
        AssistantMessage, Message, SystemMessage, ToolCallMessage, ToolCallResultMessage,
        UserMessage, next_message_id,
    },
    types::{ChatMessage, ChatRole, ContentPart, MessageToolCall, ToolCallResult},
};

impl Message {
    pub fn to_chat_message(&self) -> ChatMessage {
        match self {
            Message::System(msg) => ChatMessage {
                role: ChatRole::System,
                parts: msg.parts.clone(),
                tool_call_id: None,
                tool_name: None,
                tool_calls: Vec::new(),
            },
            Message::User(msg) => ChatMessage {
                role: ChatRole::User,
                parts: msg.parts.clone(),
                tool_call_id: None,
                tool_name: None,
                tool_calls: Vec::new(),
            },
            Message::Assistant(msg) => ChatMessage {
                role: ChatRole::Assistant,
                parts: msg.parts.clone(),
                tool_call_id: None,
                tool_name: None,
                tool_calls: msg.tool_calls.clone(),
            },
            Message::ToolCall(msg) => ChatMessage {
                role: ChatRole::Assistant,
                parts: vec![ContentPart::Text {
                    text: String::new(),
                }],
                tool_call_id: None,
                tool_name: None,
                tool_calls: vec![MessageToolCall {
                    id: msg.call_id.clone(),
                    name: msg.name.clone(),
                    arguments_json: msg.arguments_json.clone(),
                }],
            },
            Message::ToolCallResult(msg) => ChatMessage {
                role: ChatRole::Tool,
                parts: vec![ContentPart::Json {
                    value: msg.payload.clone(),
                }],
                tool_call_id: Some(msg.call_id.clone()),
                tool_name: Some(msg.name.clone()),
                tool_calls: Vec::new(),
            },
        }
    }

    pub fn from_chat_message(message: ChatMessage) -> Vec<Message> {
        let message_id = next_message_id();
        let now_ms = current_timestamp_ms();
        match message.role {
            ChatRole::System => vec![Message::System(SystemMessage {
                id: message_id,
                created_at_ms: now_ms,
                parts: message.parts,
            })],
            ChatRole::User => vec![Message::User(UserMessage {
                id: message_id,
                created_at_ms: now_ms,
                parts: message.parts,
            })],
            ChatRole::Assistant => vec![Message::Assistant(AssistantMessage {
                id: message_id,
                created_at_ms: now_ms,
                parts: message.parts,
                tool_calls: message.tool_calls,
            })],
            ChatRole::Tool => vec![Message::ToolCallResult(ToolCallResultMessage {
                id: message_id,
                created_at_ms: now_ms,
                call_id: message.tool_call_id.unwrap_or_else(|| "unknown-call".to_string()),
                name: message.tool_name.unwrap_or_else(|| "unknown-tool".to_string()),
                payload: tool_payload_from_parts(message.parts),
            })],
        }
    }

    pub fn assistant_text(text: String) -> Message {
        Message::Assistant(AssistantMessage {
            id: next_message_id(),
            created_at_ms: current_timestamp_ms(),
            parts: vec![ContentPart::Text { text }],
            tool_calls: Vec::new(),
        })
    }

    pub fn tool_call_from_result(call: &ToolCallResult) -> Message {
        Message::ToolCall(ToolCallMessage {
            id: next_message_id(),
            created_at_ms: current_timestamp_ms(),
            call_id: call.id.clone(),
            name: call.name.clone(),
            arguments_json: call.arguments_json.clone(),
        })
    }

    pub fn tool_call_result(call_id: String, name: String, payload: serde_json::Value) -> Message {
        Message::ToolCallResult(ToolCallResultMessage {
            id: next_message_id(),
            created_at_ms: current_timestamp_ms(),
            call_id,
            name,
            payload,
        })
    }
}

pub fn current_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

pub fn tool_result_from_execution(call: &ToolCallResult, payload: serde_json::Value) -> Message {
    Message::ToolCallResult(ToolCallResultMessage {
        id: next_message_id(),
        created_at_ms: current_timestamp_ms(),
        call_id: call.id.clone(),
        name: call.name.clone(),
        payload,
    })
}

pub fn tool_result_error(call: &ToolCallResult, error: &str) -> Message {
    Message::ToolCallResult(ToolCallResultMessage {
        id: next_message_id(),
        created_at_ms: current_timestamp_ms(),
        call_id: call.id.clone(),
        name: call.name.clone(),
        payload: serde_json::json!({
            "ok": false,
            "tool": call.name,
            "error": error,
        }),
    })
}

fn tool_payload_from_parts(parts: Vec<ContentPart>) -> serde_json::Value {
    for part in &parts {
        if let ContentPart::Json { value } = part {
            return value.clone();
        }
    }

    let text = parts
        .into_iter()
        .filter_map(|part| match part {
            ContentPart::Text { text } => Some(text),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n");

    if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&text) {
        return json_value;
    }

    serde_json::json!({
        "ok": false,
        "raw": text,
    })
}
