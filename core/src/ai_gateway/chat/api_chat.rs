use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

use tokio::sync::RwLock;

use crate::ai_gateway::{
    adapters::build_default_adapters,
    credentials::CredentialProvider,
    error::{GatewayError, GatewayErrorKind},
    resilience::ResilienceEngine,
    router::BackendRouter,
    types::{AIGatewayConfig, ChatRouteRef},
};
use crate::observability::runtime as observability_runtime;

use super::{
    capabilities::CapabilityGuard,
    runtime::ChatRuntime,
    thread::{
        Thread, ThreadState, metadata_parent_span_id, metadata_tick, thread_messages_snapshot,
        thread_turn_summaries,
    },
    thread_types::{
        DeriveContextOptions, RewriteContextOptions, SystemPromptAction, ThreadContextRequest,
        ThreadContextResult, ThreadOptions, TurnQuery, TurnRetentionPolicy, TurnSummary,
    },
    turn::Turn,
    types::OutputMode,
};

#[derive(Clone)]
pub struct Chat {
    chat_id: String,
    runtime: Arc<ChatRuntime>,
    threads: Arc<RwLock<HashMap<String, Thread>>>,
    thread_seq: Arc<AtomicU64>,
}

impl Chat {
    pub fn new(
        config: &AIGatewayConfig,
        credential_provider: Arc<dyn CredentialProvider>,
    ) -> Result<Self, GatewayError> {
        let runtime = Arc::new(ChatRuntime {
            router: BackendRouter::new(config)?,
            credential_provider,
            adapters: build_default_adapters(),
            capability_guard: CapabilityGuard,
            resilience: ResilienceEngine::new(config.resilience.clone()),
            default_route_ref: config.chat.default_route.clone(),
            default_turn_timeout_ms: config.chat.default_turn_timeout_ms,
        });

        Ok(Self {
            chat_id: "chat-gateway".to_string(),
            runtime,
            threads: Arc::new(RwLock::new(HashMap::new())),
            thread_seq: Arc::new(AtomicU64::new(1)),
        })
    }

    pub async fn open_thread(&self, opts: ThreadOptions) -> Result<Thread, GatewayError> {
        let route_ref = opts
            .route_ref
            .as_ref()
            .or(self.runtime.default_route_ref.as_ref());
        let backend = self.runtime.resolve_backend_route_ref(route_ref).await?;
        self.open_thread_with_backend(opts, backend).await
    }

    pub async fn open_thread_with_route_ref(
        &self,
        route_ref: ChatRouteRef,
        mut opts: ThreadOptions,
    ) -> Result<Thread, GatewayError> {
        opts.route_ref = Some(route_ref);
        self.open_thread(opts).await
    }

