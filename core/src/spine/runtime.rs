use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{
        Arc, Mutex, OnceLock, RwLock,
        atomic::{AtomicU64, Ordering},
    },
};

use anyhow::Result;
use async_trait::async_trait;
use serde_json::json;
use tokio::{sync::mpsc, task::JoinHandle};
use tokio_util::sync::CancellationToken;

use crate::{
    config::SpineRuntimeConfig,
    observability::runtime::{
        self as observability_runtime, DispatchOutcomeClass, EndpointLifecycleTransition,
    },
    spine::{
        SpineExecutionMode,
        adapters::{inline::SpineInlineAdapter, unix_socket::spawn_adapter_task},
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

pub type AdapterId = u64;

pub struct AdapterContext {
    pub adapter_id: AdapterId,
    pub shutdown: CancellationToken,
    pub act_rx: mpsc::UnboundedReceiver<Act>,
    pub sense_tx: mpsc::UnboundedSender<Sense>,
    pub port: Arc<dyn SpineAdapterPort>,
}

pub enum EndpointBinding {
    Adapter { adapter_id: AdapterId },
}

#[derive(Clone)]
enum EndpointDispatch {
    Adapter(AdapterId),
}

impl EndpointBinding {
    fn into_dispatch(self) -> EndpointDispatch {
        match self {
            EndpointBinding::Adapter { adapter_id } => EndpointDispatch::Adapter(adapter_id),
        }
    }
}

impl EndpointDispatch {
    fn adapter_id(&self) -> AdapterId {
        match self {
            EndpointDispatch::Adapter(adapter_id) => *adapter_id,
        }
    }

    fn is_compatible_with(&self, other: &Self) -> bool {
        match (self, other) {
            (EndpointDispatch::Adapter(lhs), EndpointDispatch::Adapter(rhs)) => lhs == rhs,
        }
    }

    fn binding_label(&self) -> &'static str {
        match self {
            EndpointDispatch::Adapter(_) => "adapter",
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
}

struct RegisteredEndpointRoutes {
    dispatch: EndpointDispatch,
    route_keys: BTreeSet<NeuralSignalDescriptorRouteKey>,
}

#[derive(Default)]
struct RoutingState {
    by_endpoint: BTreeMap<String, RegisteredEndpointRoutes>,
    adapters: BTreeMap<AdapterId, mpsc::UnboundedSender<Act>>,
}

pub struct Spine {
    mode: SpineExecutionMode,
    routing: RwLock<RoutingState>,
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
    async fn publish_topology_proprioception_snapshot(&self);
}

#[async_trait]
pub trait SpineAdapterPort: Send + Sync {
    async fn register_endpoint(
        &self,
        adapter_id: AdapterId,
        endpoint_name: &str,
    ) -> Result<BodyEndpointHandle>;
    async fn add_ns_descriptors(
        &self,
        body_endpoint_id: &str,
        descriptors: Vec<NeuralSignalDescriptor>,
    ) -> Result<Vec<NeuralSignalDescriptor>>;
    async fn drop_ns_descriptors(
        &self,
        body_endpoint_id: &str,
        routes: Vec<NeuralSignalDescriptorRouteKey>,
    ) -> Result<Vec<NeuralSignalDescriptorRouteKey>>;
    async fn drop_endpoint(&self, body_endpoint_id: &str);
    async fn apply_proprioception_patch(&self, entries: BTreeMap<String, String>);
    async fn apply_proprioception_drop(&self, keys: Vec<String>);
    async fn publish_topology_proprioception_snapshot(&self);
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
                    if self.inline_adapter.get().is_some() {
                        tracing::warn!(
                            target: "spine",
                            adapter_id = adapter_id,
                            "duplicate_inline_adapter_ignored"
                        );
                        continue;
                    }

                    let context = self.create_adapter_context(adapter_id);
                    let (adapter, task) =
                        SpineInlineAdapter::from_config(adapter_cfg.clone(), context);
                    if self.inline_adapter.set(Arc::clone(&adapter)).is_err() {
                        tracing::warn!(
                            target: "spine",
                            adapter_id = adapter_id,
                            "duplicate_inline_adapter_ignored"
                        );
                        continue;
                    }
                    self.tasks.lock().expect("lock poisoned").push(task);
                    adapter.emit_started();
                }
                crate::config::SpineAdapterConfig::UnixSocketNdjson {
                    config: adapter_cfg,
                } => {
                    let context = self.create_adapter_context(adapter_id);
                    let task = spawn_adapter_task(adapter_cfg.clone(), context);
                    self.tasks.lock().expect("lock poisoned").push(task);
                }
            }
        }
    }

    fn create_adapter_context(self: &Arc<Self>, adapter_id: AdapterId) -> AdapterContext {
        let (act_tx, act_rx) = mpsc::unbounded_channel::<Act>();
        let (sense_tx, mut sense_rx) = mpsc::unbounded_channel::<Sense>();

        {
            let mut routing = self.routing.write().expect("lock poisoned");
            if routing.adapters.insert(adapter_id, act_tx).is_some() {
                tracing::warn!(
                    target: "spine",
                    adapter_id = adapter_id,
                    "duplicate_adapter_dispatch_sender_replaced"
                );
            }
        }

        let spine = Arc::clone(self);
        let shutdown = self.shutdown.clone();
        let task = tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = shutdown.cancelled() => {
                        break;
                    }
                    maybe_sense = sense_rx.recv() => {
                        let Some(sense) = maybe_sense else {
                            break;
                        };
                        spine.publish_sense(sense).await;
                    }
                }
            }
            Ok::<(), anyhow::Error>(())
        });
        self.tasks.lock().expect("lock poisoned").push(task);

        let port: Arc<dyn SpineAdapterPort> = Arc::clone(self) as Arc<dyn SpineAdapterPort>;
        AdapterContext {
            adapter_id,
            shutdown: self.shutdown.clone(),
            act_rx,
            sense_tx,
            port,
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
        target = "spine.act",
        skip(self, act),
        fields(
            act_instance_id = %act.act_instance_id,
            endpoint_id = %act.endpoint_id,
            neural_signal_descriptor_id = %act.neural_signal_descriptor_id
        )
    )]
    pub async fn on_act_final(&self, tick: u64, act: Act) -> Result<ActDispatchResult, SpineError> {
        match self.dispatch_act(tick, act.clone()).await {
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
        target = "spine.act",
        skip(self, act),
        fields(
            act_instance_id = %act.act_instance_id,
            endpoint_id = %act.endpoint_id,
            neural_signal_descriptor_id = %act.neural_signal_descriptor_id
        )
    )]
    pub async fn dispatch_act(&self, tick: u64, act: Act) -> Result<ActDispatchResult, SpineError> {
        if act.act_instance_id.trim().is_empty() || act.endpoint_id.trim().is_empty() {
            tracing::warn!(
                target: "spine.act",
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
            Self::log_dispatch_outcome(tick, &act, "unknown", &outcome);
            return Ok(outcome);
        };

        match dispatch {
            EndpointDispatch::Adapter(adapter_id) => {
                observability_runtime::emit_spine_act_bind(
                    tick,
                    &act.act_instance_id,
                    Some(&act.endpoint_id),
                    Some(&act.neural_signal_descriptor_id),
                    Some("adapter"),
                    None,
                    Some(act.payload.clone()),
                );
                tracing::debug!(
                    target: "spine.act",
                    dispatch_binding = "adapter",
                    adapter_id = adapter_id,
                    "dispatching_act_to_adapter"
                );
                match self.invoke_adapter(adapter_id, act.clone()) {
                    Ok(outcome) => {
                        Self::log_dispatch_outcome(tick, &act, "adapter", &outcome);
                        Ok(outcome)
                    }
                    Err(err) => {
                        tracing::warn!(
                            target: "spine.act",
                            dispatch_binding = "adapter",
                            adapter_id = adapter_id,
                            error = %err,
                            "adapter_invoke_failed"
                        );
                        let outcome = ActDispatchResult::Lost {
                            reason_code: "dispatch_lost".to_string(),
                            reference_id: format!("spine:lost:{}", act.act_instance_id),
                        };
                        Self::log_dispatch_outcome(tick, &act, "adapter", &outcome);
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
                target: "spine.act",
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
        let adapter_id = dispatch.adapter_id();
        if !self.adapter_exists(adapter_id) {
            return Err(anyhow::anyhow!("adapter {} is not connected", adapter_id));
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
        state.by_id.insert(body_endpoint_id.clone(), registered);
        let channel_or_session = endpoint_channel_or_session(&dispatch);
        drop(state);

        observability_runtime::emit_spine_endpoint_lifecycle(
            &body_endpoint_id,
            adapter_id_from_dispatch(self, &dispatch).as_deref(),
            EndpointLifecycleTransition::Connected,
            channel_or_session,
            None,
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

        observability_runtime::emit_spine_endpoint_lifecycle(
            body_endpoint_id,
            adapter_id_from_dispatch(self, &dispatch).as_deref(),
            EndpointLifecycleTransition::Registered,
            endpoint_channel_or_session(&dispatch),
            Some(route_summary_from_routes(&accepted_routes)),
            None,
        );

        Ok(accepted_entries)
    }

    pub async fn drop_ns_descriptors(
        &self,
        body_endpoint_id: &str,
        routes: Vec<NeuralSignalDescriptorRouteKey>,
    ) -> Result<Vec<NeuralSignalDescriptorRouteKey>> {
        if routes.is_empty() {
            return Ok(Vec::new());
        }

        let normalized_routes = {
            let state = self.endpoint_state.lock().expect("lock poisoned");
            let endpoint = state
                .by_id
                .get(body_endpoint_id)
                .ok_or_else(|| anyhow::anyhow!("body endpoint is not registered"))?;
            routes
                .into_iter()
                .map(|mut route| {
                    route.endpoint_id = endpoint.body_endpoint_id.clone();
                    route
                })
                .collect::<Vec<_>>()
        };

        let drop_commit = self
            .stem_control
            .apply_neural_signal_descriptor_drop(NeuralSignalDescriptorDropPatch {
                routes: normalized_routes.clone(),
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

        if drop_commit.accepted_routes.is_empty() {
            return Ok(Vec::new());
        }

        {
            let mut state = self.endpoint_state.lock().expect("lock poisoned");
            if let Some(endpoint) = state.by_id.get_mut(body_endpoint_id) {
                for route in &drop_commit.accepted_routes {
                    endpoint.route_keys.remove(route);
                }
            }
        }
        for route in &drop_commit.accepted_routes {
            self.remove_route(route);
        }

        Ok(drop_commit.accepted_routes)
    }

    pub async fn remove_endpoint(&self, body_endpoint_id: &str) {
        let endpoint = {
            let mut state = self.endpoint_state.lock().expect("lock poisoned");
            let Some(endpoint) = state.by_id.remove(body_endpoint_id) else {
                return;
            };
            endpoint
        };
        let endpoint_routes = endpoint.route_keys.iter().cloned().collect::<Vec<_>>();
        observability_runtime::emit_spine_endpoint_lifecycle(
            body_endpoint_id,
            adapter_id_from_dispatch(self, &endpoint.dispatch).as_deref(),
            EndpointLifecycleTransition::Dropped,
            endpoint_channel_or_session(&endpoint.dispatch),
            Some(route_summary_from_routes(&endpoint_routes)),
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

    fn adapter_exists(&self, adapter_id: AdapterId) -> bool {
        self.routing
            .read()
            .expect("lock poisoned")
            .adapters
            .contains_key(&adapter_id)
    }

    fn resolve_dispatch(&self, endpoint_id: &str) -> Option<EndpointDispatch> {
        self.routing
            .read()
            .expect("lock poisoned")
            .by_endpoint
            .get(endpoint_id)
            .map(|entry| entry.dispatch.clone())
    }

    fn invoke_adapter(
        &self,
        adapter_id: AdapterId,
        act: Act,
    ) -> Result<ActDispatchResult, SpineError> {
        let tx = self
            .routing
            .read()
            .expect("lock poisoned")
            .adapters
            .get(&adapter_id)
            .cloned()
            .ok_or_else(|| backend_failure(format!("adapter {} is unavailable", adapter_id)))?;

        if tx.send(act.clone()).is_err() {
            tracing::warn!(
                target: "spine.act",
                dispatch_binding = "adapter",
                adapter_id = adapter_id,
                act_instance_id = %act.act_instance_id,
                "adapter_send_failed"
            );
            return Err(backend_failure(format!(
                "failed to dispatch act {} to adapter {}",
                act.act_instance_id, adapter_id
            )));
        }

        tracing::debug!(
            target: "spine.act",
            dispatch_binding = "adapter",
            adapter_id = adapter_id,
            act_instance_id = %act.act_instance_id,
            "act_enqueued_to_adapter"
        );
        Ok(ActDispatchResult::Acknowledged {
            reference_id: format!("adapter:act_sent:{}", act.act_instance_id),
        })
    }

    fn log_dispatch_outcome(
        tick: u64,
        act: &Act,
        dispatch_binding: &'static str,
        outcome: &ActDispatchResult,
    ) {
        observability_runtime::emit_spine_act_outcome(
            tick,
            &act.act_instance_id,
            Some(&act.endpoint_id),
            Some(&act.neural_signal_descriptor_id),
            Some(dispatch_binding),
            None,
            Some(act.payload.clone()),
            dispatch_outcome_class(outcome),
            dispatch_reason_or_reference(outcome),
        );
        match outcome {
            ActDispatchResult::Acknowledged { reference_id } => {
                tracing::info!(
                    target: "spine.act",
                    act_instance_id = %act.act_instance_id,
                    endpoint_id = %act.endpoint_id,
                    neural_signal_descriptor_id = %act.neural_signal_descriptor_id,
                    dispatch_binding = dispatch_binding,
                    reference_id = %reference_id,
                    "act_dispatch_acknowledged"
                );
            }
            ActDispatchResult::Rejected {
                reason_code,
                reference_id,
            } => {
                tracing::warn!(
                    target: "spine.act",
                    act_instance_id = %act.act_instance_id,
                    endpoint_id = %act.endpoint_id,
                    neural_signal_descriptor_id = %act.neural_signal_descriptor_id,
                    dispatch_binding = dispatch_binding,
                    reason_code = %reason_code,
                    reference_id = %reference_id,
                    "act_dispatch_rejected"
                );
            }
            ActDispatchResult::Lost {
                reason_code,
                reference_id,
            } => {
                tracing::warn!(
                    target: "spine.act",
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
        EndpointDispatch::Adapter(adapter_id) => Some(format!("adapter:{adapter_id}")),
    }
}

fn adapter_id_from_dispatch(_spine: &Spine, dispatch: &EndpointDispatch) -> Option<String> {
    match dispatch {
        EndpointDispatch::Adapter(adapter_id) => Some(adapter_id.to_string()),
    }
}

fn route_summary_from_routes(routes: &[NeuralSignalDescriptorRouteKey]) -> serde_json::Value {
    json!({
        "routes": routes
            .iter()
            .map(|route| {
                json!({
                    "type": route.r#type,
                    "endpoint_id": route.endpoint_id,
                    "descriptor_id": route.neural_signal_descriptor_id,
                })
            })
            .collect::<Vec<_>>(),
    })
}

fn dispatch_outcome_class(outcome: &ActDispatchResult) -> DispatchOutcomeClass {
    match outcome {
        ActDispatchResult::Acknowledged { .. } => DispatchOutcomeClass::Acknowledged,
        ActDispatchResult::Rejected { .. } => DispatchOutcomeClass::Rejected,
        ActDispatchResult::Lost { .. } => DispatchOutcomeClass::Lost,
    }
}

fn dispatch_reason_or_reference(outcome: &ActDispatchResult) -> Option<serde_json::Value> {
    match outcome {
        ActDispatchResult::Acknowledged { reference_id } => Some(json!({
            "reference_id": reference_id,
        })),
        ActDispatchResult::Rejected {
            reason_code,
            reference_id,
        }
        | ActDispatchResult::Lost {
            reason_code,
            reference_id,
        } => Some(json!({
            "reason_code": reason_code,
            "reference_id": reference_id,
        })),
    }
}

#[async_trait]
impl SpineControlPort for Spine {
    async fn publish_sense(&self, sense: Sense) {
        observability_runtime::emit_spine_sense_ingress(
            0,
            &sense.endpoint_id,
            Some(&sense.neural_signal_descriptor_id),
            &sense.sense_instance_id,
            json!(sense.payload.clone()),
            None,
        );

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

    async fn publish_topology_proprioception_snapshot(&self) {
        let endpoint_ids = self.body_endpoint_ids_snapshot();
        let mut entries = BTreeMap::new();
        entries.insert(
            "spine.body_endpoint_count".to_string(),
            endpoint_ids.len().to_string(),
        );
        entries.insert("spine.body_endpoints".to_string(), endpoint_ids.join(","));
        <Self as SpineControlPort>::apply_proprioception_patch(self, entries).await;
    }
}

#[async_trait]
impl SpineAdapterPort for Spine {
    async fn register_endpoint(
        &self,
        adapter_id: AdapterId,
        endpoint_name: &str,
    ) -> Result<BodyEndpointHandle> {
        self.add_endpoint(endpoint_name, EndpointBinding::Adapter { adapter_id })
    }

    async fn add_ns_descriptors(
        &self,
        body_endpoint_id: &str,
        descriptors: Vec<NeuralSignalDescriptor>,
    ) -> Result<Vec<NeuralSignalDescriptor>> {
        self.add_ns_descriptors(body_endpoint_id, descriptors).await
    }

    async fn drop_ns_descriptors(
        &self,
        body_endpoint_id: &str,
        routes: Vec<NeuralSignalDescriptorRouteKey>,
    ) -> Result<Vec<NeuralSignalDescriptorRouteKey>> {
        self.drop_ns_descriptors(body_endpoint_id, routes).await
    }

    async fn drop_endpoint(&self, body_endpoint_id: &str) {
        self.remove_endpoint(body_endpoint_id).await;
    }

    async fn apply_proprioception_patch(&self, entries: BTreeMap<String, String>) {
        <Self as SpineControlPort>::apply_proprioception_patch(self, entries).await;
    }

    async fn apply_proprioception_drop(&self, keys: Vec<String>) {
        <Self as SpineControlPort>::apply_proprioception_drop(self, keys).await;
    }

    async fn publish_topology_proprioception_snapshot(&self) {
        <Self as SpineControlPort>::publish_topology_proprioception_snapshot(self).await;
    }
}

pub async fn shutdown_global_spine(spine: Arc<Spine>) -> Result<()> {
    spine.shutdown().await;
    Ok(())
}
