use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{
        Arc, Mutex, OnceLock, RwLock,
        atomic::{AtomicU64, Ordering},
    },
};

use anyhow::Result;
use async_trait::async_trait;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tracing::Instrument;

use crate::{
    config::SpineRuntimeConfig,
    observability::{
        contract::{AdapterLifecycleState, DispatchOutcomeClass, EndpointLifecycleTransition},
        runtime as observability_runtime,
    },
    spine::{
        SpineExecutionMode,
        adapters::{inline::SpineInlineAdapter, unix_socket::UnixSocketAdapter},
        endpoint::Endpoint,
        error::{SpineError, backend_failure, invalid_batch, registration_invalid},
        types::{ActDispatchResult, NeuralSignalDescriptor, NeuralSignalDescriptorRouteKey},
    },
    stem::{SenseAfferentPathway, StemControlPort},
    types::{
        Act, NeuralSignalDescriptorDropPatch, NeuralSignalDescriptorPatch, ProprioceptionDropPatch,
        ProprioceptionPatch, Sense,
    },
};

#[derive(Debug, Clone)]
pub struct BodyEndpointHandle {
    pub body_endpoint_id: String,
}

pub enum EndpointBinding {
    Inline(Arc<dyn Endpoint>),
    AdapterChannel(u64),
}

#[derive(Clone)]
enum EndpointDispatch {
    Inline(Arc<dyn Endpoint>),
    AdapterChannel(u64),
}

impl EndpointBinding {
    fn into_dispatch(self) -> EndpointDispatch {
        match self {
            EndpointBinding::Inline(endpoint) => EndpointDispatch::Inline(endpoint),
            EndpointBinding::AdapterChannel(channel_id) => {
                EndpointDispatch::AdapterChannel(channel_id)
            }
        }
    }
}

impl EndpointDispatch {
    fn channel_id(&self) -> Option<u64> {
        match self {
            EndpointDispatch::Inline(_) => None,
            EndpointDispatch::AdapterChannel(channel_id) => Some(*channel_id),
        }
    }

    fn is_compatible_with(&self, other: &Self) -> bool {
        match (self, other) {
            (EndpointDispatch::Inline(_), EndpointDispatch::Inline(_)) => true,
            (EndpointDispatch::AdapterChannel(lhs), EndpointDispatch::AdapterChannel(rhs)) => {
                lhs == rhs
            }
            _ => false,
        }
    }

    fn binding_label(&self) -> &'static str {
        match self {
            EndpointDispatch::Inline(_) => "inline",
            EndpointDispatch::AdapterChannel(_) => "adapter",
        }
    }
}

#[derive(Clone)]
struct RegisteredBodyEndpoint {
    body_endpoint_id: String,
    dispatch: EndpointDispatch,
    route_keys: BTreeSet<NeuralSignalDescriptorRouteKey>,
}

#[derive(Default)]
struct EndpointState {
    by_id: BTreeMap<String, RegisteredBodyEndpoint>,
    by_channel: BTreeMap<u64, BTreeSet<String>>,
}

struct RegisteredEndpointRoutes {
    dispatch: EndpointDispatch,
    route_keys: BTreeSet<NeuralSignalDescriptorRouteKey>,
}

#[derive(Default)]
struct RoutingState {
    by_endpoint: BTreeMap<String, RegisteredEndpointRoutes>,
    adapter_channels: BTreeMap<u64, tokio::sync::mpsc::UnboundedSender<Act>>,
}

pub struct Spine {
    mode: SpineExecutionMode,
    routing: RwLock<RoutingState>,
    next_adapter_channel_seq: AtomicU64,
    next_body_endpoint_seq: AtomicU64,
    shutdown: CancellationToken,
    tasks: Mutex<Vec<JoinHandle<Result<()>>>>,
    endpoint_state: Mutex<EndpointState>,
    inline_adapter: OnceLock<Arc<SpineInlineAdapter>>,
    afferent_pathway: SenseAfferentPathway,
    stem_control: Arc<dyn StemControlPort>,
    endpoint_proprioception: RwLock<BTreeMap<String, String>>,
}

