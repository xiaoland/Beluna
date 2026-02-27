use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

use tokio::sync::RwLock;

use crate::ai_gateway::error::{GatewayError, GatewayErrorKind};

use super::types::{ChatMessage, UsageStats};

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
    next_turn_id: u64,
    metrics: ThreadMetrics,
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
        messages.extend_from_slice(input_messages);
        Ok(PreparedTurn {
            turn_id: thread.next_turn_id,
            messages: Arc::new(messages),
        })
    }

    pub(crate) async fn commit_turn_success(
        &self,
        chat_id: &str,
        thread_id: &str,
        input_messages: Vec<ChatMessage>,
        assistant_message: ChatMessage,
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

        thread.messages.extend(input_messages);
        thread.messages.push(assistant_message);
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
    }

    fn next_thread_id(&self) -> String {
        let value = self.thread_seq.fetch_add(1, Ordering::Relaxed);
        format!("thread-{value}")
    }
}
