use std::{
    collections::{BTreeMap, HashSet},
    sync::Arc,
    time::Instant,
};

use futures_util::{StreamExt, stream};

use crate::{
    ai_gateway::{
        credentials::CredentialProvider,
        error::{GatewayError, GatewayErrorKind},
        types::AIGatewayConfig,
    },
    observability::metrics as observability_metrics,
};

use super::{
    dispatcher::{ChatDispatcher, DispatchResult},
    executor::{ToolExecutionRequest, ToolExecutionResult, ToolExecutor},
    store::{PreparedTurn, ThreadStore, TurnCommitOutcome},
    tool::{ChatToolDefinition, ToolOverride, resolve_tools},
    types::{
        ChatEventStream, ChatMessage, ChatRole, ContentPart, FinishReason, OutputMode,
        ThreadMessageMutationOutcome, ThreadMessageMutationRequest, ToolCallResult, TurnLimits,
        TurnPayload, TurnResponse,
    },
};

// ---------------------------------------------------------------------------
// Chat — definition aggregate
// ---------------------------------------------------------------------------

/// A Chat defines the *what* of a conversation: tools, system prompt,
/// default route, output mode. It is an immutable definition; runtime state
/// lives in [`Thread`] instances derived from it.
#[derive(Clone)]
pub struct Chat {
    chat_id: String,
    tools: Vec<ChatToolDefinition>,
    system_prompt: Option<String>,
    default_route: Option<String>,
    default_output_mode: OutputMode,
    default_limits: TurnLimits,
    default_turn_timeout_ms: u64,
    default_tool_call_mode: ToolCallContinuationMode,
    default_max_tool_rounds: u32,
    enable_thinking: bool,
    dispatcher: Arc<ChatDispatcher>,
    store: Arc<ThreadStore>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolCallContinuationMode {
    ImmediateNewTurn,
    NextTurn,
}

impl Default for ToolCallContinuationMode {
    fn default() -> Self {
        Self::ImmediateNewTurn
    }
}

/// Options for creating a new [`Chat`].
#[derive(Default)]
pub struct ChatOptions {
    pub chat_id: Option<String>,
    pub tools: Vec<ChatToolDefinition>,
    pub system_prompt: Option<String>,
    pub default_route: Option<String>,
    pub default_output_mode: Option<OutputMode>,
    pub default_limits: Option<TurnLimits>,
    pub default_tool_call_mode: Option<ToolCallContinuationMode>,
    pub enable_thinking: bool,
}

/// Builder that owns the shared infrastructure for constructing `Chat` instances.
#[derive(Clone)]
pub struct ChatFactory {
    dispatcher: Arc<ChatDispatcher>,
    store: Arc<ThreadStore>,
    default_route: Option<String>,
    default_turn_timeout_ms: u64,
    default_max_tool_rounds: u32,
}

impl ChatFactory {
    pub fn new(
        config: &AIGatewayConfig,
        credential_provider: Arc<dyn CredentialProvider>,
    ) -> Result<Self, GatewayError> {
        let dispatcher = Arc::new(ChatDispatcher::new(config, credential_provider)?);
        let store = Arc::new(ThreadStore::new(
            config.chat.default_session_ttl_seconds,
            config.chat.default_max_turn_context_messages,
        ));

        Ok(Self {
            dispatcher,
            store,
            default_route: config.chat.default_route.clone(),
            default_turn_timeout_ms: config.chat.default_turn_timeout_ms,
            default_max_tool_rounds: config.chat.default_max_tool_rounds,
        })
    }

