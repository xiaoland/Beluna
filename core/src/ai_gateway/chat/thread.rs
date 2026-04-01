use std::{collections::BTreeMap, sync::Arc, time::Instant};

use serde_json::{Value, json};
use tokio::sync::Mutex;

use crate::{
    ai_gateway::error::{GatewayError, GatewayErrorKind},
    observability::{metrics as observability_metrics, runtime as observability_runtime},
};

use super::{
    message::{AssistantMessage, Message},
    message_codec::current_timestamp_ms,
    runtime::{BoundBackend, ChatRuntime, next_request_id, turn_payload_json},
    thread_types::{TurnInput, TurnOutput, TurnQuery, TurnRef},
    tool::{ChatToolDefinition, resolve_tools},
    tool_scheduler::ToolScheduler,
    turn::Turn,
    types::{
        ChatEventStream, ChatMessage, ChatRole, ContentPart, FinishReason, OutputMode, TurnLimits,
        TurnPayload,
    },
};

#[derive(Clone)]
pub struct Thread {
    chat_id: String,
    thread_id: String,
    runtime: Arc<ChatRuntime>,
    pub(crate) state: Arc<Mutex<ThreadState>>,
}

pub(crate) struct ThreadState {
    pub backend: BoundBackend,
    pub turns: Vec<Turn>,
    pub tools: Vec<ChatToolDefinition>,
    pub system_prompt: Option<String>,
    pub default_output_mode: OutputMode,
    pub default_limits: TurnLimits,
    pub default_turn_timeout_ms: u64,
    pub enable_thinking: bool,
    pub next_turn_id: u64,
}

impl Thread {
    pub(crate) fn new(
        chat_id: String,
        thread_id: String,
        runtime: Arc<ChatRuntime>,
        state: ThreadState,
    ) -> Self {
        Self {
            chat_id,
            thread_id,
            runtime,
            state: Arc::new(Mutex::new(state)),
        }
    }

    pub fn thread_id(&self) -> &str {
        &self.thread_id
    }

    pub fn chat_id(&self) -> &str {
        &self.chat_id
    }

    pub async fn turns(&self) -> Vec<Turn> {
        self.state.lock().await.turns.clone()
    }

    pub async fn find_turns(&self, query: TurnQuery) -> Vec<TurnRef> {
        let guard = self.state.lock().await;
        let mut out = guard
            .turns
            .iter()
            .enumerate()
            .filter(|(_, turn)| match query.has_tool_calls {
                Some(expected) => turn.has_tool_calls() == expected,
                None => true,
            })
            .filter(|(_, turn)| match query.min_turn_id {
                Some(min_id) => turn.turn_id() >= min_id,
                None => true,
            })
            .filter(|(_, turn)| match query.max_turn_id {
                Some(max_id) => turn.turn_id() <= max_id,
                None => true,
            })
            .map(|(index, turn)| TurnRef {
                turn_id: turn.turn_id(),
                index,
            })
            .collect::<Vec<_>>();
        if let Some(limit) = query.limit {
            out.truncate(limit);
        }
        out
    }