    pub async fn derive_context(
        &self,
        source_thread: &Thread,
        request: ThreadContextRequest,
        opts: DeriveContextOptions,
    ) -> Result<(Thread, ThreadContextResult), GatewayError> {
        let source_state = source_thread.state.lock().await;
        let source_thread_id = source_thread.thread_id().to_string();
        let source_turn_ids = source_state
            .turns
            .iter()
            .map(|turn| turn.turn_id())
            .collect::<Vec<_>>();
        let selected_turns = select_turns_by_retention(&source_state.turns, &request.retention)?;
        let kept_turn_ids = selected_turns
            .iter()
            .map(|turn| turn.turn_id())
            .collect::<Vec<_>>();
        let dropped_turn_ids = dropped_turn_ids_from_source(&source_turn_ids, &kept_turn_ids);
        let source_backend = source_state.backend.clone();
        let source_tools = source_state.tools.clone();
        let source_system_prompt = source_state.system_prompt.clone();
        let source_default_output_mode = source_state.default_output_mode.clone();
        let source_default_limits = source_state.default_limits.clone();
        let source_default_turn_timeout_ms = source_state.default_turn_timeout_ms;
        let source_enable_thinking = source_state.enable_thinking;
        drop(source_state);

        let backend = if let Some(route_ref) = opts.route_ref.as_ref() {
            self.runtime
                .resolve_backend_route_ref(Some(route_ref))
                .await?
        } else {
            source_backend
        };
        let system_prompt =
            apply_system_prompt_action(source_system_prompt, &request.system_prompt)?;

        let thread_id = opts.thread_id.unwrap_or_else(|| {
            format!("thread-{}", self.thread_seq.fetch_add(1, Ordering::Relaxed))
        });
        let next_turn_id = next_turn_id_from_turns(&selected_turns);

        let state = ThreadState {
            backend,
            turns: selected_turns,
            tools: source_tools,
            system_prompt,
            default_output_mode: source_default_output_mode,
            default_limits: source_default_limits,
            default_turn_timeout_ms: source_default_turn_timeout_ms,
            enable_thinking: source_enable_thinking,
            next_turn_id,
        };

        let thread = Thread::new(
            self.chat_id.clone(),
            thread_id.clone(),
            Arc::clone(&self.runtime),
            state,
        );
        self.threads.write().await.insert(thread_id, thread.clone());
        emit_thread_snapshot_event(
            &thread,
            "derived",
            &opts.metadata,
            ThreadSnapshotObservability {
                source_thread_id: Some(source_thread_id),
                source_turn_ids: Some(kept_turn_ids.clone()),
                kept_turn_ids: Some(kept_turn_ids.clone()),
                dropped_turn_ids: Some(dropped_turn_ids.clone()),
                continuation_dropped: Some(request.drop_unfinished_continuation),
                context_reason: Some(request.reason.as_label().to_string()),
            },
        )
        .await;
        Ok((
            thread,
            ThreadContextResult {
                kept_turn_ids,
                dropped_turn_ids,
                continuation_dropped: request.drop_unfinished_continuation,
            },
        ))
    }

    pub async fn rewrite_context(
        &self,
        thread: &Thread,
        request: ThreadContextRequest,
        opts: RewriteContextOptions,
    ) -> Result<ThreadContextResult, GatewayError> {
        let (kept_turn_ids, dropped_turn_ids) = {
            let mut state = thread.state.lock().await;
            let source_turn_ids = state
                .turns
                .iter()
                .map(|turn| turn.turn_id())
                .collect::<Vec<_>>();
            let selected_turns = select_turns_by_retention(&state.turns, &request.retention)?;
            let kept_turn_ids = selected_turns
                .iter()
                .map(|turn| turn.turn_id())
                .collect::<Vec<_>>();
            let dropped_turn_ids = dropped_turn_ids_from_source(&source_turn_ids, &kept_turn_ids);
            let next_turn_id = next_turn_id_from_turns(&selected_turns);
            state.turns = selected_turns;
            state.next_turn_id = next_turn_id;
            state.system_prompt =
                apply_system_prompt_action(state.system_prompt.clone(), &request.system_prompt)?;
            (kept_turn_ids, dropped_turn_ids)
        };

        emit_thread_snapshot_event(
            thread,
            "rewritten",
            &opts.metadata,
            ThreadSnapshotObservability {
                source_thread_id: Some(thread.thread_id().to_string()),
                source_turn_ids: Some(kept_turn_ids.clone()),
                kept_turn_ids: Some(kept_turn_ids.clone()),
                dropped_turn_ids: Some(dropped_turn_ids.clone()),
                continuation_dropped: Some(request.drop_unfinished_continuation),
                context_reason: Some(request.reason.as_label().to_string()),
            },
        )
        .await;
        Ok(ThreadContextResult {
            kept_turn_ids,
            dropped_turn_ids,
            continuation_dropped: request.drop_unfinished_continuation,
        })
    }