    pub async fn create(&self, opts: ChatOptions) -> Chat {
        static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        let chat_id = opts.chat_id.unwrap_or_else(|| {
            format!(
                "chat-{}",
                SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            )
        });

        // Register in store so threads can be opened immediately.
        self.store.register_chat(&chat_id).await;

        Chat {
            chat_id,
            tools: opts.tools,
            system_prompt: opts.system_prompt,
            default_route: opts.default_route.or_else(|| self.default_route.clone()),
            default_output_mode: opts.default_output_mode.unwrap_or(OutputMode::Text),
            default_limits: opts.default_limits.unwrap_or_default(),
            default_turn_timeout_ms: self.default_turn_timeout_ms,
            default_tool_call_mode: opts
                .default_tool_call_mode
                .unwrap_or(ToolCallContinuationMode::default()),
            default_max_tool_rounds: self.default_max_tool_rounds.max(1),
            enable_thinking: opts.enable_thinking,
            dispatcher: Arc::clone(&self.dispatcher),
            store: Arc::clone(&self.store),
        }
    }
}

impl Chat {
    pub fn chat_id(&self) -> &str {
        &self.chat_id
    }

    pub fn tools(&self) -> &[ChatToolDefinition] {
        &self.tools
    }

    pub async fn open_thread(&self, opts: ThreadOptions) -> Result<Thread, GatewayError> {
        let thread_id = self
            .store
            .open_thread(&self.chat_id, opts.thread_id, opts.seed_messages)
            .await?;

        tracing::info!(
            target: "ai_gateway",
            event = "chat_thread_lifecycle",
            action = "opened",
            chat_id = %self.chat_id,
            thread_id = %thread_id,
            "chat_thread_lifecycle"
        );

        Ok(Thread {
            chat: self.clone(),
            thread_id,
            tool_overrides: opts.tool_overrides,
            tool_call_mode: opts.tool_call_mode,
        })
    }

    pub async fn close(&self) {
        self.store.remove_chat(&self.chat_id).await;
        tracing::info!(
            target: "ai_gateway",
            event = "chat_lifecycle",
            action = "closed",
            chat_id = %self.chat_id,
            "chat_lifecycle"
        );
    }
}

// ---------------------------------------------------------------------------
// Thread — runtime conversation derived from a Chat
// ---------------------------------------------------------------------------

/// Options for opening a new [`Thread`].
#[derive(Default)]
pub struct ThreadOptions {
    pub thread_id: Option<String>,
    pub seed_messages: Vec<ChatMessage>,
    pub tool_overrides: Vec<ToolOverride>,
    pub tool_call_mode: Option<ToolCallContinuationMode>,
}

/// A Thread is a runtime conversation bound to a parent [`Chat`].
/// It holds the growing message history and can execute turns.
#[derive(Clone)]
pub struct Thread {
    chat: Chat,
    thread_id: String,
    tool_overrides: Vec<ToolOverride>,
    tool_call_mode: Option<ToolCallContinuationMode>,
}

struct TurnExecutionOutcome {
    response: TurnResponse,
    messages_to_commit: Vec<ChatMessage>,
    consumed_pending_tool_messages: bool,
    new_pending_tool_messages: Option<Vec<ChatMessage>>,
    tool_call_count: usize,
}

impl Thread {
    pub fn thread_id(&self) -> &str {
        &self.thread_id
    }

    pub fn chat_id(&self) -> &str {
        self.chat.chat_id()
    }

