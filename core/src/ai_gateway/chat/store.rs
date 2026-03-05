use std::{
    collections::{HashMap, HashSet},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};

use tokio::sync::RwLock;

use crate::ai_gateway::error::{GatewayError, GatewayErrorKind};

use super::types::{
    ChatMessage, ChatRole, MessageBoundarySelector, SystemPromptUpdate,
    ThreadMessageMutationOutcome, ThreadMessageMutationRequest, UsageStats,
};

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct ThreadState {
    pub thread_id: String,
    pub next_turn_id: u64,
    pub message_count: usize,
    pub turns_total: u64,
    pub tool_calls_total: u64,
    pub failures_total: u64,
    pub last_turn_latency_ms: Option<u64>,
}

pub(crate) struct PreparedTurn {
    pub turn_id: u64,
    pub messages: Arc<Vec<ChatMessage>>,
    pub system_prompt: Option<String>,
    pub consumed_pending_tool_messages: bool,
}

pub(crate) struct TurnCommitOutcome {
    pub turns_total: u64,
    pub tool_calls_total: u64,
    pub tokens_in_total: u64,
    pub tokens_out_total: u64,
    pub failures_total: u64,
    pub last_turn_latency_ms: Option<u64>,
}

// ---------------------------------------------------------------------------
// Internal state
// ---------------------------------------------------------------------------

struct ThreadData {
    messages: Vec<ChatMessage>,
    pending_tool_messages: Vec<ChatMessage>,
    system_prompt_mode: ThreadSystemPromptMode,
    next_turn_id: u64,
    metrics: ThreadMetrics,
}

#[derive(Default)]
enum ThreadSystemPromptMode {
    #[default]
    UseChatDefault,
    Override(String),
    Cleared,
}

#[derive(Default)]
struct ThreadMetrics {
    turns_total: u64,
    tool_calls_total: u64,
    tokens_in_total: u64,
    tokens_out_total: u64,
    failures_total: u64,
    last_turn_latency_ms: Option<u64>,
}

struct ChatData {
    threads: HashMap<String, ThreadData>,
    created_at: Instant,
    expires_at: Instant,
}

// ---------------------------------------------------------------------------
// ThreadStore
// ---------------------------------------------------------------------------

pub(crate) struct ThreadStore {
    chats: RwLock<HashMap<String, ChatData>>,
    thread_seq: AtomicU64,
    ttl: Duration,
    max_turn_context_messages: usize,
}

impl ThreadStore {
    pub(crate) fn new(ttl_seconds: u64, max_turn_context_messages: usize) -> Self {
        Self {
            chats: RwLock::new(HashMap::new()),
            thread_seq: AtomicU64::new(1),
            ttl: Duration::from_secs(ttl_seconds.max(1)),
            max_turn_context_messages,
        }
    }

    /// Register a chat. Idempotent — re-registering refreshes expiry.
    pub(crate) async fn register_chat(&self, chat_id: &str) {
        let mut guard = self.chats.write().await;
        self.sweep_expired(&mut guard);
        let now = Instant::now();
        guard
            .entry(chat_id.to_string())
            .and_modify(|c| c.expires_at = now + self.ttl)
            .or_insert_with(|| ChatData {
                threads: HashMap::new(),
                created_at: now,
                expires_at: now + self.ttl,
            });
    }

    pub(crate) async fn remove_chat(&self, chat_id: &str) {
        let mut guard = self.chats.write().await;
        guard.remove(chat_id);
    }

    pub(crate) async fn open_thread(
        &self,
        chat_id: &str,
        thread_id: Option<String>,
        seed_messages: Vec<ChatMessage>,
    ) -> Result<String, GatewayError> {
        let mut guard = self.chats.write().await;
        self.sweep_expired(&mut guard);

        let chat = guard.get_mut(chat_id).ok_or_else(|| {
            GatewayError::new(
                GatewayErrorKind::InvalidRequest,
                format!("chat '{}' not found", chat_id),
            )
            .with_retryable(false)
        })?;
        chat.expires_at = Instant::now() + self.ttl;

        let thread_id = thread_id.unwrap_or_else(|| self.next_thread_id());
        let entry = chat
            .threads
            .entry(thread_id.clone())
            .or_insert_with(|| ThreadData {
                messages: Vec::new(),
                pending_tool_messages: Vec::new(),
                system_prompt_mode: ThreadSystemPromptMode::UseChatDefault,
                next_turn_id: 0,
                metrics: ThreadMetrics::default(),
            });
        if !seed_messages.is_empty() {
            entry.messages.extend(seed_messages);
            self.trim_context(entry);
        }

        Ok(thread_id)
    }