    pub async fn query_turns(
        &self,
        thread_id: &str,
        query: TurnQuery,
    ) -> Result<Vec<TurnSummary>, GatewayError> {
        let thread = self
            .threads
            .read()
            .await
            .get(thread_id)
            .cloned()
            .ok_or_else(|| {
                GatewayError::new(
                    GatewayErrorKind::InvalidRequest,
                    format!("thread '{}' not found", thread_id),
                )
                .with_retryable(false)
            })?;

        let refs = thread.find_turns(query).await;
        let turns = thread.turns().await;
        Ok(refs
            .into_iter()
            .filter_map(|turn_ref| turns.get(turn_ref.index))
            .map(|turn| TurnSummary {
                turn_id: turn.turn_id(),
                message_count: turn.message_count(),
                tool_call_count: turn.tool_call_count(),
                completed: turn.completed(),
            })
            .collect::<Vec<_>>())
    }

    async fn open_thread_with_backend(
        &self,
        opts: ThreadOptions,
        backend: super::runtime::BoundBackend,
    ) -> Result<Thread, GatewayError> {
        let ThreadOptions {
            thread_id,
            route_ref: _,
            tools,
            system_prompt,
            default_output_mode,
            default_limits,
            enable_thinking,
            seed_turns,
            metadata,
        } = opts;

        let thread_id = thread_id.unwrap_or_else(|| {
            format!("thread-{}", self.thread_seq.fetch_add(1, Ordering::Relaxed))
        });
        let mut turns = seed_turns;
        for turn in &turns {
            turn.validate_tool_linkage()?;
        }
        reindex_turn_ids(&mut turns);
        let next_turn_id = turns.len() as u64 + 1;

        let state = ThreadState {
            backend,
            turns,
            tools,
            system_prompt,
            default_output_mode: default_output_mode.unwrap_or(OutputMode::Text),
            default_limits: default_limits.unwrap_or_default(),
            default_turn_timeout_ms: self.runtime.default_turn_timeout_ms,
            enable_thinking,
            next_turn_id,
        };

        let thread = Thread::new(
            self.chat_id.clone(),
            thread_id.clone(),
            Arc::clone(&self.runtime),
            state,
        );
        self.threads.write().await.insert(thread_id, thread.clone());
        emit_thread_snapshot_event(
            &thread,
            "opened",
            &metadata,
            ThreadSnapshotObservability::default(),
        )
        .await;
        Ok(thread)
    }
}

#[derive(Default)]
struct ThreadSnapshotObservability {
    source_thread_id: Option<String>,
    source_turn_ids: Option<Vec<u64>>,
    kept_turn_ids: Option<Vec<u64>>,
    dropped_turn_ids: Option<Vec<u64>>,
    continuation_dropped: Option<bool>,
    context_reason: Option<String>,
}

async fn emit_thread_snapshot_event(
    thread: &Thread,
    kind: &str,
    metadata: &BTreeMap<String, String>,
    observability: ThreadSnapshotObservability,
) {
    let tick = metadata_tick(metadata);
    let turns = thread.turns().await;
    let context_reason = observability
        .context_reason
        .or_else(|| metadata_context_reason(metadata))
        .or_else(|| Some(kind.to_string()));
    observability_runtime::emit_ai_gateway_chat_thread(
        observability_runtime::AiGatewayChatThreadArgs {
            tick,
            thread_id: thread.thread_id().to_string(),
            span_id: format!("ai-gateway.chat.thread:{kind}:{}", thread.thread_id()),
            parent_span_id: metadata_parent_span_id(metadata),
            organ_id: metadata.get("organ_id").cloned(),
            request_id: None,
            kind: kind.to_string(),
            messages: thread_messages_snapshot(&turns),
            turn_summaries: Some(thread_turn_summaries(&turns)),
            source_thread_id: observability.source_thread_id,
            source_turn_ids: observability
                .source_turn_ids
                .map(|value| serde_json::json!(value)),
            kept_turn_ids: observability
                .kept_turn_ids
                .map(|value| serde_json::json!(value)),
            dropped_turn_ids: observability
                .dropped_turn_ids
                .map(|value| serde_json::json!(value)),
            continuation_dropped: observability.continuation_dropped,
            context_reason,
        },
    );
}

fn metadata_context_reason(metadata: &BTreeMap<String, String>) -> Option<String> {
    metadata.get("context_reason").and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return None;
        }
        Some(trimmed.to_string())
    })
}