    /// Execute a non-streaming turn.
    pub async fn complete(&self, input: TurnInput) -> Result<TurnOutput, GatewayError> {
        if input.messages.is_empty() {
            return Err(GatewayError::new(
                GatewayErrorKind::InvalidRequest,
                "turn requires at least one input message",
            )
            .with_retryable(false));
        }

        let started_at = Instant::now();
        let prepared = self
            .chat
            .store
            .prepare_turn(
                &self.chat.chat_id,
                &self.thread_id,
                &input.messages,
                self.chat.system_prompt.as_deref(),
            )
            .await?;
        let turn_id = prepared.turn_id;
        let execution_result = self.execute_turn_with_prepared(input, prepared).await;
        let latency_ms = started_at.elapsed().as_millis() as u64;

        match execution_result {
            Ok(outcome) => {
                let commit = self
                    .chat
                    .store
                    .commit_turn_success(
                        &self.chat.chat_id,
                        &self.thread_id,
                        outcome.consumed_pending_tool_messages,
                        outcome.messages_to_commit,
                        outcome.new_pending_tool_messages,
                        outcome.response.usage.clone(),
                        outcome.tool_call_count,
                        latency_ms,
                    )
                    .await?;

                emit_turn_summary(
                    &self.chat.chat_id,
                    &self.thread_id,
                    turn_id,
                    &outcome.response,
                    &commit,
                    outcome.tool_call_count,
                    latency_ms,
                    "ok",
                );
                record_turn_metrics(
                    &self.chat.chat_id,
                    &self.thread_id,
                    &outcome.response,
                    outcome.tool_call_count,
                    &commit,
                    latency_ms,
                );

                Ok(TurnOutput {
                    chat_id: self.chat.chat_id.clone(),
                    thread_id: self.thread_id.clone(),
                    turn_id,
                    response: outcome.response,
                })
            }
            Err(err) => {
                let commit = self
                    .chat
                    .store
                    .commit_turn_failure(&self.chat.chat_id, &self.thread_id, latency_ms)
                    .await?;

                tracing::info!(
                    target: "ai_gateway",
                    event = "chat_turn_anomaly",
                    chat_id = %self.chat.chat_id,
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
                    &self.chat.chat_id,
                    &self.thread_id,
                    &format!("{:?}", err.kind),
                );
                observability_metrics::set_chat_thread_last_turn_latency_ms(
                    &self.chat.chat_id,
                    &self.thread_id,
                    commit.last_turn_latency_ms.unwrap_or(latency_ms),
                );

                Err(err)
            }
        }
    }