    pub(crate) async fn prepare_turn(
        &self,
        chat_id: &str,
        thread_id: &str,
        input_messages: &[ChatMessage],
        chat_system_prompt: Option<&str>,
    ) -> Result<PreparedTurn, GatewayError> {
        let mut guard = self.chats.write().await;
        self.sweep_expired(&mut guard);

        let chat = guard.get_mut(chat_id).ok_or_else(|| {
            GatewayError::new(
                GatewayErrorKind::InvalidRequest,
                format!("chat '{}' not found", chat_id),
            )
            .with_retryable(false)
        })?;
        chat.expires_at = Instant::now() + self.ttl;

        let thread = chat.threads.get(thread_id).ok_or_else(|| {
            GatewayError::new(
                GatewayErrorKind::InvalidRequest,
                format!("thread '{}' not found", thread_id),
            )
            .with_retryable(false)
        })?;

        let mut messages = thread.messages.clone();
        let consumed_pending_tool_messages = !thread.pending_tool_messages.is_empty();
        if consumed_pending_tool_messages {
            messages.extend(thread.pending_tool_messages.iter().cloned());
        }
        messages.extend_from_slice(input_messages);
        let system_prompt = thread.system_prompt_mode.resolve(chat_system_prompt);
        Ok(PreparedTurn {
            turn_id: thread.next_turn_id,
            messages: Arc::new(messages),
            system_prompt,
            consumed_pending_tool_messages,
        })
    }

    pub(crate) async fn commit_turn_success(
        &self,
        chat_id: &str,
        thread_id: &str,
        consumed_pending_tool_messages: bool,
        mut new_messages: Vec<ChatMessage>,
        new_pending_tool_messages: Option<Vec<ChatMessage>>,
        usage: Option<UsageStats>,
        tool_call_count: usize,
        latency_ms: u64,
    ) -> Result<TurnCommitOutcome, GatewayError> {
        let mut guard = self.chats.write().await;
        let chat = guard.get_mut(chat_id).ok_or_else(|| {
            GatewayError::new(
                GatewayErrorKind::InvalidRequest,
                format!("chat '{}' not found", chat_id),
            )
            .with_retryable(false)
        })?;
        chat.expires_at = Instant::now() + self.ttl;

        let thread = chat.threads.get_mut(thread_id).ok_or_else(|| {
            GatewayError::new(
                GatewayErrorKind::InvalidRequest,
                format!("thread '{}' not found", thread_id),
            )
            .with_retryable(false)
        })?;

        if consumed_pending_tool_messages {
            thread
                .messages
                .extend(thread.pending_tool_messages.drain(..));
        }
        thread.messages.append(&mut new_messages);
        if let Some(mut pending) = new_pending_tool_messages {
            thread.pending_tool_messages.clear();
            thread.pending_tool_messages.append(&mut pending);
        }
        self.trim_context(thread);

        thread.next_turn_id += 1;
        thread.metrics.turns_total += 1;
        thread.metrics.tool_calls_total = thread
            .metrics
            .tool_calls_total
            .saturating_add(tool_call_count as u64);
        thread.metrics.last_turn_latency_ms = Some(latency_ms);

        let (tokens_in, tokens_out) = usage
            .map(|u| (u.input_tokens.unwrap_or(0), u.output_tokens.unwrap_or(0)))
            .unwrap_or((0, 0));
        thread.metrics.tokens_in_total = thread.metrics.tokens_in_total.saturating_add(tokens_in);
        thread.metrics.tokens_out_total =
            thread.metrics.tokens_out_total.saturating_add(tokens_out);

        Ok(TurnCommitOutcome {
            turns_total: thread.metrics.turns_total,
            tool_calls_total: thread.metrics.tool_calls_total,
            tokens_in_total: thread.metrics.tokens_in_total,
            tokens_out_total: thread.metrics.tokens_out_total,
            failures_total: thread.metrics.failures_total,
            last_turn_latency_ms: thread.metrics.last_turn_latency_ms,
        })
    }