    pub async fn complete(&self, input: TurnInput) -> Result<TurnOutput, GatewayError> {
        if input.messages.is_empty() {
            return Err(GatewayError::new(
                GatewayErrorKind::InvalidRequest,
                "turn requires at least one input message",
            )
            .with_retryable(false));
        }

        let mut guard = self.state.lock().await;
        let turn_id = guard.next_turn_id;
        let started_at = Instant::now();

        let effective_tools = resolve_tools(&guard.tools, &input.tool_overrides);
        let mut limits = input.limits.unwrap_or_else(|| guard.default_limits.clone());
        if limits.max_request_time_ms.is_none() {
            limits.max_request_time_ms = Some(guard.default_turn_timeout_ms);
        }

        let output_mode = input
            .output_mode
            .unwrap_or_else(|| guard.default_output_mode.clone());
        let enable_thinking = input.enable_thinking.unwrap_or(guard.enable_thinking);

        let mut metadata = input.metadata.clone();
        metadata.insert("chat_id".to_string(), self.chat_id.clone());
        metadata.insert("thread_id".to_string(), self.thread_id.clone());
        metadata.insert("turn_id".to_string(), turn_id.to_string());

        let payload = TurnPayload {
            messages: Arc::new(build_dispatch_messages(&guard, &input.messages)),
            tools: effective_tools,
            output_mode,
            limits,
            enable_thinking,
            metadata: metadata.clone(),
        };
        let request_id = next_request_id(&guard.backend.backend_id, &guard.backend.model);
        let dispatch_payload = turn_payload_json(&payload);

        observability_runtime::emit_ai_gateway_chat_turn(
            observability_runtime::AiGatewayChatTurnArgs {
                tick: metadata_tick(&metadata),
                thread_id: self.thread_id.clone(),
                turn_id,
                span_id: format!("ai-gateway.chat.turn:{}:{turn_id}", self.thread_id),
                parent_span_id: metadata_parent_span_id(&metadata),
                organ_id: metadata.get("organ_id").cloned(),
                request_id: Some(request_id.clone()),
                status: "started".to_string(),
                dispatch_payload: dispatch_payload.clone(),
                messages_when_committed: None,
                metadata: serde_json::to_value(&metadata)
                    .unwrap_or_else(|_| json!({ "serialization_error": true })),
                finish_reason: None,
                usage: None,
                backend_metadata: Some(backend_summary_value(&guard.backend)),
                error: None,
            },
        );

        let mut response = match self
            .runtime
            .dispatch_complete(&guard.backend, request_id.clone(), &payload)
            .await
        {
            Ok(response) => response,
            Err(err) => {
                observability_runtime::emit_ai_gateway_chat_turn(
                    observability_runtime::AiGatewayChatTurnArgs {
                        tick: metadata_tick(&metadata),
                        thread_id: self.thread_id.clone(),
                        turn_id,
                        span_id: format!("ai-gateway.chat.turn:{}:{turn_id}", self.thread_id),
                        parent_span_id: metadata_parent_span_id(&metadata),
                        organ_id: metadata.get("organ_id").cloned(),
                        request_id: Some(request_id.clone()),
                        status: "error".to_string(),
                        dispatch_payload: dispatch_payload.clone(),
                        messages_when_committed: None,
                        metadata: serde_json::to_value(&metadata)
                            .unwrap_or_else(|_| json!({ "serialization_error": true })),
                        finish_reason: None,
                        usage: None,
                        backend_metadata: Some(backend_summary_value(&guard.backend)),
                        error: Some(json!(err.clone())),
                    },
                );
                return Err(err);
            }
        };
        let mut turn = Turn::new(turn_id);
        *turn.metadata_mut() = metadata.clone();
        for input_message in input.messages {
            for message in Message::from_chat_message(input_message) {
                turn.append_one(message, None).await?;
            }
        }

        let assistant_message = Message::Assistant(AssistantMessage {
            id: format!("assistant-{}", turn_id),
            created_at_ms: current_timestamp_ms(),
            parts: vec![ContentPart::Text {
                text: response.output_text.clone(),
            }],
            tool_calls: if input.tool_executor.is_some() {
                Vec::new()
            } else {
                response
                    .tool_calls
                    .iter()
                    .map(|call| super::types::MessageToolCall {
                        id: call.id.clone(),
                        name: call.name.clone(),
                        arguments_json: call.arguments_json.clone(),
                    })
                    .collect::<Vec<_>>()
            },
        });
        turn.append_one(assistant_message, None).await?;

        if let Some(executor) = input.tool_executor.as_ref() {
            let scheduler = ToolScheduler::new(
                self.chat_id.clone(),
                self.thread_id.clone(),
                turn_id,
                Arc::clone(executor),
            );
            for call in &response.tool_calls {
                turn.append_one(Message::tool_call_from_result(call), Some(&scheduler))
                    .await?;
            }
            if !response.tool_calls.is_empty() {
                response.output_text.clear();
                response.tool_calls.clear();
                response.finish_reason = FinishReason::ToolCalls;
                response.pending_tool_call_continuation = true;
            } else {
                response.pending_tool_call_continuation = false;
            }
        } else {
            response.pending_tool_call_continuation = false;
        }

        turn.finalize(response.usage.clone(), response.finish_reason.clone());

        guard.turns.push(turn);
        guard.next_turn_id = guard.next_turn_id.saturating_add(1);
        let committed_turn = guard.turns.last().cloned();

        observability_metrics::set_chat_thread_last_turn_latency_ms(
            &self.chat_id,
            &self.thread_id,
            started_at.elapsed().as_millis() as u64,
        );
        observability_metrics::increment_chat_thread_turns_total(&self.chat_id, &self.thread_id);
        if let Some(committed_turn) = committed_turn.as_ref() {
            observability_runtime::emit_ai_gateway_chat_turn(
                observability_runtime::AiGatewayChatTurnArgs {
                    tick: metadata_tick(committed_turn.metadata()),
                    thread_id: self.thread_id.clone(),
                    turn_id,
                    span_id: format!("ai-gateway.chat.turn:{}:{turn_id}", self.thread_id),
                    parent_span_id: metadata_parent_span_id(committed_turn.metadata()),
                    organ_id: committed_turn.metadata().get("organ_id").cloned(),
                    request_id: Some(request_id.clone()),
                    status: turn_status_label(&response).to_string(),
                    dispatch_payload: dispatch_payload.clone(),
                    messages_when_committed: serde_json::to_value(committed_turn.messages()).ok(),
                    metadata: serde_json::to_value(committed_turn.metadata())
                        .unwrap_or_else(|_| json!({ "serialization_error": true })),
                    finish_reason: serde_json::to_value(&response.finish_reason).ok(),
                    usage: option_value(response.usage.as_ref()),
                    backend_metadata: serde_json::to_value(&response.backend_metadata).ok(),
                    error: None,
                },
            );
        }
        observability_runtime::emit_ai_gateway_chat_thread(
            observability_runtime::AiGatewayChatThreadArgs {
                tick: metadata_tick(&metadata),
                thread_id: self.thread_id.clone(),
                span_id: format!("ai-gateway.chat.thread:turn-committed:{}", self.thread_id),
                parent_span_id: metadata_parent_span_id(&metadata),
                organ_id: metadata.get("organ_id").cloned(),
                request_id: Some(request_id.clone()),
                kind: "turn_committed".to_string(),
                messages: thread_messages_snapshot(&guard.turns),
                turn_summaries: Some(turn_summaries_value(&guard.turns)),
                source_thread_id: None,
                source_turn_ids: None,
                kept_turn_ids: None,
                dropped_turn_ids: None,
                continuation_dropped: None,
                context_reason: metadata
                    .get("context_reason")
                    .cloned()
                    .or_else(|| Some("turn_committed".to_string())),
            },
        );

        Ok(TurnOutput {
            chat_id: self.chat_id.clone(),
            thread_id: self.thread_id.clone(),
            turn_id,
            response,
        })
    }