    async fn execute_turn_with_prepared(
        &self,
        input: TurnInput,
        prepared: PreparedTurn,
    ) -> Result<TurnExecutionOutcome, GatewayError> {
        let turn_id = prepared.turn_id;
        let effective_tools = resolve_tools(
            &self.chat.tools,
            if input.tool_overrides.is_empty() {
                &self.tool_overrides
            } else {
                &input.tool_overrides
            },
        );
        let route = input
            .route_override
            .as_deref()
            .or(self.chat.default_route.as_deref());
        let mut limits = input
            .limits
            .unwrap_or_else(|| self.chat.default_limits.clone());
        if limits.max_request_time_ms.is_none() {
            limits.max_request_time_ms = Some(self.chat.default_turn_timeout_ms);
        }
        let output_mode = input
            .output_mode
            .unwrap_or_else(|| self.chat.default_output_mode.clone());
        let enable_thinking = input.enable_thinking.unwrap_or(self.chat.enable_thinking);
        let continuation_mode = input
            .tool_call_mode_override
            .or(self.tool_call_mode)
            .unwrap_or(self.chat.default_tool_call_mode);
        let tool_executor = input.tool_executor;

        let mut metadata = input.metadata;
        metadata
            .entry("chat_id".to_string())
            .or_insert_with(|| self.chat.chat_id.clone());
        metadata
            .entry("thread_id".to_string())
            .or_insert_with(|| self.thread_id.clone());
        metadata
            .entry("turn_id".to_string())
            .or_insert_with(|| turn_id.to_string());

        let mut dispatch_messages = if let Some(ref sys) = prepared.system_prompt {
            let mut msgs = Vec::with_capacity(prepared.messages.len() + 1);
            msgs.push(ChatMessage {
                role: ChatRole::System,
                parts: vec![ContentPart::Text { text: sys.clone() }],
                tool_call_id: None,
                tool_name: None,
                tool_calls: vec![],
            });
            msgs.extend(prepared.messages.iter().cloned());
            msgs
        } else {
            prepared.messages.as_ref().clone()
        };
        validate_tool_message_chain(&dispatch_messages)?;

        let mut messages_to_commit = input.messages;
        let mut new_pending_tool_messages: Option<Vec<ChatMessage>> = None;
        let mut tool_call_count = 0_usize;
        let mut tool_rounds = 0_u32;

        loop {
            let payload = TurnPayload {
                messages: Arc::new(dispatch_messages.clone()),
                tools: effective_tools.clone(),
                output_mode: output_mode.clone(),
                limits: limits.clone(),
                enable_thinking,
                metadata: metadata.clone(),
            };
            let DispatchResult { mut response, .. } =
                self.chat.dispatcher.complete(&payload, route).await?;

            let assistant_message = assistant_message_from_response(&response);
            messages_to_commit.push(assistant_message.clone());
            tool_call_count = tool_call_count.saturating_add(response.tool_calls.len());

            let Some(executor) = tool_executor.as_ref() else {
                response.pending_tool_call_continuation = false;
                return Ok(TurnExecutionOutcome {
                    response,
                    messages_to_commit,
                    consumed_pending_tool_messages: prepared.consumed_pending_tool_messages,
                    new_pending_tool_messages,
                    tool_call_count,
                });
            };

            if response.tool_calls.is_empty() {
                response.pending_tool_call_continuation = false;
                return Ok(TurnExecutionOutcome {
                    response,
                    messages_to_commit,
                    consumed_pending_tool_messages: prepared.consumed_pending_tool_messages,
                    new_pending_tool_messages,
                    tool_call_count,
                });
            }

            ensure_unique_tool_call_names(&response.tool_calls)?;
            tool_rounds = tool_rounds.saturating_add(1);
            if tool_rounds > self.chat.default_max_tool_rounds {
                return Err(GatewayError::new(
                    GatewayErrorKind::InvalidRequest,
                    format!(
                        "tool call rounds {} exceeded limit {}",
                        tool_rounds, self.chat.default_max_tool_rounds
                    ),
                )
                .with_retryable(false));
            }

            let batch = execute_tool_calls_parallel(
                executor,
                &self.chat.chat_id,
                &self.thread_id,
                turn_id,
                &response.tool_calls,
            )
            .await;

            match continuation_mode {
                ToolCallContinuationMode::ImmediateNewTurn => {
                    if batch.reset_messages_applied {
                        return Err(GatewayError::new(
                            GatewayErrorKind::InvalidRequest,
                            "tool executor requested context reset in immediate_new_turn mode",
                        )
                        .with_retryable(false));
                    }
                    messages_to_commit.extend(batch.tool_messages.iter().cloned());
                    dispatch_messages.push(assistant_message);
                    dispatch_messages.extend(batch.tool_messages);
                    validate_tool_message_chain(&dispatch_messages)?;
                }
                ToolCallContinuationMode::NextTurn => {
                    new_pending_tool_messages = Some(if batch.reset_messages_applied {
                        Vec::new()
                    } else {
                        batch.tool_messages
                    });
                    response.output_text.clear();
                    response.tool_calls.clear();
                    response.finish_reason = FinishReason::ToolCalls;
                    response.pending_tool_call_continuation = true;
                    return Ok(TurnExecutionOutcome {
                        response,
                        messages_to_commit,
                        consumed_pending_tool_messages: prepared.consumed_pending_tool_messages,
                        new_pending_tool_messages,
                        tool_call_count,
                    });
                }
            }
        }
    }

    /// Execute a streaming turn.
    pub async fn stream(&self, _input: TurnInput) -> Result<ChatEventStream, GatewayError> {
        Err(GatewayError::new(
            GatewayErrorKind::UnsupportedCapability,
            "thread streaming is not yet implemented; use complete()",
        )
        .with_retryable(false))
    }

