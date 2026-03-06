use std::sync::Arc;

use super::{
    executor::{ToolExecutionRequest, ToolExecutionResult, ToolExecutor},
    message::{Message, ToolCallMessage},
    types::ToolCallResult,
};

#[derive(Clone)]
pub struct ToolScheduler {
    chat_id: String,
    thread_id: String,
    turn_id: u64,
    executor: Arc<dyn ToolExecutor>,
}

impl ToolScheduler {
    pub fn new(
        chat_id: String,
        thread_id: String,
        turn_id: u64,
        executor: Arc<dyn ToolExecutor>,
    ) -> Self {
        Self {
            chat_id,
            thread_id,
            turn_id,
            executor,
        }
    }

    pub async fn execute_tool_call(
        &self,
        call_message: &ToolCallMessage,
    ) -> (Message, bool) {
        let call = ToolCallResult {
            id: call_message.call_id.clone(),
            name: call_message.name.clone(),
            arguments_json: call_message.arguments_json.clone(),
            status: super::types::ToolCallStatus::Ready,
        };

        let request = ToolExecutionRequest {
            chat_id: self.chat_id.clone(),
            thread_id: self.thread_id.clone(),
            turn_id: self.turn_id,
            call: call.clone(),
        };

        match self.executor.execute_call(request).await {
            Ok(ToolExecutionResult {
                payload,
                reset_messages_applied,
            }) => (Message::tool_call_result(call.id, call.name, payload), reset_messages_applied),
            Err(err) => (
                Message::tool_call_result(
                    call.id,
                    call.name,
                    serde_json::json!({
                        "ok": false,
                        "error": err.message,
                    }),
                ),
                false,
            ),
        }
    }
}