#[async_trait]
pub trait SpineControlPort: Send + Sync {
    async fn publish_sense(&self, sense: Sense);
    async fn apply_proprioception_patch(&self, entries: BTreeMap<String, String>);
    async fn apply_proprioception_drop(&self, keys: Vec<String>);
    async fn refresh_topology_proprioception(&self);
}

impl Spine {
    pub fn new(
        config: &SpineRuntimeConfig,
        afferent_pathway: SenseAfferentPathway,
        stem_control: Arc<dyn StemControlPort>,
    ) -> Arc<Self> {
        let spine = Arc::new(Self {
            mode: SpineExecutionMode::SerializedDeterministic,
            routing: RwLock::new(RoutingState::default()),
            next_adapter_channel_seq: AtomicU64::new(0),
            next_body_endpoint_seq: AtomicU64::new(0),
            shutdown: CancellationToken::new(),
            tasks: Mutex::new(Vec::new()),
            endpoint_state: Mutex::new(EndpointState::default()),
            inline_adapter: OnceLock::new(),
            afferent_pathway: afferent_pathway.clone(),
            stem_control,
            endpoint_proprioception: RwLock::new(BTreeMap::new()),
        });

        spine.start_adapters(config);
        spine
    }

    fn start_adapters(self: &Arc<Self>, config: &SpineRuntimeConfig) {
        for (index, adapter_config) in config.adapters.iter().enumerate() {
            let adapter_id = (index as u64) + 1;
            match adapter_config {
                crate::config::SpineAdapterConfig::Inline {
                    config: adapter_cfg,
                } => {
                    let adapter = Arc::new(SpineInlineAdapter::new(
                        adapter_id,
                        adapter_cfg.clone(),
                        Arc::clone(self),
                        self.shutdown.clone(),
                    ));

                    if self.inline_adapter.set(Arc::clone(&adapter)).is_err() {
                        tracing::warn!(
                            target: "spine",
                            adapter_id = adapter_id,
                            "duplicate_inline_adapter_ignored"
                        );
                        continue;
                    }

                    tracing::info!(
                        target: "spine",
                        adapter_type = "inline",
                        adapter_id = adapter.adapter_id(),
                        act_queue_capacity = adapter_cfg.act_queue_capacity,
                        sense_queue_capacity = adapter_cfg.sense_queue_capacity,
                        "adapter_started"
                    );
                    observability_runtime::emit_spine_adapter_lifecycle(
                        "inline",
                        &adapter.adapter_id().to_string(),
                        AdapterLifecycleState::Enabled,
                        None,
                    );
                }
                crate::config::SpineAdapterConfig::UnixSocketNdjson {
                    config: adapter_cfg,
                } => {
                    let adapter =
                        UnixSocketAdapter::new(adapter_cfg.socket_path.clone(), adapter_id);
                    let spine = Arc::clone(self);
                    let shutdown = self.shutdown.clone();
                    let socket_path = adapter_cfg.socket_path.clone();
                    let adapter_span = tracing::info_span!(
                        target: "spine",
                        "unix_socket_adapter_task",
                        adapter_id = adapter_id,
                        socket_path = %socket_path.display()
                    );
                    let task = tokio::spawn(
                        async move {
                            tracing::info!(
                                target: "spine",
                                adapter_type = "unix-socket-ndjson",
                                adapter_id = adapter_id,
                                socket_path = %socket_path.display(),
                                "adapter_started"
                            );
                            observability_runtime::emit_spine_adapter_lifecycle(
                                "unix_socket_ndjson",
                                &adapter_id.to_string(),
                                AdapterLifecycleState::Enabled,
                                None,
                            );
                            let result = adapter.run(spine, shutdown).await;
                            if let Err(err) = &result {
                                let reason = err.to_string();
                                observability_runtime::emit_spine_adapter_lifecycle(
                                    "unix_socket_ndjson",
                                    &adapter_id.to_string(),
                                    AdapterLifecycleState::Faulted,
                                    Some(&reason),
                                );
                            }
                            result
                        }
                        .instrument(adapter_span),
                    );
                    self.tasks.lock().expect("lock poisoned").push(task);
                }
            }
        }
    }

    pub fn mode(&self) -> SpineExecutionMode {
        self.mode
    }