    pub(crate) async fn commit_turn_failure(
        &self,
        chat_id: &str,
        thread_id: &str,
        latency_ms: u64,
    ) -> Result<TurnCommitOutcome, GatewayError> {
        let mut guard = self.chats.write().await;
        let chat = guard.get_mut(chat_id).ok_or_else(|| {
            GatewayError::new(
                GatewayErrorKind::InvalidRequest,
                format!("chat '{}' not found", chat_id),
            )
            .with_retryable(false)
        })?;
        let thread = chat.threads.get_mut(thread_id).ok_or_else(|| {
            GatewayError::new(
                GatewayErrorKind::InvalidRequest,
                format!("thread '{}' not found", thread_id),
            )
            .with_retryable(false)
        })?;

        thread.metrics.failures_total = thread.metrics.failures_total.saturating_add(1);
        thread.metrics.last_turn_latency_ms = Some(latency_ms);

        Ok(TurnCommitOutcome {
            turns_total: thread.metrics.turns_total,
            tool_calls_total: thread.metrics.tool_calls_total,
            tokens_in_total: thread.metrics.tokens_in_total,
            tokens_out_total: thread.metrics.tokens_out_total,
            failures_total: thread.metrics.failures_total,
            last_turn_latency_ms: thread.metrics.last_turn_latency_ms,
        })
    }

    pub(crate) async fn thread_state(
        &self,
        chat_id: &str,
        thread_id: &str,
    ) -> Result<ThreadState, GatewayError> {
        let guard = self.chats.read().await;
        let chat = guard.get(chat_id).ok_or_else(|| {
            GatewayError::new(
                GatewayErrorKind::InvalidRequest,
                format!("chat '{}' not found", chat_id),
            )
            .with_retryable(false)
        })?;
        let thread = chat.threads.get(thread_id).ok_or_else(|| {
            GatewayError::new(
                GatewayErrorKind::InvalidRequest,
                format!("thread '{}' not found", thread_id),
            )
            .with_retryable(false)
        })?;

        Ok(ThreadState {
            thread_id: thread_id.to_string(),
            next_turn_id: thread.next_turn_id,
            message_count: thread.messages.len(),
            turns_total: thread.metrics.turns_total,
            tool_calls_total: thread.metrics.tool_calls_total,
            failures_total: thread.metrics.failures_total,
            last_turn_latency_ms: thread.metrics.last_turn_latency_ms,
        })
    }

    pub(crate) async fn mutate_thread_messages_atomically(
        &self,
        chat_id: &str,
        thread_id: &str,
        request: ThreadMessageMutationRequest,
    ) -> Result<ThreadMessageMutationOutcome, GatewayError> {
        let mut guard = self.chats.write().await;
        let chat = guard.get_mut(chat_id).ok_or_else(|| {
            GatewayError::new(
                GatewayErrorKind::InvalidRequest,
                format!("chat '{}' not found", chat_id),
            )
            .with_retryable(false)
        })?;
        chat.expires_at = Instant::now() + self.ttl;

        let thread = chat.threads.get_mut(thread_id).ok_or_else(|| {
            GatewayError::new(
                GatewayErrorKind::InvalidRequest,
                format!("thread '{}' not found", thread_id),
            )
            .with_retryable(false)
        })?;

        let mut removed_messages = 0usize;
        if let Some(range) = request.trim_message_range.as_ref() {
            let start_idx = resolve_message_boundary(&thread.messages, &range.start);
            let end_idx = resolve_message_boundary(&thread.messages, &range.end);
            match (start_idx, end_idx) {
                (Some(start_idx), Some(end_idx)) => {
                    if start_idx > end_idx {
                        return Err(GatewayError::new(
                            GatewayErrorKind::InvalidRequest,
                            format!(
                                "invalid message range: start={} is after end={} for thread '{}'",
                                start_idx, end_idx, thread_id
                            ),
                        )
                        .with_retryable(false));
                    }

                    removed_messages = end_idx - start_idx + 1;
                    thread.messages.drain(start_idx..=end_idx);
                    let dropped_orphans = drop_leading_orphan_tool_messages(&mut thread.messages);
                    if dropped_orphans > 0 {
                        removed_messages = removed_messages.saturating_add(dropped_orphans);
                        tracing::warn!(
                            target: "ai_gateway",
                            dropped_orphan_tool_messages = dropped_orphans,
                            "thread_mutation_dropped_leading_orphan_tool_messages"
                        );
                    }
                }
                (None, _) | (_, None) => {
                    if !request.trim_if_resolvable {
                        return Err(GatewayError::new(
                            GatewayErrorKind::InvalidRequest,
                            format!(
                                "cannot resolve message boundaries '{:?}' -> '{:?}' for thread '{}'",
                                range.start, range.end, thread_id
                            ),
                        )
                        .with_retryable(false));
                    }
                }
            }
        }

        let effective_system_prompt_changed = thread
            .system_prompt_mode
            .apply_update(request.system_prompt_update)?;

        Ok(ThreadMessageMutationOutcome {
            removed_messages,
            remaining_messages: thread.messages.len(),
            effective_system_prompt_changed,
        })
    }

