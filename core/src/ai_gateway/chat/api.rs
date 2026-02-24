use std::{sync::Arc, time::Instant};

use crate::{
    ai_gateway::{
        error::{GatewayError, GatewayErrorKind},
        gateway::AIGateway,
        types_chat::{
            BelunaContentPart, BelunaMessage, BelunaMessageToolCall, ChatEventStream, ChatRequest,
        },
    },
    observability::metrics as observability_metrics,
};

use super::{
    session_store::{InMemoryChatSessionStore, TurnCommitOutcome},
    types::{
        ChatSessionOpenRequest, ChatThreadOpenRequest, ChatThreadState, ChatTurnRequest,
        ChatTurnResponse,
    },
};

#[derive(Clone)]
pub struct ChatGateway {
    gateway: Arc<AIGateway>,
    store: Arc<InMemoryChatSessionStore>,
    default_route_ref: Option<String>,
    default_turn_timeout_ms: u64,
}

#[derive(Clone)]
pub struct ChatSessionHandle {
    chat_gateway: ChatGateway,
    session_id: String,
}

#[derive(Clone)]
pub struct ChatThreadHandle {
    chat_gateway: ChatGateway,
    session_id: String,
    thread_id: String,
}

impl ChatGateway {
    pub(crate) fn new(
        gateway: Arc<AIGateway>,
        store: Arc<InMemoryChatSessionStore>,
        default_route_ref: Option<String>,
        default_turn_timeout_ms: u64,
    ) -> Self {
        Self {
            gateway,
            store,
            default_route_ref,
            default_turn_timeout_ms,
        }
    }

    pub async fn open_session(
        &self,
        request: ChatSessionOpenRequest,
    ) -> Result<ChatSessionHandle, GatewayError> {
        let (session_id, default_route_ref) = self
            .store
            .open_session(request, self.default_route_ref.clone())
            .await?;

        tracing::info!(
            target: "ai_gateway",
            event = "chat_session_lifecycle",
            action = "opened",
            session_id = %session_id,
            default_route_ref = default_route_ref.as_deref().unwrap_or("-"),
            "chat_session_lifecycle"
        );

        Ok(ChatSessionHandle {
            chat_gateway: self.clone(),
            session_id,
        })
    }
}

impl ChatSessionHandle {
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    pub async fn open_thread(
        &self,
        request: ChatThreadOpenRequest,
    ) -> Result<ChatThreadHandle, GatewayError> {
        let thread_id = self
            .chat_gateway
            .store
            .open_thread(&self.session_id, request)
            .await?;

        Ok(ChatThreadHandle {
            chat_gateway: self.chat_gateway.clone(),
            session_id: self.session_id.clone(),
            thread_id,
        })
    }

    pub async fn close(&self) {
        self.chat_gateway.store.close_session(&self.session_id).await;
        tracing::info!(
            target: "ai_gateway",
            event = "chat_session_lifecycle",
            action = "closed",
            session_id = %self.session_id,
            "chat_session_lifecycle"
        );
    }
}

impl ChatThreadHandle {
    pub fn thread_id(&self) -> &str {
        &self.thread_id
    }

    pub async fn state(&self) -> Result<ChatThreadState, GatewayError> {
        self.chat_gateway
            .store
            .thread_state(&self.session_id, &self.thread_id)
            .await
    }

    pub async fn turn_once(
        &self,
        mut request: ChatTurnRequest,
    ) -> Result<ChatTurnResponse, GatewayError> {
        if request.input_messages.is_empty() {
            return Err(
                GatewayError::new(
                    GatewayErrorKind::InvalidRequest,
                    "chat turn requires at least one input message",
                )
                .with_retryable(false),
            );
        }

        let started_at = Instant::now();
        let prepared = self
            .chat_gateway
            .store
            .prepare_turn(
                &self.session_id,
                &self.thread_id,
                request.route_ref_override.clone(),
                self.chat_gateway.default_route_ref.clone(),
                &request.input_messages,
            )
            .await?;

        let turn_id = prepared.turn_id;
        let request_id = request.request_id.take().unwrap_or_else(|| {
            format!(
                "chat-{}-{}-turn-{}",
                self.session_id, self.thread_id, turn_id
            )
        });

        if request.limits.max_request_time_ms.is_none() {
            request.limits.max_request_time_ms = Some(self.chat_gateway.default_turn_timeout_ms);
        }
        request
            .metadata
            .entry("chat_session_id".to_string())
            .or_insert_with(|| self.session_id.clone());
        request
            .metadata
            .entry("chat_thread_id".to_string())
            .or_insert_with(|| self.thread_id.clone());
        request
            .metadata
            .entry("chat_turn_id".to_string())
            .or_insert_with(|| turn_id.to_string());

        let dispatch_request = ChatRequest {
            request_id: Some(request_id),
            route: prepared.route_ref,
            messages: prepared.messages,
            tools: request.tools,
            tool_choice: request.tool_choice,
            output_mode: request.output_mode,
            limits: request.limits,
            metadata: request.metadata,
            cost_attribution_id: request.cost_attribution_id,
        };

        let response_result = self.chat_gateway.gateway.chat_once(dispatch_request).await;
        let latency_ms = started_at.elapsed().as_millis() as u64;

        match response_result {
            Ok(response) => {
                let assistant_message = assistant_message_from_response(&response);
                let commit = self
                    .chat_gateway
                    .store
                    .commit_turn_success(
                        &self.session_id,
                        &self.thread_id,
                        request.input_messages,
                        assistant_message,
                        response.usage.clone(),
                        response.tool_calls.len(),
                        latency_ms,
                    )
                    .await?;

                emit_turn_summary(
                    &self.session_id,
                    &self.thread_id,
                    turn_id,
                    &response,
                    &commit,
                    latency_ms,
                    "ok",
                );

                record_turn_metrics(
                    &self.session_id,
                    &self.thread_id,
                    &response,
                    &commit,
                    latency_ms,
                );

                Ok(ChatTurnResponse {
                    session_id: self.session_id.clone(),
                    thread_id: self.thread_id.clone(),
                    turn_id,
                    response,
                })
            }
            Err(err) => {
                let commit = self
                    .chat_gateway
                    .store
                    .commit_turn_failure(&self.session_id, &self.thread_id, latency_ms)
                    .await?;

                tracing::info!(
                    target: "ai_gateway",
                    event = "chat_turn_anomaly",
                    session_id = %self.session_id,
                    thread_id = %self.thread_id,
                    turn_id = turn_id,
                    latency_ms = latency_ms,
                    error_kind = ?err.kind,
                    error = %err.message,
                    "chat_turn_anomaly"
                );

                observability_metrics::increment_chat_task_failures_total(
                    "backend_infer",
                    &format!("{:?}", err.kind),
                );
                observability_metrics::increment_chat_thread_failures_total(
                    &self.session_id,
                    &self.thread_id,
                    &format!("{:?}", err.kind),
                );
                observability_metrics::set_chat_thread_last_turn_latency_ms(
                    &self.session_id,
                    &self.thread_id,
                    commit.last_turn_latency_ms.unwrap_or(latency_ms),
                );

                Err(err)
            }
        }
    }