    pub fn body_endpoint_ids_snapshot(&self) -> Vec<String> {
        let state = self.endpoint_state.lock().expect("lock poisoned");
        let mut ids = state.by_id.keys().cloned().collect::<Vec<_>>();
        ids.sort();
        ids
    }

    pub fn inline_adapter(&self) -> Option<Arc<SpineInlineAdapter>> {
        self.inline_adapter.get().cloned()
    }

    #[tracing::instrument(
        name = "spine_on_act_final",
        target = "spine.dispatch",
        skip(self, act),
        fields(
            act_instance_id = %act.act_instance_id,
            endpoint_id = %act.endpoint_id,
            neural_signal_descriptor_id = %act.neural_signal_descriptor_id
        )
    )]
    pub async fn on_act_final(&self, act: Act) -> Result<ActDispatchResult, SpineError> {
        match self.dispatch_act(act.clone()).await {
            Ok(ActDispatchResult::Acknowledged { reference_id }) => {
                Ok(ActDispatchResult::Acknowledged { reference_id })
            }
            Ok(ActDispatchResult::Rejected {
                reason_code,
                reference_id,
            }) => {
                self.emit_dispatch_failure_sense(&act, &reason_code, &reference_id)
                    .await;
                Ok(ActDispatchResult::Rejected {
                    reason_code,
                    reference_id,
                })
            }
            Ok(ActDispatchResult::Lost {
                reason_code,
                reference_id,
            }) => {
                self.emit_dispatch_failure_sense(&act, &reason_code, &reference_id)
                    .await;
                Ok(ActDispatchResult::Lost {
                    reason_code,
                    reference_id,
                })
            }
            Err(err) => {
                let reason_code = "spine_dispatch_error".to_string();
                let reference_id =
                    format!("spine:error:{}:{}", act.act_instance_id, err.kind as u8);
                self.emit_dispatch_failure_sense(&act, &reason_code, &reference_id)
                    .await;
                Ok(ActDispatchResult::Lost {
                    reason_code,
                    reference_id,
                })
            }
        }
    }

    #[tracing::instrument(
        name = "spine_dispatch_act",
        target = "spine.dispatch",
        skip(self, act),
        fields(
            act_instance_id = %act.act_instance_id,
            endpoint_id = %act.endpoint_id,
            neural_signal_descriptor_id = %act.neural_signal_descriptor_id
        )
    )]
    pub async fn dispatch_act(&self, act: Act) -> Result<ActDispatchResult, SpineError> {
        if act.act_instance_id.trim().is_empty() || act.endpoint_id.trim().is_empty() {
            tracing::warn!(
                target: "spine.dispatch",
                "act_dispatch_invalid_input"
            );
            return Err(invalid_batch(
                "act dispatch is missing act_instance_id/endpoint_id",
            ));
        }

        let Some(dispatch) = self.resolve_dispatch(&act.endpoint_id) else {
            let outcome = ActDispatchResult::Rejected {
                reason_code: "endpoint_not_found".to_string(),
                reference_id: format!("spine:missing_endpoint:{}", act.act_instance_id),
            };
            Self::log_dispatch_outcome(&act, "unknown", None, &outcome);
            return Ok(outcome);
        };

        match dispatch {
            EndpointDispatch::Inline(endpoint) => {
                tracing::debug!(
                    target: "spine.dispatch",
                    dispatch_binding = "inline",
                    "dispatching_act_to_inline_endpoint"
                );

                match endpoint.invoke(act.clone()).await {
                    Ok(outcome) => {
                        Self::log_dispatch_outcome(&act, "inline", None, &outcome);
                        Ok(outcome)
                    }
                    Err(err) => {
                        tracing::warn!(
                            target: "spine.dispatch",
                            dispatch_binding = "inline",
                            error = %err,
                            "inline_endpoint_invoke_failed"
                        );
                        let outcome = ActDispatchResult::Rejected {
                            reason_code: "endpoint_error".to_string(),
                            reference_id: format!("spine:error:{}", act.act_instance_id),
                        };
                        Self::log_dispatch_outcome(&act, "inline", None, &outcome);
                        Ok(outcome)
                    }
                }
            }
            EndpointDispatch::AdapterChannel(channel_id) => {
                tracing::debug!(
                    target: "spine.dispatch",
                    dispatch_binding = "adapter",
                    adapter_channel_id = channel_id,
                    "dispatching_act_to_adapter_channel"
                );
                match self.invoke_adapter_endpoint(channel_id, act.clone()) {
                    Ok(outcome) => {
                        Self::log_dispatch_outcome(&act, "adapter", Some(channel_id), &outcome);
                        Ok(outcome)
                    }
                    Err(err) => {
                        tracing::warn!(
                            target: "spine.dispatch",
                            dispatch_binding = "adapter",
                            adapter_channel_id = channel_id,
                            error = %err,
                            "adapter_channel_invoke_failed"
                        );
                        let outcome = ActDispatchResult::Lost {
                            reason_code: "dispatch_lost".to_string(),
                            reference_id: format!("spine:lost:{}", act.act_instance_id),
                        };
                        Self::log_dispatch_outcome(&act, "adapter", Some(channel_id), &outcome);
                        Ok(outcome)
                    }
                }
            }
        }
    }

    async fn emit_dispatch_failure_sense(&self, act: &Act, reason_code: &str, reference_id: &str) {
        let sense = Sense {
            sense_instance_id: uuid::Uuid::new_v4().to_string(),
            endpoint_id: "core.spine".to_string(),
            neural_signal_descriptor_id: "dispatch.failed".to_string(),
            payload: format!(
                "act_instance_id={}; endpoint_id={}; neural_signal_descriptor_id={}; reason_code={}; reference_id={}",
                act.act_instance_id,
                act.endpoint_id,
                act.neural_signal_descriptor_id,
                reason_code,
                reference_id
            ),
            weight: 1.0,
            act_instance_id: Some(act.act_instance_id.clone()),
        };
        if let Err(err) = self.afferent_pathway.send(sense).await {
            tracing::warn!(
                target: "spine.dispatch",
                act_instance_id = %act.act_instance_id,
                error = %err,
                "failed_to_emit_dispatch_failure_sense"
            );
        }
    }

    pub fn add_endpoint(
        &self,
        endpoint_name: &str,
        binding: EndpointBinding,
    ) -> Result<BodyEndpointHandle> {
        let endpoint_name = endpoint_name.trim();
        if endpoint_name.is_empty() {
            return Err(anyhow::anyhow!("endpoint_name cannot be empty"));
        }

        let dispatch = binding.into_dispatch();
        if let Some(channel_id) = dispatch.channel_id()
            && !self.adapter_channel_exists(channel_id)
        {
            return Err(anyhow::anyhow!(
                "adapter channel {} is not connected",
                channel_id
            ));
        }

        let suffix = self
            .next_body_endpoint_seq
            .fetch_add(1, Ordering::Relaxed)
            .saturating_add(1);
        let body_endpoint_id = format!("{}.{}", endpoint_name, suffix);

        let registered = RegisteredBodyEndpoint {
            body_endpoint_id: body_endpoint_id.clone(),
            dispatch: dispatch.clone(),
            route_keys: BTreeSet::new(),
        };
        let mut state = self.endpoint_state.lock().expect("lock poisoned");
        if let Some(channel_id) = dispatch.channel_id() {
            state
                .by_channel
                .entry(channel_id)
                .or_default()
                .insert(body_endpoint_id.clone());
        }
        state.by_id.insert(body_endpoint_id.clone(), registered);
        let channel_or_session = endpoint_channel_or_session(&dispatch);
        drop(state);

        observability_runtime::emit_spine_endpoint_lifecycle(
            &body_endpoint_id,
            EndpointLifecycleTransition::Connected,
            channel_or_session,
            None,
        );

        Ok(BodyEndpointHandle { body_endpoint_id })
    }

    pub async fn add_ns_descriptors(
        &self,
        body_endpoint_id: &str,
        descriptors: Vec<NeuralSignalDescriptor>,
    ) -> Result<Vec<NeuralSignalDescriptor>> {
        if descriptors.is_empty() {
            return Ok(Vec::new());
        }

        let normalized_entries = {
            let state = self.endpoint_state.lock().expect("lock poisoned");
            let endpoint = state
                .by_id
                .get(body_endpoint_id)
                .ok_or_else(|| anyhow::anyhow!("body endpoint is not registered"))?;
            descriptors
                .into_iter()
                .map(|mut descriptor| {
                    descriptor.endpoint_id = endpoint.body_endpoint_id.clone();
                    descriptor
                })
                .collect::<Vec<_>>()
        };

        let patch_commit = self
            .stem_control
            .apply_neural_signal_descriptor_patch(NeuralSignalDescriptorPatch {
                entries: normalized_entries,
            })
            .await;

        for rejected in &patch_commit.rejected_entries {
            tracing::warn!(
                target = "spine",
                endpoint_id = %rejected.entry.endpoint_id,
                neural_signal_descriptor_id = %rejected.entry.neural_signal_descriptor_id,
                reason_code = %rejected.reason_code,
                "stem_rejected_ns_descriptor_patch_entry"
            );
        }

        if patch_commit.accepted_entries.is_empty() {
            return Ok(Vec::new());
        }

        let accepted_entries = patch_commit
            .accepted_entries
            .into_iter()
            .filter(|entry| {
                if entry.endpoint_id == body_endpoint_id {
                    return true;
                }
                tracing::warn!(
                    target = "spine",
                    requested_body_endpoint_id = body_endpoint_id,
                    committed_endpoint_id = %entry.endpoint_id,
                    neural_signal_descriptor_id = %entry.neural_signal_descriptor_id,
                    "stem_committed_unexpected_endpoint_ns_descriptor"
                );
                false
            })
            .collect::<Vec<_>>();

        if accepted_entries.is_empty() {
            return Ok(Vec::new());
        }

        let accepted_routes = accepted_entries
            .iter()
            .map(route_key_from_descriptor)
            .collect::<Vec<_>>();

        let dispatch = {
            let mut state = self.endpoint_state.lock().expect("lock poisoned");
            if let Some(endpoint) = state.by_id.get_mut(body_endpoint_id) {
                for route in &accepted_routes {
                    endpoint.route_keys.insert(route.clone());
                }
                Some(endpoint.dispatch.clone())
            } else {
                None
            }
        };
        let Some(dispatch) = dispatch else {
            self.rollback_ns_routes(accepted_routes).await;
            return Ok(Vec::new());
        };

        if let Err(err) = accepted_routes
            .iter()
            .cloned()
            .try_for_each(|route| self.upsert_route(route, dispatch.clone()))
        {
            {
                let mut state = self.endpoint_state.lock().expect("lock poisoned");
                if let Some(endpoint) = state.by_id.get_mut(body_endpoint_id) {
                    for route in &accepted_routes {
                        endpoint.route_keys.remove(route);
                    }
                }
            }
            self.rollback_ns_routes(accepted_routes).await;
            return Err(anyhow::anyhow!(err.to_string()));
        }

        Ok(accepted_entries)
    }

    pub async fn remove_endpoint(&self, body_endpoint_id: &str) {
        let endpoint = {
            let mut state = self.endpoint_state.lock().expect("lock poisoned");
            let Some(endpoint) = state.by_id.remove(body_endpoint_id) else {
                return;
            };

            if let Some(channel_id) = endpoint.dispatch.channel_id()
                && let Some(ids) = state.by_channel.get_mut(&channel_id)
            {
                ids.remove(body_endpoint_id);
                if ids.is_empty() {
                    state.by_channel.remove(&channel_id);
                }
            }

            endpoint
        };
        observability_runtime::emit_spine_endpoint_lifecycle(
            body_endpoint_id,
            EndpointLifecycleTransition::Dropped,
            endpoint_channel_or_session(&endpoint.dispatch),
            None,
        );
        let endpoint_routes = endpoint.route_keys.into_iter().collect::<Vec<_>>();

        let drop_commit = self
            .stem_control
            .apply_neural_signal_descriptor_drop(NeuralSignalDescriptorDropPatch {
                routes: endpoint_routes.clone(),
            })
            .await;

        for rejected in &drop_commit.rejected_routes {
            tracing::warn!(
                target = "spine",
                endpoint_id = %rejected.route.endpoint_id,
                neural_signal_descriptor_id = %rejected.route.neural_signal_descriptor_id,
                reason_code = %rejected.reason_code,
                "stem_rejected_ns_descriptor_drop_route"
            );
        }
        if drop_commit.accepted_routes.len() != endpoint_routes.len() {
            tracing::warn!(
                target = "spine",
                endpoint_id = %body_endpoint_id,
                expected_route_count = endpoint_routes.len(),
                accepted_route_count = drop_commit.accepted_routes.len(),
                "stem_drop_commit_partial_accept_for_endpoint"
            );
        }

        for route in &drop_commit.accepted_routes {
            self.remove_route(route);
        }
    }

    async fn rollback_ns_routes(&self, routes: Vec<NeuralSignalDescriptorRouteKey>) {
        if routes.is_empty() {
            return;
        }
        let drop_commit = self
            .stem_control
            .apply_neural_signal_descriptor_drop(NeuralSignalDescriptorDropPatch { routes })
            .await;
        for rejected in &drop_commit.rejected_routes {
            tracing::warn!(
                target = "spine",
                endpoint_id = %rejected.route.endpoint_id,
                neural_signal_descriptor_id = %rejected.route.neural_signal_descriptor_id,
                reason_code = %rejected.reason_code,
                "stem_rejected_ns_descriptor_rollback_route"
            );
        }
    }

    pub(crate) fn on_adapter_channel_open(
        &self,
        adapter_id: u64,
        tx: tokio::sync::mpsc::UnboundedSender<Act>,
    ) -> u64 {
        let channel_id = self.allocate_adapter_channel_id(adapter_id);
        let mut routing = self.routing.write().expect("lock poisoned");
        routing.adapter_channels.insert(channel_id, tx);
        channel_id
    }

    pub(crate) async fn on_adapter_channel_closed(&self, channel_id: u64) {
        {
            let mut routing = self.routing.write().expect("lock poisoned");
            routing.adapter_channels.remove(&channel_id);
        }

        let endpoint_ids = {
            let mut state = self.endpoint_state.lock().expect("lock poisoned");
            state.by_channel.remove(&channel_id).unwrap_or_default()
        };

        for endpoint_id in endpoint_ids {
            self.remove_endpoint(&endpoint_id).await;
        }
    }

    pub async fn shutdown(&self) {
        self.shutdown.cancel();
        let tasks = {
            let mut guard = self.tasks.lock().expect("lock poisoned");
            std::mem::take(&mut *guard)
        };
        for task in tasks {
            match task.await {
                Ok(Ok(())) => {}
                Ok(Err(err)) => {
                    tracing::error!(target: "spine", error = ?err, "spine_adapter_exited_with_error")
                }
                Err(err) => {
                    tracing::error!(target: "spine", error = %err, "spine_adapter_task_join_failed")
                }
            }
        }
    }

    fn allocate_adapter_channel_id(&self, adapter_id: u64) -> u64 {
        let sequence = self
            .next_adapter_channel_seq
            .fetch_add(1, Ordering::Relaxed)
            .saturating_add(1);
        (adapter_id << 32) | (sequence & 0xFFFF_FFFF)
    }

    fn adapter_channel_exists(&self, channel_id: u64) -> bool {
        self.routing
            .read()
            .expect("lock poisoned")
            .adapter_channels
            .contains_key(&channel_id)
    }

    fn resolve_dispatch(&self, endpoint_id: &str) -> Option<EndpointDispatch> {
        self.routing
            .read()
            .expect("lock poisoned")
            .by_endpoint
            .get(endpoint_id)
            .map(|entry| entry.dispatch.clone())
    }

    fn invoke_adapter_endpoint(
        &self,
        channel_id: u64,
        act: Act,
    ) -> Result<ActDispatchResult, SpineError> {
        let tx = self
            .routing
            .read()
            .expect("lock poisoned")
            .adapter_channels
            .get(&channel_id)
            .cloned()
            .ok_or_else(|| {
                backend_failure(format!("adapter channel {} is unavailable", channel_id))
            })?;

        if tx.send(act.clone()).is_err() {
            tracing::warn!(
                target: "spine.dispatch",
                dispatch_binding = "adapter",
                adapter_channel_id = channel_id,
                act_instance_id = %act.act_instance_id,
                "adapter_channel_send_failed"
            );
            return Err(backend_failure(format!(
                "failed to dispatch act {} to adapter channel {}",
                act.act_instance_id, channel_id
            )));
        }

        tracing::debug!(
            target: "spine.dispatch",
            dispatch_binding = "adapter",
            adapter_channel_id = channel_id,
            act_instance_id = %act.act_instance_id,
            "act_enqueued_to_adapter_channel"
        );
        Ok(ActDispatchResult::Acknowledged {
            reference_id: format!("adapter:act_sent:{}", act.act_instance_id),
        })
    }

    fn log_dispatch_outcome(
        act: &Act,
        dispatch_binding: &'static str,
        channel_id: Option<u64>,
        outcome: &ActDispatchResult,
    ) {
        observability_runtime::emit_spine_dispatch_outcome(
            &act.act_instance_id,
            &format!("endpoint:{}", act.endpoint_id),
            dispatch_outcome_class(outcome),
            Some(&act.neural_signal_descriptor_id),
            None,
            None,
        );
        match (outcome, channel_id) {
            (ActDispatchResult::Acknowledged { reference_id }, Some(channel_id)) => {
                tracing::info!(
                    target: "spine.dispatch",
                    act_instance_id = %act.act_instance_id,
                    endpoint_id = %act.endpoint_id,
                    neural_signal_descriptor_id = %act.neural_signal_descriptor_id,
                    dispatch_binding = dispatch_binding,
                    adapter_channel_id = channel_id,
                    reference_id = %reference_id,
                    "act_dispatch_acknowledged"
                );
            }
            (ActDispatchResult::Acknowledged { reference_id }, None) => {
                tracing::info!(
                    target: "spine.dispatch",
                    act_instance_id = %act.act_instance_id,
                    endpoint_id = %act.endpoint_id,
                    neural_signal_descriptor_id = %act.neural_signal_descriptor_id,
                    dispatch_binding = dispatch_binding,
                    reference_id = %reference_id,
                    "act_dispatch_acknowledged"
                );
            }
            (
                ActDispatchResult::Rejected {
                    reason_code,
                    reference_id,
                },
                Some(channel_id),
            ) => {
                tracing::warn!(
                    target: "spine.dispatch",
                    act_instance_id = %act.act_instance_id,
                    endpoint_id = %act.endpoint_id,
                    neural_signal_descriptor_id = %act.neural_signal_descriptor_id,
                    dispatch_binding = dispatch_binding,
                    adapter_channel_id = channel_id,
                    reason_code = %reason_code,
                    reference_id = %reference_id,
                    "act_dispatch_rejected"
                );
            }
            (
                ActDispatchResult::Rejected {
                    reason_code,
                    reference_id,
                },
                None,
            ) => {
                tracing::warn!(
                    target: "spine.dispatch",
                    act_instance_id = %act.act_instance_id,
                    endpoint_id = %act.endpoint_id,
                    neural_signal_descriptor_id = %act.neural_signal_descriptor_id,
                    dispatch_binding = dispatch_binding,
                    reason_code = %reason_code,
                    reference_id = %reference_id,
                    "act_dispatch_rejected"
                );
            }
            (
                ActDispatchResult::Lost {
                    reason_code,
                    reference_id,
                },
                Some(channel_id),
            ) => {
                tracing::warn!(
                    target: "spine.dispatch",
                    act_instance_id = %act.act_instance_id,
                    endpoint_id = %act.endpoint_id,
                    neural_signal_descriptor_id = %act.neural_signal_descriptor_id,
                    dispatch_binding = dispatch_binding,
                    adapter_channel_id = channel_id,
                    reason_code = %reason_code,
                    reference_id = %reference_id,
                    "act_dispatch_lost"
                );
            }
            (
                ActDispatchResult::Lost {
                    reason_code,
                    reference_id,
                },
                None,
            ) => {
                tracing::warn!(
                    target: "spine.dispatch",
                    act_instance_id = %act.act_instance_id,
                    endpoint_id = %act.endpoint_id,
                    neural_signal_descriptor_id = %act.neural_signal_descriptor_id,
                    dispatch_binding = dispatch_binding,
                    reason_code = %reason_code,
                    reference_id = %reference_id,
                    "act_dispatch_lost"
                );
            }
        }
    }

    fn upsert_route(
        &self,
        route: NeuralSignalDescriptorRouteKey,
        dispatch: EndpointDispatch,
    ) -> Result<(), SpineError> {
        if route.endpoint_id.trim().is_empty()
            || route.neural_signal_descriptor_id.trim().is_empty()
        {
            return Err(registration_invalid(
                "route endpoint_id/neural_signal_descriptor_id cannot be empty",
            ));
        }

        let mut routing = self.routing.write().expect("lock poisoned");
        let entry = routing
            .by_endpoint
            .entry(route.endpoint_id.clone())
            .or_insert_with(|| RegisteredEndpointRoutes {
                dispatch: dispatch.clone(),
                route_keys: BTreeSet::new(),
            });

        if !entry.dispatch.is_compatible_with(&dispatch) {
            return Err(backend_failure(format!(
                "endpoint '{}' already uses '{}' binding",
                route.endpoint_id,
                entry.dispatch.binding_label(),
            )));
        }

        entry.dispatch = dispatch;
        entry.route_keys.insert(route);
        Ok(())
    }

    fn remove_route(&self, route: &NeuralSignalDescriptorRouteKey) {
        let mut routing = self.routing.write().expect("lock poisoned");
        let mut remove_endpoint = false;

        if let Some(entry) = routing.by_endpoint.get_mut(&route.endpoint_id) {
            entry.route_keys.remove(route);
            remove_endpoint = entry.route_keys.is_empty();
        }

        if remove_endpoint {
            routing.by_endpoint.remove(&route.endpoint_id);
        }
    }
}

