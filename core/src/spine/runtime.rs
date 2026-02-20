use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{
        Arc, Mutex, OnceLock, RwLock,
        atomic::{AtomicU64, Ordering},
    },
};

use anyhow::Result;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tracing::Instrument;

use crate::{
    afferent_pathway::SenseAfferentPathway,
    config::SpineRuntimeConfig,
    spine::{
        SpineExecutionMode,
        adapters::{inline::SpineInlineAdapter, unix_socket::UnixSocketAdapter},
        endpoint::Endpoint,
        error::{SpineError, backend_failure, invalid_batch, registration_invalid},
        types::{ActDispatchResult, NeuralSignalDescriptor, NeuralSignalDescriptorRouteKey},
    },
    types::{Act, NeuralSignalDescriptorCatalog},
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
    descriptors: BTreeMap<NeuralSignalDescriptorRouteKey, NeuralSignalDescriptor>,
}

#[derive(Default)]
struct RoutingState {
    version: u64,
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
}

impl Spine {
    pub fn new(config: &SpineRuntimeConfig, afferent_pathway: SenseAfferentPathway) -> Arc<Self> {
        let spine = Arc::new(Self {
            mode: SpineExecutionMode::SerializedDeterministic,
            routing: RwLock::new(RoutingState::default()),
            next_adapter_channel_seq: AtomicU64::new(0),
            next_body_endpoint_seq: AtomicU64::new(0),
            shutdown: CancellationToken::new(),
            tasks: Mutex::new(Vec::new()),
            endpoint_state: Mutex::new(EndpointState::default()),
            inline_adapter: OnceLock::new(),
        });

        spine.start_adapters(config, afferent_pathway);
        spine
    }