    fn sweep_expired(&self, chats: &mut HashMap<String, ChatData>) {
        let now = Instant::now();
        chats.retain(|_, c| c.expires_at > now);
    }

    fn trim_context(&self, thread: &mut ThreadData) {
        if self.max_turn_context_messages == 0 {
            return;
        }
        if thread.messages.len() <= self.max_turn_context_messages {
            return;
        }
        let remove_count = thread.messages.len() - self.max_turn_context_messages;
        thread.messages.drain(0..remove_count);
        let dropped_orphans = drop_leading_orphan_tool_messages(&mut thread.messages);
        if dropped_orphans > 0 {
            tracing::warn!(
                target: "ai_gateway",
                dropped_orphan_tool_messages = dropped_orphans,
                "trim_context_dropped_leading_orphan_tool_messages"
            );
        }
    }

    fn next_thread_id(&self) -> String {
        let value = self.thread_seq.fetch_add(1, Ordering::Relaxed);
        format!("thread-{value}")
    }
}

impl ThreadSystemPromptMode {
    fn resolve(&self, chat_system_prompt: Option<&str>) -> Option<String> {
        match self {
            ThreadSystemPromptMode::UseChatDefault => {
                chat_system_prompt.map(std::string::ToString::to_string)
            }
            ThreadSystemPromptMode::Override(value) => Some(value.clone()),
            ThreadSystemPromptMode::Cleared => None,
        }
    }

    fn apply_update(&mut self, update: SystemPromptUpdate) -> Result<bool, GatewayError> {
        match update {
            SystemPromptUpdate::Keep => Ok(false),
            SystemPromptUpdate::Replace(prompt) => {
                let trimmed = prompt.trim();
                if trimmed.is_empty() {
                    return Err(GatewayError::new(
                        GatewayErrorKind::InvalidRequest,
                        "system_prompt_update.replace must not be empty",
                    )
                    .with_retryable(false));
                }
                let next = ThreadSystemPromptMode::Override(trimmed.to_string());
                let changed = match self {
                    ThreadSystemPromptMode::Override(current) => current != trimmed,
                    ThreadSystemPromptMode::UseChatDefault | ThreadSystemPromptMode::Cleared => {
                        true
                    }
                };
                *self = next;
                Ok(changed)
            }
            SystemPromptUpdate::Clear => {
                let changed = !matches!(self, ThreadSystemPromptMode::Cleared);
                *self = ThreadSystemPromptMode::Cleared;
                Ok(changed)
            }
        }
    }
}

fn resolve_message_boundary(
    messages: &[ChatMessage],
    selector: &MessageBoundarySelector,
) -> Option<usize> {
    match selector {
        MessageBoundarySelector::FirstUserMessage => messages
            .iter()
            .position(|message| matches!(message.role, ChatRole::User)),
        MessageBoundarySelector::LatestAssistantToolBatchEnd => {
            let assistant_idx =
                messages
                    .iter()
                    .enumerate()
                    .rev()
                    .find_map(|(index, message)| {
                        if matches!(message.role, ChatRole::Assistant)
                            && !message.tool_calls.is_empty()
                        {
                            Some(index)
                        } else {
                            None
                        }
                    })?;
            let call_ids = messages[assistant_idx]
                .tool_calls
                .iter()
                .map(|call| call.id.as_str())
                .collect::<HashSet<_>>();
            if call_ids.is_empty() {
                return Some(assistant_idx);
            }

            let mut boundary = assistant_idx;
            for (index, message) in messages.iter().enumerate().skip(assistant_idx + 1) {
                if !matches!(message.role, ChatRole::Tool) {
                    break;
                }
                let Some(tool_call_id) = message.tool_call_id.as_deref() else {
                    break;
                };
                if !call_ids.contains(tool_call_id) {
                    break;
                }
                boundary = index;
            }
            Some(boundary)
        }
    }
}

fn drop_leading_orphan_tool_messages(messages: &mut Vec<ChatMessage>) -> usize {
    let dropped = messages
        .iter()
        .take_while(|message| matches!(message.role, ChatRole::Tool))
        .count();
    if dropped > 0 {
        messages.drain(0..dropped);
    }
    dropped
}