fn route_key_from_descriptor(
    descriptor: &NeuralSignalDescriptor,
) -> NeuralSignalDescriptorRouteKey {
    NeuralSignalDescriptorRouteKey {
        r#type: descriptor.r#type,
        endpoint_id: descriptor.endpoint_id.clone(),
        neural_signal_descriptor_id: descriptor.neural_signal_descriptor_id.clone(),
    }
}

fn endpoint_channel_or_session(dispatch: &EndpointDispatch) -> Option<String> {
    match dispatch {
        EndpointDispatch::Inline(_) => Some("inline".to_string()),
        EndpointDispatch::AdapterChannel(channel_id) => {
            Some(format!("adapter_channel:{channel_id}"))
        }
    }
}

fn dispatch_outcome_class(outcome: &ActDispatchResult) -> DispatchOutcomeClass {
    match outcome {
        ActDispatchResult::Acknowledged { .. } => DispatchOutcomeClass::Acknowledged,
        ActDispatchResult::Rejected { .. } => DispatchOutcomeClass::Rejected,
        ActDispatchResult::Lost { .. } => DispatchOutcomeClass::Lost,
    }
}

#[async_trait]
impl SpineControlPort for Spine {
    async fn publish_sense(&self, sense: Sense) {
        if let Err(err) = self.afferent_pathway.send(sense).await {
            tracing::warn!(
                target = "spine.control",
                error = %err,
                "dropping_sense_due_to_closed_afferent_pathway"
            );
        }
    }