    pub async fn turn_stream(&self, _request: ChatTurnRequest) -> Result<ChatEventStream, GatewayError> {
        Err(
            GatewayError::new(
                GatewayErrorKind::UnsupportedCapability,
                "chat thread streaming is not implemented yet; use turn_once",
            )
            .with_retryable(false),
        )
    }
}

fn assistant_message_from_response(response: &crate::ai_gateway::types_chat::ChatResponse) -> BelunaMessage {
    let tool_calls = response
        .tool_calls
        .iter()
        .map(|call| BelunaMessageToolCall {
            id: call.id.clone(),
            name: call.name.clone(),
            arguments_json: call.arguments_json.clone(),
        })
        .collect::<Vec<_>>();

    BelunaMessage {
        role: crate::ai_gateway::types_chat::BelunaRole::Assistant,
        parts: vec![BelunaContentPart::Text {
            text: response.output_text.clone(),
        }],
        tool_call_id: None,
        tool_name: None,
        tool_calls,
    }
}

fn emit_turn_summary(
    session_id: &str,
    thread_id: &str,
    turn_id: u64,
    response: &crate::ai_gateway::types_chat::ChatResponse,
    commit: &TurnCommitOutcome,
    latency_ms: u64,
    outcome: &str,
) {
    let backend_id = response
        .backend_metadata
        .get("backend_id")
        .and_then(|value| value.as_str())
        .unwrap_or("-");
    let model = response
        .backend_metadata
        .get("model")
        .and_then(|value| value.as_str())
        .unwrap_or("-");
    let usage_in_tokens = response.usage.as_ref().and_then(|item| item.input_tokens);
    let usage_out_tokens = response.usage.as_ref().and_then(|item| item.output_tokens);

    tracing::info!(
        target: "ai_gateway",
        event = "chat_turn_summary",
        session_id = %session_id,
        thread_id = %thread_id,
        turn_id = turn_id,
        backend_id = backend_id,
        model = model,
        attempts = 1,
        latency_ms = latency_ms,
        tool_rounds = response.tool_calls.len(),
        usage_in_tokens = ?usage_in_tokens,
        usage_out_tokens = ?usage_out_tokens,
        finish_reason = ?response.finish_reason,
        outcome = outcome,
        thread_turns_total = commit.turns_total,
        thread_tool_calls_total = commit.tool_calls_total,
        thread_failures_total = commit.failures_total,
        "chat_turn_summary"
    );
}

fn record_turn_metrics(
    session_id: &str,
    thread_id: &str,
    response: &crate::ai_gateway::types_chat::ChatResponse,
    commit: &TurnCommitOutcome,
    latency_ms: u64,
) {
    let backend = response
        .backend_metadata
        .get("backend_id")
        .and_then(|value| value.as_str())
        .unwrap_or("-");
    let model = response
        .backend_metadata
        .get("model")
        .and_then(|value| value.as_str())
        .unwrap_or("-");

    observability_metrics::record_chat_task_latency_ms("backend_infer", backend, model, latency_ms);
    observability_metrics::increment_chat_thread_turns_total(session_id, thread_id);
    observability_metrics::add_chat_thread_tool_calls_total(
        session_id,
        thread_id,
        "model_tool_calls",
        response.tool_calls.len() as u64,
    );
    observability_metrics::add_chat_thread_tokens_in_total(
        session_id,
        thread_id,
        response
            .usage
            .as_ref()
            .and_then(|usage| usage.input_tokens)
            .unwrap_or(0),
    );
    observability_metrics::add_chat_thread_tokens_out_total(
        session_id,
        thread_id,
        response
            .usage
            .as_ref()
            .and_then(|usage| usage.output_tokens)
            .unwrap_or(0),
    );
    observability_metrics::set_chat_thread_last_turn_latency_ms(session_id, thread_id, latency_ms);

    let _ = commit.tokens_in_total;
    let _ = commit.tokens_out_total;
}
