use std::{
    collections::{BTreeMap, HashMap},
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};

use tokio::sync::RwLock;

use crate::ai_gateway::{
    chat::types::{ChatSessionOpenRequest, ChatThreadOpenRequest, ChatThreadState},
    error::{GatewayError, GatewayErrorKind},
    types_chat::{BelunaMessage, UsageStats},
};

pub(crate) struct InMemoryChatSessionStore {
    sessions: RwLock<HashMap<String, SessionState>>,
    session_seq: AtomicU64,
    thread_seq: AtomicU64,
    ttl: Duration,
    max_turn_context_messages: usize,
}

struct SessionState {
    default_route_ref: Option<String>,
    metadata: BTreeMap<String, String>,
    threads: HashMap<String, ThreadState>,
    created_at: Instant,
    expires_at: Instant,
}

struct ThreadState {
    messages: Vec<BelunaMessage>,
    next_turn_id: u64,
    metadata: BTreeMap<String, String>,
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

pub(crate) struct PreparedTurn {
    pub turn_id: u64,
    pub route_ref: Option<String>,
    pub messages: Vec<BelunaMessage>,
}

pub(crate) struct TurnCommitOutcome {
    pub turns_total: u64,
    pub tool_calls_total: u64,
    pub tokens_in_total: u64,
    pub tokens_out_total: u64,
    pub failures_total: u64,
    pub last_turn_latency_ms: Option<u64>,
}

impl InMemoryChatSessionStore {
    pub(crate) fn new(ttl_seconds: u64, max_turn_context_messages: usize) -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
            session_seq: AtomicU64::new(1),
            thread_seq: AtomicU64::new(1),
            ttl: Duration::from_secs(ttl_seconds.max(1)),
            max_turn_context_messages,
        }
    }

    pub(crate) async fn open_session(
        &self,
        request: ChatSessionOpenRequest,
        fallback_default_route_ref: Option<String>,
    ) -> Result<(String, Option<String>), GatewayError> {
        let mut sessions = self.sessions.write().await;
        self.sweep_expired_locked(&mut sessions);

        let now = Instant::now();
        let session_id = request
            .session_id
            .unwrap_or_else(|| self.next_session_id());
        let default_route_ref = request
            .default_route_ref
            .or(fallback_default_route_ref)
            .filter(|value| !value.trim().is_empty());

        if let Some(existing) = sessions.get_mut(&session_id) {
            if default_route_ref.is_some() {
                existing.default_route_ref = default_route_ref.clone();
            }
            if !request.metadata.is_empty() {
                existing.metadata.extend(request.metadata);
            }
            existing.expires_at = now + self.ttl;
            return Ok((session_id, existing.default_route_ref.clone()));
        }

        sessions.insert(
            session_id.clone(),
            SessionState {
                default_route_ref: default_route_ref.clone(),
                metadata: request.metadata,
                threads: HashMap::new(),
                created_at: now,
                expires_at: now + self.ttl,
            },
        );
        Ok((session_id, default_route_ref))
    }

    pub(crate) async fn close_session(&self, session_id: &str) {
        let mut sessions = self.sessions.write().await;
        sessions.remove(session_id);
    }

    pub(crate) async fn open_thread(
        &self,
        session_id: &str,
        request: ChatThreadOpenRequest,
    ) -> Result<String, GatewayError> {
        let mut sessions = self.sessions.write().await;
        self.sweep_expired_locked(&mut sessions);

        let session = sessions.get_mut(session_id).ok_or_else(|| {
            GatewayError::new(
                GatewayErrorKind::InvalidRequest,
                format!("chat session '{}' not found", session_id),
            )
            .with_retryable(false)
        })?;
        session.expires_at = Instant::now() + self.ttl;

        let thread_id = request.thread_id.unwrap_or_else(|| self.next_thread_id());
        let entry = session.threads.entry(thread_id.clone()).or_insert_with(|| ThreadState {
            messages: Vec::new(),
            next_turn_id: 0,
            metadata: BTreeMap::new(),
            metrics: ThreadMetrics::default(),
        });
        if !request.seed_messages.is_empty() {
            entry.messages.extend(request.seed_messages);
            self.trim_context(entry);
        }
        if !request.metadata.is_empty() {
            entry.metadata.extend(request.metadata);
        }

        Ok(thread_id)
    }

    pub(crate) async fn prepare_turn(
        &self,
        session_id: &str,
        thread_id: &str,
        route_ref_override: Option<String>,
        fallback_default_route_ref: Option<String>,
        input_messages: &[BelunaMessage],
    ) -> Result<PreparedTurn, GatewayError> {
        let mut sessions = self.sessions.write().await;
        self.sweep_expired_locked(&mut sessions);

        let session = sessions.get_mut(session_id).ok_or_else(|| {
            GatewayError::new(
                GatewayErrorKind::InvalidRequest,
                format!("chat session '{}' not found", session_id),
            )
            .with_retryable(false)
        })?;
        session.expires_at = Instant::now() + self.ttl;

        let thread = session.threads.get(thread_id).ok_or_else(|| {
            GatewayError::new(
                GatewayErrorKind::InvalidRequest,
                format!("chat thread '{}' not found", thread_id),
            )
            .with_retryable(false)
        })?;

        let route_ref = route_ref_override
            .filter(|value| !value.trim().is_empty())
            .or_else(|| session.default_route_ref.clone())
            .or(fallback_default_route_ref)
            .filter(|value| !value.trim().is_empty());

        let mut messages = thread.messages.clone();
        messages.extend_from_slice(input_messages);
        Ok(PreparedTurn {
            turn_id: thread.next_turn_id,
            route_ref,
            messages,
        })
    }

    pub(crate) async fn commit_turn_success(
        &self,
        session_id: &str,
        thread_id: &str,
        input_messages: Vec<BelunaMessage>,
        assistant_message: BelunaMessage,
        usage: Option<UsageStats>,
        tool_call_count: usize,
        latency_ms: u64,
    ) -> Result<TurnCommitOutcome, GatewayError> {
        let mut sessions = self.sessions.write().await;
        let session = sessions.get_mut(session_id).ok_or_else(|| {
            GatewayError::new(
                GatewayErrorKind::InvalidRequest,
                format!("chat session '{}' not found", session_id),
            )
            .with_retryable(false)
        })?;
        session.expires_at = Instant::now() + self.ttl;

        let thread = session.threads.get_mut(thread_id).ok_or_else(|| {
            GatewayError::new(
                GatewayErrorKind::InvalidRequest,
                format!("chat thread '{}' not found", thread_id),
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
            .map(|item| {
                (
                    item.input_tokens.unwrap_or(0),
                    item.output_tokens.unwrap_or(0),
                )
            })
            .unwrap_or((0, 0));
        thread.metrics.tokens_in_total = thread.metrics.tokens_in_total.saturating_add(tokens_in);
        thread.metrics.tokens_out_total = thread.metrics.tokens_out_total.saturating_add(tokens_out);

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
        session_id: &str,
        thread_id: &str,
        latency_ms: u64,
    ) -> Result<TurnCommitOutcome, GatewayError> {
        let mut sessions = self.sessions.write().await;
        let session = sessions.get_mut(session_id).ok_or_else(|| {
            GatewayError::new(
                GatewayErrorKind::InvalidRequest,
                format!("chat session '{}' not found", session_id),
            )
            .with_retryable(false)
        })?;
        let thread = session.threads.get_mut(thread_id).ok_or_else(|| {
            GatewayError::new(
                GatewayErrorKind::InvalidRequest,
                format!("chat thread '{}' not found", thread_id),
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
        session_id: &str,
        thread_id: &str,
    ) -> Result<ChatThreadState, GatewayError> {
        let sessions = self.sessions.read().await;
        let session = sessions.get(session_id).ok_or_else(|| {
            GatewayError::new(
                GatewayErrorKind::InvalidRequest,
                format!("chat session '{}' not found", session_id),
            )
            .with_retryable(false)
        })?;
        let thread = session.threads.get(thread_id).ok_or_else(|| {
            GatewayError::new(
                GatewayErrorKind::InvalidRequest,
                format!("chat thread '{}' not found", thread_id),
            )
            .with_retryable(false)
        })?;

        Ok(ChatThreadState {
            session_id: session_id.to_string(),
            thread_id: thread_id.to_string(),
            next_turn_id: thread.next_turn_id,
            message_count: thread.messages.len(),
            turns_total: thread.metrics.turns_total,
            tool_calls_total: thread.metrics.tool_calls_total,
            failures_total: thread.metrics.failures_total,
            last_turn_latency_ms: thread.metrics.last_turn_latency_ms,
        })
    }

    fn sweep_expired_locked(&self, sessions: &mut HashMap<String, SessionState>) {
        let now = Instant::now();
        sessions.retain(|_, state| {
            let _ = state.created_at;
            state.expires_at > now
        });
    }

    fn trim_context(&self, thread: &mut ThreadState) {
        if self.max_turn_context_messages == 0 {
            return;
        }
        if thread.messages.len() <= self.max_turn_context_messages {
            return;
        }

        let remove_count = thread.messages.len() - self.max_turn_context_messages;
        thread.messages.drain(0..remove_count);
    }

    fn next_session_id(&self) -> String {
        let value = self.session_seq.fetch_add(1, Ordering::Relaxed);
        format!("session-{value}")
    }

    fn next_thread_id(&self) -> String {
        let value = self.thread_seq.fetch_add(1, Ordering::Relaxed);
        format!("thread-{value}")
    }
}