fn dropped_turn_ids_from_source(source_turn_ids: &[u64], kept_turn_ids: &[u64]) -> Vec<u64> {
    let kept = kept_turn_ids.iter().copied().collect::<BTreeSet<_>>();
    source_turn_ids
        .iter()
        .copied()
        .filter(|turn_id| !kept.contains(turn_id))
        .collect::<Vec<_>>()
}

fn select_turns_by_retention(
    source_turns: &[Turn],
    retention: &TurnRetentionPolicy,
) -> Result<Vec<Turn>, GatewayError> {
    match retention {
        TurnRetentionPolicy::KeepAll => Ok(source_turns.to_vec()),
        TurnRetentionPolicy::KeepLastTurns { count } => {
            let keep = (*count).min(source_turns.len());
            let from = source_turns.len().saturating_sub(keep);
            Ok(source_turns[from..].to_vec())
        }
        TurnRetentionPolicy::KeepSelectedTurnIds { turn_ids } => {
            ensure_unique_turn_ids(turn_ids)?;
            select_turns_in_source_order(source_turns, turn_ids)
        }
        TurnRetentionPolicy::DropAll => Ok(Vec::new()),
    }
}

fn ensure_unique_turn_ids(turn_ids: &[u64]) -> Result<(), GatewayError> {
    let mut seen = BTreeSet::new();
    for turn_id in turn_ids {
        if seen.insert(*turn_id) {
            continue;
        }
        return Err(GatewayError::new(
            GatewayErrorKind::InvalidRequest,
            format!("duplicate turn_id '{}' in retention policy", turn_id),
        )
        .with_retryable(false));
    }
    Ok(())
}

fn select_turns_in_source_order(
    source_turns: &[Turn],
    selected_turn_ids: &[u64],
) -> Result<Vec<Turn>, GatewayError> {
    if selected_turn_ids.is_empty() {
        return Ok(Vec::new());
    }

    let existing_turn_ids = source_turns
        .iter()
        .map(Turn::turn_id)
        .collect::<BTreeSet<_>>();
    for turn_id in selected_turn_ids {
        if existing_turn_ids.contains(turn_id) {
            continue;
        }
        return Err(GatewayError::new(
            GatewayErrorKind::InvalidRequest,
            format!("turn_id '{}' not found in source thread", turn_id),
        )
        .with_retryable(false));
    }

    let selected_turn_ids_set = selected_turn_ids.iter().copied().collect::<BTreeSet<_>>();
    Ok(source_turns
        .iter()
        .filter(|turn| selected_turn_ids_set.contains(&turn.turn_id()))
        .cloned()
        .collect::<Vec<_>>())
}

fn next_turn_id_from_turns(turns: &[Turn]) -> u64 {
    turns
        .iter()
        .map(Turn::turn_id)
        .max()
        .unwrap_or(0)
        .saturating_add(1)
}

fn apply_system_prompt_action(
    current: Option<String>,
    action: &SystemPromptAction,
) -> Result<Option<String>, GatewayError> {
    match action {
        SystemPromptAction::Keep => Ok(current),
        SystemPromptAction::Clear => Ok(None),
        SystemPromptAction::Replace { prompt } => {
            let trimmed = prompt.trim();
            if trimmed.is_empty() {
                return Err(GatewayError::new(
                    GatewayErrorKind::InvalidRequest,
                    "system prompt cannot be empty for replace action",
                )
                .with_retryable(false));
            }
            Ok(Some(trimmed.to_string()))
        }
    }
}

fn reindex_turn_ids(turns: &mut [Turn]) {
    for (index, turn) in turns.iter_mut().enumerate() {
        let source_turn_id = turn.turn_id();
        let reindexed_turn_id = index as u64 + 1;
        turn.set_turn_id(reindexed_turn_id);
        if source_turn_id != reindexed_turn_id {
            turn.metadata_mut()
                .insert("source_turn_id".to_string(), source_turn_id.to_string());
        }
    }
}
