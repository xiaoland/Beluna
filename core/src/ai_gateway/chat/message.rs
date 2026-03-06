use serde::{Deserialize, Serialize};

use super::types::{ContentPart, MessageToolCall};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageKind {
    System,
    User,
    Assistant,
    ToolCall,
    ToolCallResult,
}

pub trait MessageTrait {
    fn id(&self) -> &str;
    fn created_at_ms(&self) -> u64;
    fn kind(&self) -> MessageKind;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMessage {
    pub id: String,
    pub created_at_ms: u64,
    pub parts: Vec<ContentPart>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserMessage {
    pub id: String,
    pub created_at_ms: u64,
    pub parts: Vec<ContentPart>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistantMessage {
    pub id: String,
    pub created_at_ms: u64,
    pub parts: Vec<ContentPart>,
    pub tool_calls: Vec<MessageToolCall>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallMessage {
    pub id: String,
    pub created_at_ms: u64,
    pub call_id: String,
    pub name: String,
    pub arguments_json: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallResultMessage {
    pub id: String,
    pub created_at_ms: u64,
    pub call_id: String,
    pub name: String,
    pub payload: serde_json::Value,
}

impl MessageTrait for SystemMessage {
    fn id(&self) -> &str {
        &self.id
    }

    fn created_at_ms(&self) -> u64 {
        self.created_at_ms
    }

    fn kind(&self) -> MessageKind {
        MessageKind::System
    }
}

impl MessageTrait for UserMessage {
    fn id(&self) -> &str {
        &self.id
    }

    fn created_at_ms(&self) -> u64 {
        self.created_at_ms
    }

    fn kind(&self) -> MessageKind {
        MessageKind::User
    }
}

impl MessageTrait for AssistantMessage {
    fn id(&self) -> &str {
        &self.id
    }

    fn created_at_ms(&self) -> u64 {
        self.created_at_ms
    }

    fn kind(&self) -> MessageKind {
        MessageKind::Assistant
    }
}

impl MessageTrait for ToolCallMessage {
    fn id(&self) -> &str {
        &self.id
    }

    fn created_at_ms(&self) -> u64 {
        self.created_at_ms
    }

    fn kind(&self) -> MessageKind {
        MessageKind::ToolCall
    }
}

impl MessageTrait for ToolCallResultMessage {
    fn id(&self) -> &str {
        &self.id
    }

    fn created_at_ms(&self) -> u64 {
        self.created_at_ms
    }

    fn kind(&self) -> MessageKind {
        MessageKind::ToolCallResult
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Message {
    System(SystemMessage),
    User(UserMessage),
    Assistant(AssistantMessage),
    ToolCall(ToolCallMessage),
    ToolCallResult(ToolCallResultMessage),
}

impl Message {
    pub fn id(&self) -> &str {
        match self {
            Message::System(msg) => &msg.id,
            Message::User(msg) => &msg.id,
            Message::Assistant(msg) => &msg.id,
            Message::ToolCall(msg) => &msg.id,
            Message::ToolCallResult(msg) => &msg.id,
        }
    }

    pub fn kind(&self) -> MessageKind {
        match self {
            Message::System(_) => MessageKind::System,
            Message::User(_) => MessageKind::User,
            Message::Assistant(_) => MessageKind::Assistant,
            Message::ToolCall(_) => MessageKind::ToolCall,
            Message::ToolCallResult(_) => MessageKind::ToolCallResult,
        }
    }
}

pub(crate) fn next_message_id() -> String {
    static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    format!(
        "msg-{}",
        SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    )
}
