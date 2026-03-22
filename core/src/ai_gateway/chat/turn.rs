use std::collections::{BTreeMap, HashMap};

use crate::ai_gateway::error::{GatewayError, GatewayErrorKind};

use super::{
    message::{Message, MessageKind, ToolCallMessage, ToolCallResultMessage},
    tool_scheduler::ToolScheduler,
    types::{FinishReason, UsageStats},
};

#[derive(Debug, Clone)]
pub struct Turn {
    turn_id: u64,
    messages: Vec<Message>,
    metadata: BTreeMap<String, String>,
    usage: Option<UsageStats>,
    finish_reason: Option<FinishReason>,
    completed: bool,
}

impl Turn {
    pub fn new(turn_id: u64) -> Self {
        Self {
            turn_id,
            messages: Vec::new(),
            metadata: BTreeMap::new(),
            usage: None,
            finish_reason: None,
            completed: false,
        }
    }

    pub async fn append_one(
        &mut self,
        message: Message,
        scheduler: Option<&ToolScheduler>,
    ) -> Result<AppendOutcome, GatewayError> {
        let mut reset_messages_applied = false;
        match message {
            Message::ToolCall(tool_call_message) => {
                let Some(scheduler) = scheduler else {
                    return Err(GatewayError::new(
                        GatewayErrorKind::InvalidRequest,
                        "ToolCallMessage append requires ToolScheduler",
                    )
                    .with_retryable(false));
                };
                self.messages
                    .push(Message::ToolCall(tool_call_message.clone()));
                let (result, call_reset_messages_applied) =
                    scheduler.execute_tool_call(&tool_call_message).await;
                self.messages.push(result);
                reset_messages_applied = call_reset_messages_applied;
            }
            other => {
                self.messages.push(other);
            }
        }

        self.validate_tool_linkage()?;
        Ok(AppendOutcome {
            reset_messages_applied,
        })
    }

    pub fn truncate_one(&mut self) -> Result<(), GatewayError> {
        if self.messages.is_empty() {
            return Ok(());
        }

        let snapshot = self.messages.clone();
        let tail_is_tool_result = matches!(self.messages.last(), Some(Message::ToolCallResult(_)));
        if tail_is_tool_result {
            // A tool result is never truncated alone; its matching call is the same logical unit.
            let Some(Message::ToolCallResult(ToolCallResultMessage { call_id, .. })) =
                self.messages.pop()
            else {
                unreachable!("checked by tail_is_tool_result");
            };

            match self.messages.pop() {
                Some(Message::ToolCall(ToolCallMessage {
                    call_id: tool_call_id,
                    ..
                })) if tool_call_id == call_id => {}
                Some(_) => {
                    self.messages = snapshot;
                    return Err(GatewayError::new(
                        GatewayErrorKind::ProtocolViolation,
                        "truncate_one found a broken tool-call/result boundary",
                    )
                    .with_retryable(false));
                }
                None => {
                    self.messages = snapshot;
                    return Err(GatewayError::new(
                        GatewayErrorKind::ProtocolViolation,
                        "truncate_one cannot remove a standalone tool result",
                    )
                    .with_retryable(false));
                }
            }
        } else {
            let _ = self.messages.pop();
        }

        self.validate_tool_linkage()
    }

    pub fn validate_tool_linkage(&self) -> Result<(), GatewayError> {
        let mut index = 0_usize;
        while index < self.messages.len() {
            match &self.messages[index] {
                Message::ToolCall(call) => {
                    let Some(next) = self.messages.get(index + 1) else {
                        return Err(GatewayError::new(
                            GatewayErrorKind::InvalidRequest,
                            format!(
                                "turn {} is incomplete: tool call '{}' has no result",
                                self.turn_id, call.call_id
                            ),
                        )
                        .with_retryable(false));
                    };
                    let Message::ToolCallResult(result) = next else {
                        return Err(GatewayError::new(
                            GatewayErrorKind::InvalidRequest,
                            format!(
                                "turn {} is incomplete: tool call '{}' must be followed by ToolCallResultMessage",
                                self.turn_id, call.call_id
                            ),
                        )
                        .with_retryable(false));
                    };
                    if result.call_id != call.call_id {
                        return Err(GatewayError::new(
                            GatewayErrorKind::InvalidRequest,
                            format!(
                                "turn {} has mismatched tool linkage: call '{}' != result '{}'",
                                self.turn_id, call.call_id, result.call_id
                            ),
                        )
                        .with_retryable(false));
                    }
                    index += 2;
                }
                Message::ToolCallResult(result) => {
                    return Err(GatewayError::new(
                        GatewayErrorKind::InvalidRequest,
                        format!(
                            "turn {} has dangling ToolCallResultMessage '{}'",
                            self.turn_id, result.call_id
                        ),
                    )
                    .with_retryable(false));
                }
                _ => index += 1,
            }
        }
        Ok(())
    }

    pub fn has_tool_calls(&self) -> bool {
        self.messages
            .iter()
            .any(|message| matches!(message.kind(), MessageKind::ToolCall))
    }

    pub fn turn_id(&self) -> u64 {
        self.turn_id
    }

    pub(crate) fn set_turn_id(&mut self, turn_id: u64) {
        self.turn_id = turn_id;
    }

    pub fn messages(&self) -> &[Message] {
        &self.messages
    }

    pub fn metadata(&self) -> &BTreeMap<String, String> {
        &self.metadata
    }

    pub(crate) fn metadata_mut(&mut self) -> &mut BTreeMap<String, String> {
        &mut self.metadata
    }

    pub fn usage(&self) -> Option<&UsageStats> {
        self.usage.as_ref()
    }

    pub fn finish_reason(&self) -> Option<&FinishReason> {
        self.finish_reason.as_ref()
    }

    pub fn completed(&self) -> bool {
        self.completed
    }

    pub(crate) fn finalize(&mut self, usage: Option<UsageStats>, finish_reason: FinishReason) {
        self.usage = usage;
        self.finish_reason = Some(finish_reason);
        self.completed = true;
    }

    pub fn message_count(&self) -> usize {
        self.messages.len()
    }

    pub fn tool_call_count(&self) -> usize {
        self.messages
            .iter()
            .filter(|message| matches!(message.kind(), MessageKind::ToolCall))
            .count()
    }

    pub fn tool_result_payload_by_call_id(&self) -> HashMap<String, serde_json::Value> {
        let mut out = HashMap::new();
        for message in &self.messages {
            if let Message::ToolCallResult(result) = message {
                out.insert(result.call_id.clone(), result.payload.clone());
            }
        }
        out
    }
}

pub struct AppendOutcome {
    pub reset_messages_applied: bool,
}
