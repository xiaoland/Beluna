//! Shared wire-format helpers used across multiple HTTP adapters.
//!
//! Only truly protocol-neutral helpers belong here. Per-adapter serialization
//! (messages, tools, tool_calls) lives in each adapter's own `wire` module.

use crate::ai_gateway::chat::types::{ChatRole, FinishReason};

/// Map a [`ChatRole`] to its standard wire-format string.
pub(crate) fn role_to_wire(role: &ChatRole) -> &'static str {
    match role {
        ChatRole::System => "system",
        ChatRole::User => "user",
        ChatRole::Assistant => "assistant",
        ChatRole::Tool => "tool",
    }
}

/// Parse a finish_reason string (or None) into a [`FinishReason`].
pub(crate) fn parse_finish_reason(value: Option<&str>) -> FinishReason {
    match value.unwrap_or("stop") {
        "stop" => FinishReason::Stop,
        "length" => FinishReason::Length,
        "tool_calls" => FinishReason::ToolCalls,
        other => FinishReason::Other(other.to_string()),
    }
}