    fn start_adapters(
        self: &Arc<Self>,
        config: &SpineRuntimeConfig,
        afferent_pathway: SenseAfferentPathway,
    ) {
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
                        afferent_pathway.clone(),
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
                }
                crate::config::SpineAdapterConfig::UnixSocketNdjson {
                    config: adapter_cfg,
                } => {
                    let adapter =
                        UnixSocketAdapter::new(adapter_cfg.socket_path.clone(), adapter_id);
                    let afferent_pathway = afferent_pathway.clone();
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
                            adapter.run(afferent_pathway, spine, shutdown).await
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

    pub fn neural_signal_descriptor_catalog_snapshot(&self) -> NeuralSignalDescriptorCatalog {
        self.catalog_snapshot()
    }

    pub fn inline_adapter(&self) -> Option<Arc<SpineInlineAdapter>> {
        self.inline_adapter.get().cloned()
    }

    #[tracing::instrument(
        name = "spine_dispatch_act",
        target = "spine.dispatch",
        skip(self, act),
        fields(
            act_id = %act.act_id,
            endpoint_id = %act.endpoint_id,
            neural_signal_descriptor_id = %act.neural_signal_descriptor_id
        )
    )]
    pub async fn dispatch_act(&self, act: Act) -> Result<ActDispatchResult, SpineError> {
        if act.act_id.trim().is_empty() || act.endpoint_id.trim().is_empty() {
            tracing::warn!(
                target: "spine.dispatch",
                "act_dispatch_invalid_input"
            );
            return Err(invalid_batch("act dispatch is missing act_id/endpoint_id"));
        }

        let Some(dispatch) = self.resolve_dispatch(&act.endpoint_id) else {
            let outcome = ActDispatchResult::Rejected {
                reason_code: "endpoint_not_found".to_string(),
                reference_id: format!("spine:missing_endpoint:{}", act.act_id),
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
                            reference_id: format!("spine:error:{}", act.act_id),
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
                        let outcome = ActDispatchResult::Rejected {
                            reason_code: "endpoint_error".to_string(),
                            reference_id: format!("spine:error:{}", act.act_id),
                        };
                        Self::log_dispatch_outcome(&act, "adapter", Some(channel_id), &outcome);
                        Ok(outcome)
                    }
                }
            }
        }
    }

    pub fn add_endpoint(
        &self,
        endpoint_name: &str,
        binding: EndpointBinding,
        descriptors: Vec<NeuralSignalDescriptor>,
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
        let mut route_keys = BTreeSet::new();

        for mut descriptor in descriptors {
            descriptor.endpoint_id = body_endpoint_id.clone();
            self.upsert_route(descriptor.clone(), dispatch.clone())
                .map_err(|err| anyhow::anyhow!(err.to_string()))?;
            route_keys.insert(NeuralSignalDescriptorRouteKey {
                r#type: descriptor.r#type,
                endpoint_id: descriptor.endpoint_id.clone(),
                neural_signal_descriptor_id: descriptor.neural_signal_descriptor_id.clone(),
            });
        }

        let registered = RegisteredBodyEndpoint {
            body_endpoint_id: body_endpoint_id.clone(),
            dispatch: dispatch.clone(),
            route_keys,
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

        Ok(BodyEndpointHandle { body_endpoint_id })
    }

    pub fn add_capabilities(
        &self,
        body_endpoint_id: &str,
        descriptors: Vec<NeuralSignalDescriptor>,
    ) -> Result<Vec<NeuralSignalDescriptor>> {
        let mut state = self.endpoint_state.lock().expect("lock poisoned");
        let endpoint = state
            .by_id
            .get_mut(body_endpoint_id)
            .ok_or_else(|| anyhow::anyhow!("body endpoint is not registered"))?;

        let mut registered = Vec::new();
        for mut descriptor in descriptors {
            descriptor.endpoint_id = endpoint.body_endpoint_id.clone();
            self.upsert_route(descriptor.clone(), endpoint.dispatch.clone())
                .map_err(|err| anyhow::anyhow!(err.to_string()))?;
            endpoint.route_keys.insert(NeuralSignalDescriptorRouteKey {
                r#type: descriptor.r#type,
                endpoint_id: descriptor.endpoint_id.clone(),
                neural_signal_descriptor_id: descriptor.neural_signal_descriptor_id.clone(),
            });
            registered.push(descriptor);
        }

        Ok(registered)
    }

    pub fn remove_endpoint(&self, body_endpoint_id: &str) -> Vec<NeuralSignalDescriptorRouteKey> {
        let mut state = self.endpoint_state.lock().expect("lock poisoned");
        let Some(endpoint) = state.by_id.remove(body_endpoint_id) else {
            return Vec::new();
        };

        if let Some(channel_id) = endpoint.dispatch.channel_id()
            && let Some(ids) = state.by_channel.get_mut(&channel_id)
        {
            ids.remove(body_endpoint_id);
            if ids.is_empty() {
                state.by_channel.remove(&channel_id);
            }
        }

        let mut dropped = Vec::new();
        for route in endpoint.route_keys {
            if self.remove_route(&route).is_some() {
                dropped.push(route);
            }
        }
        dropped
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

    pub(crate) fn on_adapter_channel_closed(
        &self,
        channel_id: u64,
    ) -> Vec<NeuralSignalDescriptorRouteKey> {
        {
            let mut routing = self.routing.write().expect("lock poisoned");
            routing.adapter_channels.remove(&channel_id);
        }

        let endpoint_ids = {
            let mut state = self.endpoint_state.lock().expect("lock poisoned");
            state.by_channel.remove(&channel_id).unwrap_or_default()
        };

        let mut dropped = Vec::new();
        for endpoint_id in endpoint_ids {
            dropped.extend(self.remove_endpoint(&endpoint_id));
        }
        dropped
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

    fn catalog_snapshot(&self) -> NeuralSignalDescriptorCatalog {
        let routing = self.routing.read().expect("lock poisoned");
        let mut entries: Vec<_> = routing
            .by_endpoint
            .values()
            .flat_map(|item| item.descriptors.values().cloned())
            .collect();
        entries.sort_by(|lhs, rhs| {
            lhs.r#type
                .cmp(&rhs.r#type)
                .then_with(|| lhs.endpoint_id.cmp(&rhs.endpoint_id))
                .then_with(|| {
                    lhs.neural_signal_descriptor_id
                        .cmp(&rhs.neural_signal_descriptor_id)
                })
        });

        NeuralSignalDescriptorCatalog {
            version: format!("spine:v{}", routing.version),
            entries,
        }
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
                act_id = %act.act_id,
                "adapter_channel_send_failed"
            );
            return Err(backend_failure(format!(
                "failed to dispatch act {} to adapter channel {}",
                act.act_id, channel_id
            )));
        }

        tracing::debug!(
            target: "spine.dispatch",
            dispatch_binding = "adapter",
            adapter_channel_id = channel_id,
            act_id = %act.act_id,
            "act_enqueued_to_adapter_channel"
        );
        Ok(ActDispatchResult::Acknowledged {
            reference_id: format!("adapter:act_sent:{}", act.act_id),
        })
    }

    fn log_dispatch_outcome(
        act: &Act,
        dispatch_binding: &'static str,
        channel_id: Option<u64>,
        outcome: &ActDispatchResult,
    ) {
        match (outcome, channel_id) {
            (ActDispatchResult::Acknowledged { reference_id }, Some(channel_id)) => {
                tracing::info!(
                    target: "spine.dispatch",
                    act_id = %act.act_id,
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
                    act_id = %act.act_id,
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
                    act_id = %act.act_id,
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
                    act_id = %act.act_id,
                    endpoint_id = %act.endpoint_id,
                    neural_signal_descriptor_id = %act.neural_signal_descriptor_id,
                    dispatch_binding = dispatch_binding,
                    reason_code = %reason_code,
                    reference_id = %reference_id,
                    "act_dispatch_rejected"
                );
            }
        }
    }

    fn upsert_route(
        &self,
        descriptor: NeuralSignalDescriptor,
        dispatch: EndpointDispatch,
    ) -> Result<(), SpineError> {
        if descriptor.endpoint_id.trim().is_empty()
            || descriptor.neural_signal_descriptor_id.trim().is_empty()
        {
            return Err(registration_invalid(
                "route endpoint_id/neural_signal_descriptor_id cannot be empty",
            ));
        }

        let route = NeuralSignalDescriptorRouteKey {
            r#type: descriptor.r#type,
            endpoint_id: descriptor.endpoint_id.clone(),
            neural_signal_descriptor_id: descriptor.neural_signal_descriptor_id.clone(),
        };

        let mut routing = self.routing.write().expect("lock poisoned");
        let entry = routing
            .by_endpoint
            .entry(route.endpoint_id.clone())
            .or_insert_with(|| RegisteredEndpointRoutes {
                dispatch: dispatch.clone(),
                descriptors: BTreeMap::new(),
            });

        if !entry.dispatch.is_compatible_with(&dispatch) {
            return Err(backend_failure(format!(
                "endpoint '{}' already uses '{}' binding",
                route.endpoint_id,
                entry.dispatch.binding_label(),
            )));
        }

        entry.dispatch = dispatch;
        entry.descriptors.insert(route, descriptor);
        routing.version = routing.version.saturating_add(1);
        Ok(())
    }

    fn remove_route(
        &self,
        route: &NeuralSignalDescriptorRouteKey,
    ) -> Option<NeuralSignalDescriptor> {
        let mut routing = self.routing.write().expect("lock poisoned");
        let mut removed = None;
        let mut remove_endpoint = false;

        if let Some(entry) = routing.by_endpoint.get_mut(&route.endpoint_id) {
            removed = entry.descriptors.remove(route);
            remove_endpoint = entry.descriptors.is_empty();
        }

        if remove_endpoint {
            routing.by_endpoint.remove(&route.endpoint_id);
        }

        if removed.is_some() {
            routing.version = routing.version.saturating_add(1);
        }

        removed
    }
}

pub async fn shutdown_global_spine(spine: Arc<Spine>) -> Result<()> {
    spine.shutdown().await;
    Ok(())
}
