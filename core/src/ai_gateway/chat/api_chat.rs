use std::{
    collections::HashMap,
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
    types::AIGatewayConfig,
};

use super::{
    capabilities::CapabilityGuard,
    runtime::ChatRuntime,
    thread::{Thread, ThreadState},
    thread_types::{CloneThreadOptions, ThreadOptions, TurnQuery, TurnSummary},
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
            default_route: config.chat.default_route.clone(),
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
        let route = opts
            .route_or_alias
            .as_deref()
            .or(self.runtime.default_route.as_deref());
        let backend = self.runtime.resolve_backend(route).await?;
        self.open_thread_with_backend(opts, backend).await
    }

    pub async fn open_thread_with_route(
        &self,
        route_or_alias: impl AsRef<str>,
        mut opts: ThreadOptions,
    ) -> Result<Thread, GatewayError> {
        opts.route_or_alias = Some(route_or_alias.as_ref().to_string());
        self.open_thread(opts).await
    }

    pub async fn clone_thread_with_turns(
        &self,
        source_thread: &Thread,
        ordered_turn_ids: &[u64],
        opts: CloneThreadOptions,
    ) -> Result<Thread, GatewayError> {
        let source_state = source_thread.state.lock().await;
        let mut selected_turns = select_turns(&source_state.turns, ordered_turn_ids)?;
        reindex_turn_ids(&mut selected_turns);
        let source_backend = source_state.backend.clone();
        let source_tools = source_state.tools.clone();
        let source_system_prompt = source_state.system_prompt.clone();
        let source_default_output_mode = source_state.default_output_mode.clone();
        let source_default_limits = source_state.default_limits.clone();
        let source_default_turn_timeout_ms = source_state.default_turn_timeout_ms;
        let source_enable_thinking = source_state.enable_thinking;
        drop(source_state);

        let backend = if let Some(route) = opts.route_or_alias.as_deref() {
            self.runtime.resolve_backend(Some(route)).await?
        } else {
            source_backend
        };

        let thread_id = opts.thread_id.unwrap_or_else(|| {
            format!("thread-{}", self.thread_seq.fetch_add(1, Ordering::Relaxed))
        });
        let next_turn_id = selected_turns.len() as u64 + 1;

        let state = ThreadState {
            backend,
            turns: selected_turns,
            tools: source_tools,
            system_prompt: opts.system_prompt.or(source_system_prompt),
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
        Ok(thread)
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
            route_or_alias: _,
            tools,
            system_prompt,
            default_output_mode,
            default_limits,
            enable_thinking,
            seed_turns,
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
        Ok(thread)
    }
}

fn select_turns(
    source_turns: &[Turn],
    ordered_turn_ids: &[u64],
) -> Result<Vec<Turn>, GatewayError> {
    if ordered_turn_ids.is_empty() {
        return Ok(Vec::new());
    }

    let by_id = source_turns
        .iter()
        .map(|turn| (turn.turn_id(), turn.clone()))
        .collect::<HashMap<_, _>>();

    ordered_turn_ids
        .iter()
        .map(|turn_id| {
            by_id.get(turn_id).cloned().ok_or_else(|| {
                GatewayError::new(
                    GatewayErrorKind::InvalidRequest,
                    format!("turn_id '{}' not found in source thread", turn_id),
                )
                .with_retryable(false)
            })
        })
        .collect::<Result<Vec<_>, _>>()
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
