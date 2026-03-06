use std::sync::Arc;

use async_trait::async_trait;
use beluna::ai_gateway::{
    chat::{
        ContentPart, Message, ToolCallMessage, ToolExecutionRequest, ToolExecutionResult,
        ToolExecutor, Turn, UserMessage, tool_scheduler::ToolScheduler,
    },
    error::{GatewayError, GatewayErrorKind},
};

struct SuccessfulExecutor;

#[async_trait]
impl ToolExecutor for SuccessfulExecutor {
    async fn execute_call(
        &self,
        request: ToolExecutionRequest,
    ) -> Result<ToolExecutionResult, GatewayError> {
        Ok(ToolExecutionResult {
            payload: serde_json::json!({
                "ok": true,
                "tool": request.call.name,
            }),
            reset_messages_applied: true,
        })
    }
}

struct FailingExecutor;

#[async_trait]
impl ToolExecutor for FailingExecutor {
    async fn execute_call(
        &self,
        _request: ToolExecutionRequest,
    ) -> Result<ToolExecutionResult, GatewayError> {
        Err(
            GatewayError::new(GatewayErrorKind::Internal, "tool execution failed")
                .with_retryable(false),
        )
    }
}

fn tool_call_message(call_id: &str) -> Message {
    Message::ToolCall(ToolCallMessage {
        id: format!("msg-{call_id}"),
        created_at_ms: 1,
        call_id: call_id.to_string(),
        name: "demo-tool".to_string(),
        arguments_json: "{\"arg\":1}".to_string(),
    })
}

fn user_message(message_id: &str, text: &str) -> Message {
    Message::User(UserMessage {
        id: message_id.to_string(),
        created_at_ms: 1,
        parts: vec![ContentPart::Text {
            text: text.to_string(),
        }],
    })
}

#[tokio::test]
async fn append_one_tool_call_appends_linked_result_and_truncate_removes_bundle() {
    let scheduler = ToolScheduler::new(
        "chat-1".to_string(),
        "thread-1".to_string(),
        1,
        Arc::new(SuccessfulExecutor),
    );
    let mut turn = Turn::new(1);

    let outcome = turn
        .append_one(tool_call_message("call-1"), Some(&scheduler))
        .await
        .expect("append_one should succeed");

    assert!(outcome.reset_messages_applied);
    assert_eq!(turn.message_count(), 2);
    assert_eq!(turn.tool_call_count(), 1);
    assert_eq!(
        turn.tool_result_payload_by_call_id().get("call-1"),
        Some(&serde_json::json!({
            "ok": true,
            "tool": "demo-tool",
        }))
    );

    turn.truncate_one().expect("truncate_one should succeed");
    assert_eq!(turn.message_count(), 0);
}

#[tokio::test]
async fn truncate_one_removes_trailing_tool_result_and_matching_call_together() {
    let scheduler = ToolScheduler::new(
        "chat-1".to_string(),
        "thread-1".to_string(),
        1,
        Arc::new(SuccessfulExecutor),
    );
    let mut turn = Turn::new(1);

    turn.append_one(user_message("user-1", "keep me"), None)
        .await
        .expect("user append should succeed");
    turn.append_one(tool_call_message("call-keep-boundary"), Some(&scheduler))
        .await
        .expect("tool append should succeed");

    turn.truncate_one().expect("truncate_one should succeed");

    assert_eq!(turn.message_count(), 1);
    match &turn.messages()[0] {
        Message::User(message) => match &message.parts[0] {
            ContentPart::Text { text } => assert_eq!(text, "keep me"),
            other => panic!("expected text content, got {other:?}"),
        },
        other => panic!("expected retained user message, got {other:?}"),
    }
}

#[tokio::test]
async fn append_one_tool_call_captures_tool_failure_as_result_message() {
    let scheduler = ToolScheduler::new(
        "chat-1".to_string(),
        "thread-1".to_string(),
        1,
        Arc::new(FailingExecutor),
    );
    let mut turn = Turn::new(1);

    turn.append_one(tool_call_message("call-2"), Some(&scheduler))
        .await
        .expect("append_one should still succeed");

    assert_eq!(turn.message_count(), 2);
    assert_eq!(
        turn.tool_result_payload_by_call_id().get("call-2"),
        Some(&serde_json::json!({
            "ok": false,
            "error": "tool execution failed",
        }))
    );
}