    pub async fn mutate_messages_atomically(
        &self,
        request: ThreadMessageMutationRequest,
    ) -> Result<ThreadMessageMutationOutcome, GatewayError> {
        let outcome = self
            .chat
            .store
            .mutate_thread_messages_atomically(&self.chat.chat_id, &self.thread_id, request)
            .await?;

        tracing::info!(
            target: "ai_gateway",
            event = "chat_thread_message_mutation",
            chat_id = %self.chat.chat_id,
            thread_id = %self.thread_id,
            removed_messages = outcome.removed_messages,
            remaining_messages = outcome.remaining_messages,
            effective_system_prompt_changed = outcome.effective_system_prompt_changed,
            "chat_thread_message_mutation"
        );
        Ok(outcome)
    }
}

// ---------------------------------------------------------------------------
// Turn input / output
// ---------------------------------------------------------------------------

pub struct TurnInput {
    pub messages: Vec<ChatMessage>,
    pub route_override: Option<String>,
    pub tool_overrides: Vec<ToolOverride>,
    pub output_mode: Option<OutputMode>,
    pub limits: Option<TurnLimits>,
    pub enable_thinking: Option<bool>,
    pub tool_executor: Option<Arc<dyn ToolExecutor>>,
    pub tool_call_mode_override: Option<ToolCallContinuationMode>,
    pub metadata: BTreeMap<String, String>,
}

impl Default for TurnInput {
    fn default() -> Self {
        Self {
            messages: Vec::new(),
            route_override: None,
            tool_overrides: Vec::new(),
            output_mode: None,
            limits: None,
            enable_thinking: None,
            tool_executor: None,
            tool_call_mode_override: None,
            metadata: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TurnOutput {
    pub chat_id: String,
    pub thread_id: String,
    pub turn_id: u64,
    pub response: TurnResponse,
}

struct ToolBatchExecution {
    tool_messages: Vec<ChatMessage>,
    reset_messages_applied: bool,
}

async fn execute_tool_calls_parallel(
    executor: &Arc<dyn ToolExecutor>,
    chat_id: &str,
    thread_id: &str,
    turn_id: u64,
    calls: &[ToolCallResult],
) -> ToolBatchExecution {
    if calls.is_empty() {
        return ToolBatchExecution {
            tool_messages: Vec::new(),
            reset_messages_applied: false,
        };
    }

    let tasks = calls.iter().cloned().enumerate().map(|(idx, call)| {
        let executor = Arc::clone(executor);
        let chat_id = chat_id.to_string();
        let thread_id = thread_id.to_string();
        async move {
            let request = ToolExecutionRequest {
                chat_id,
                thread_id,
                turn_id,
                call: call.clone(),
            };
            let result = executor.execute_call(request).await;
            (idx, call, result)
        }
    });

    let mut completed = stream::iter(tasks)
        .buffer_unordered(calls.len())
        .collect::<Vec<_>>()
        .await;
    completed.sort_by_key(|(idx, _, _)| *idx);

    let mut reset_messages_applied = false;
    let mut tool_messages = Vec::with_capacity(completed.len());
    for (_, call, result) in completed {
        let ToolExecutionResult {
            payload,
            reset_messages_applied: call_reset_messages_applied,
        } = match result {
            Ok(item) => item,
            Err(err) => ToolExecutionResult {
                payload: serde_json::json!({
                    "ok": false,
                    "tool": call.name,
                    "error": err.message,
                }),
                reset_messages_applied: false,
            },
        };
        if call_reset_messages_applied {
            reset_messages_applied = true;
        }
        tool_messages.push(ChatMessage {
            role: ChatRole::Tool,
            parts: vec![ContentPart::Json { value: payload }],
            tool_call_id: Some(call.id),
            tool_name: Some(call.name),
            tool_calls: vec![],
        });
    }

    ToolBatchExecution {
        tool_messages,
        reset_messages_applied,
    }
}

fn ensure_unique_tool_call_names(calls: &[ToolCallResult]) -> Result<(), GatewayError> {
    let mut seen = HashSet::with_capacity(calls.len());
    for call in calls {
        if !seen.insert(call.name.as_str()) {
            return Err(GatewayError::new(
                GatewayErrorKind::InvalidRequest,
                format!(
                    "assistant tool_calls in one batch must use unique tool names; duplicate='{}'",
                    call.name
                ),
            )
            .with_retryable(false));
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn assistant_message_from_response(response: &TurnResponse) -> ChatMessage {
    let tool_calls = response
        .tool_calls
        .iter()
        .map(|call| super::types::MessageToolCall {
            id: call.id.clone(),
            name: call.name.clone(),
            arguments_json: call.arguments_json.clone(),
        })
        .collect::<Vec<_>>();

    ChatMessage {
        role: ChatRole::Assistant,
        parts: vec![ContentPart::Text {
            text: response.output_text.clone(),
        }],
        tool_call_id: None,
        tool_name: None,
        tool_calls,
    }
}

fn emit_turn_summary(
    chat_id: &str,
    thread_id: &str,
    turn_id: u64,
    response: &TurnResponse,
    commit: &TurnCommitOutcome,
    tool_call_count: usize,
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
        chat_id = %chat_id,
        thread_id = %thread_id,
        turn_id = turn_id,
        backend_id = backend_id,
        model = model,
        latency_ms = latency_ms,
        tool_rounds = tool_call_count,
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
    chat_id: &str,
    thread_id: &str,
    response: &TurnResponse,
    tool_call_count: usize,
    _commit: &TurnCommitOutcome,
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
    observability_metrics::increment_chat_thread_turns_total(chat_id, thread_id);
    observability_metrics::add_chat_thread_tool_calls_total(
        chat_id,
        thread_id,
        "model_tool_calls",
        tool_call_count as u64,
    );
    observability_metrics::add_chat_thread_tokens_in_total(
        chat_id,
        thread_id,
        response
            .usage
            .as_ref()
            .and_then(|usage| usage.input_tokens)
            .unwrap_or(0),
    );
    observability_metrics::add_chat_thread_tokens_out_total(
        chat_id,
        thread_id,
        response
            .usage
            .as_ref()
            .and_then(|usage| usage.output_tokens)
            .unwrap_or(0),
    );
    observability_metrics::set_chat_thread_last_turn_latency_ms(chat_id, thread_id, latency_ms);
}

fn validate_tool_message_chain(messages: &[ChatMessage]) -> Result<(), GatewayError> {
    let mut active_tool_call_ids: Option<HashSet<&str>> = None;
    for (index, message) in messages.iter().enumerate() {
        match message.role {
            ChatRole::Assistant => {
                if message.tool_calls.is_empty() {
                    active_tool_call_ids = None;
                } else {
                    let ids = message
                        .tool_calls
                        .iter()
                        .map(|call| call.id.as_str())
                        .collect::<HashSet<_>>();
                    active_tool_call_ids = Some(ids);
                }
            }
            ChatRole::Tool => {
                let tool_call_id = message.tool_call_id.as_deref().ok_or_else(|| {
                    GatewayError::new(
                        GatewayErrorKind::InvalidRequest,
                        format!(
                            "messages with role \"tool\" must be a response to a preceeding message with \"tool_calls\" (index={}, missing tool_call_id)",
                            index
                        ),
                    )
                    .with_retryable(false)
                })?;
                let Some(active) = active_tool_call_ids.as_mut() else {
                    return Err(
                        GatewayError::new(
                            GatewayErrorKind::InvalidRequest,
                            format!(
                                "messages with role \"tool\" must be a response to a preceeding message with \"tool_calls\" (index={}, tool_call_id={})",
                                index, tool_call_id
                            ),
                        )
                        .with_retryable(false),
                    );
                };
                if !active.contains(tool_call_id) {
                    return Err(
                        GatewayError::new(
                            GatewayErrorKind::InvalidRequest,
                            format!(
                                "messages with role \"tool\" must be a response to a preceeding message with \"tool_calls\" (index={}, tool_call_id={})",
                                index, tool_call_id
                            ),
                        )
                        .with_retryable(false),
                    );
                }
                active.remove(tool_call_id);
            }
            ChatRole::System | ChatRole::User => {
                active_tool_call_ids = None;
            }
        }
    }
    Ok(())
}