    async fn apply_proprioception_patch(&self, entries: BTreeMap<String, String>) {
        if entries.is_empty() {
            return;
        }
        {
            let mut state = self.endpoint_proprioception.write().expect("lock poisoned");
            for (key, value) in &entries {
                state.insert(key.clone(), value.clone());
            }
        }
        self.stem_control
            .apply_proprioception_patch(ProprioceptionPatch { entries })
            .await;
    }

    async fn apply_proprioception_drop(&self, keys: Vec<String>) {
        if keys.is_empty() {
            return;
        }
        {
            let mut state = self.endpoint_proprioception.write().expect("lock poisoned");
            for key in &keys {
                state.remove(key);
            }
        }
        self.stem_control
            .apply_proprioception_drop(ProprioceptionDropPatch { keys })
            .await;
    }

    async fn refresh_topology_proprioception(&self) {
        let endpoint_ids = self.body_endpoint_ids_snapshot();
        let mut entries = BTreeMap::new();
        entries.insert(
            "spine.body_endpoint_count".to_string(),
            endpoint_ids.len().to_string(),
        );
        entries.insert("spine.body_endpoints".to_string(), endpoint_ids.join(","));
        self.apply_proprioception_patch(entries).await;
    }
}

pub async fn shutdown_global_spine(spine: Arc<Spine>) -> Result<()> {
    spine.shutdown().await;
    Ok(())
}