    pub async fn append_turn(&self, turn: Turn) -> Result<(), GatewayError> {
        turn.validate_tool_linkage()?;
        let mut guard = self.state.lock().await;
        if turn.turn_id() != guard.next_turn_id {
            return Err(GatewayError::new(
                GatewayErrorKind::InvalidRequest,
                format!(
                    "turn_id '{}' is out of sequence; expected '{}'",
                    turn.turn_id(),
                    guard.next_turn_id
                ),
            )
            .with_retryable(false));
        }
        guard.turns.push(turn);
        guard.next_turn_id = guard.next_turn_id.saturating_add(1);
        Ok(())
    }

    pub async fn stream(&self, _input: TurnInput) -> Result<ChatEventStream, GatewayError> {
        Err(GatewayError::new(
            GatewayErrorKind::UnsupportedCapability,
            "thread streaming is not implemented; use complete()",
        )
        .with_retryable(false))
    }
}

pub(crate) fn metadata_tick(metadata: &BTreeMap<String, String>) -> u64 {
    metadata
        .get("tick")
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(0)
}

pub(crate) fn metadata_parent_span_id(metadata: &BTreeMap<String, String>) -> Option<String> {
    metadata
        .get("parent_span_id")
        .cloned()
        .or_else(|| metadata.get("request_id").cloned())
}

fn option_value<T: serde::Serialize>(value: Option<&T>) -> Option<Value> {
    value.and_then(|item| serde_json::to_value(item).ok())
}

fn backend_summary_value(backend: &BoundBackend) -> Value {
    json!({
        "backend_id": backend.backend_id,
        "model": backend.model,
    })
}

fn turn_status_label(response: &super::types::TurnResponse) -> &'static str {
    if response.pending_tool_call_continuation {
        "committed_pending_continuation"
    } else {
        "committed"
    }
}

pub(crate) fn thread_messages_snapshot(turns: &[Turn]) -> Value {
    Value::Array(
        turns
            .iter()
            .flat_map(|turn| {
                turn.messages()
                    .iter()
                    .enumerate()
                    .map(move |(message_index, message)| {
                        json!({
                            "turn_id": turn.turn_id(),
                            "message_index": message_index,
                            "message": message,
                        })
                    })
            })
            .collect(),
    )
}

fn turn_summaries_value(turns: &[Turn]) -> Value {
    Value::Array(
        turns
            .iter()
            .map(|turn| {
                json!({
                    "turn_id": turn.turn_id(),
                    "message_count": turn.message_count(),
                    "tool_call_count": turn.tool_call_count(),
                    "completed": turn.completed(),
                    "metadata": turn.metadata(),
                    "usage": turn.usage(),
                    "finish_reason": turn.finish_reason(),
                })
            })
            .collect(),
    )
}

pub(crate) fn thread_turn_summaries(turns: &[Turn]) -> Value {
    turn_summaries_value(turns)
}

fn build_dispatch_messages(
    state: &ThreadState,
    input_messages: &[ChatMessage],
) -> Vec<ChatMessage> {
    let mut messages = Vec::new();
    if let Some(system_prompt) = state.system_prompt.as_ref() {
        messages.push(ChatMessage {
            role: ChatRole::System,
            parts: vec![ContentPart::Text {
                text: system_prompt.clone(),
            }],
            tool_call_id: None,
            tool_name: None,
            tool_calls: Vec::new(),
        });
    }

    for turn in &state.turns {
        messages.extend(turn.messages().iter().map(Message::to_chat_message));
    }
    messages.extend_from_slice(input_messages);
    messages
}
