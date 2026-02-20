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
use uuid::Uuid;

use crate::{
    afferent_pathway::SenseAfferentPathway,
    config::SpineRuntimeConfig,
    spine::{
        SpineExecutionMode,
        adapters::{inline::SpineInlineAdapter, unix_socket::UnixSocketAdapter},
        endpoint::Endpoint,
        error::{SpineError, backend_failure, invalid_batch, registration_invalid},
        types::{
            ActDispatchResult, EndpointCapabilityDescriptor, RouteKey, SpineCapabilityCatalog,
        },
    },
    types::Act,
};

#[derive(Debug, Clone)]
pub struct BodyEndpointHandle {
    pub body_endpoint_id: Uuid,
    pub body_endpoint_name: String,
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
    body_endpoint_id: Uuid,
    body_endpoint_name: String,
    dispatch: EndpointDispatch,
    route_keys: BTreeSet<RouteKey>,
}

#[derive(Default)]
struct EndpointState {
    by_id: BTreeMap<Uuid, RegisteredBodyEndpoint>,
    by_name: BTreeMap<String, Uuid>,
    by_channel: BTreeMap<u64, BTreeSet<Uuid>>,
}

struct RegisteredEndpointRoutes {
    dispatch: EndpointDispatch,
    descriptors: BTreeMap<String, EndpointCapabilityDescriptor>,
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

    pub fn capability_catalog_snapshot(&self) -> SpineCapabilityCatalog {
        self.catalog_snapshot()
    }

    pub fn inline_adapter(&self) -> Option<Arc<SpineInlineAdapter>> {
        self.inline_adapter.get().cloned()
    }

    pub async fn dispatch_act(&self, act: Act) -> Result<ActDispatchResult, SpineError> {
        if act.act_id.trim().is_empty() || act.body_endpoint_name.trim().is_empty() {
            return Err(invalid_batch("act dispatch is missing act_id/endpoint_id"));
        }

        let Some(dispatch) = self.resolve_dispatch(&act.body_endpoint_name) else {
            return Ok(ActDispatchResult::Rejected {
                reason_code: "endpoint_not_found".to_string(),
                reference_id: format!("spine:missing_endpoint:{}", act.act_id),
            });
        };

        match dispatch {
            EndpointDispatch::Inline(endpoint) => match endpoint.invoke(act.clone()).await {
                Ok(outcome) => Ok(outcome),
                Err(_) => Ok(ActDispatchResult::Rejected {
                    reason_code: "endpoint_error".to_string(),
                    reference_id: format!("spine:error:{}", act.act_id),
                }),
            },
            EndpointDispatch::AdapterChannel(channel_id) => {
                match self.invoke_adapter_endpoint(channel_id, act.clone()) {
                    Ok(outcome) => Ok(outcome),
                    Err(_) => Ok(ActDispatchResult::Rejected {
                        reason_code: "endpoint_error".to_string(),
                        reference_id: format!("spine:error:{}", act.act_id),
                    }),
                }
            }
        }
    }

    pub fn add_endpoint(
        &self,
        endpoint_name: &str,
        binding: EndpointBinding,
        descriptors: Vec<EndpointCapabilityDescriptor>,
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

        let mut state = self.endpoint_state.lock().expect("lock poisoned");
        if state.by_name.contains_key(endpoint_name) {
            return Err(anyhow::anyhow!(
                "endpoint name is already registered: {}",
                endpoint_name
            ));
        }

        let body_endpoint_name = endpoint_name.to_string();
        let body_endpoint_id = Uuid::now_v7();
        let mut route_keys = BTreeSet::new();

        for mut descriptor in descriptors {
            descriptor.route.endpoint_id = body_endpoint_name.clone();
            self.upsert_route(descriptor.clone(), dispatch.clone())
                .map_err(|err| anyhow::anyhow!(err.to_string()))?;
            route_keys.insert(descriptor.route);
        }

        let registered = RegisteredBodyEndpoint {
            body_endpoint_id,
            body_endpoint_name: body_endpoint_name.clone(),
            dispatch: dispatch.clone(),
            route_keys,
        };

        state
            .by_name
            .insert(body_endpoint_name.clone(), body_endpoint_id);
        if let Some(channel_id) = dispatch.channel_id() {
            state
                .by_channel
                .entry(channel_id)
                .or_default()
                .insert(body_endpoint_id);
        }
        state.by_id.insert(body_endpoint_id, registered);

        Ok(BodyEndpointHandle {
            body_endpoint_id,
            body_endpoint_name,
        })
    }

    pub fn add_capabilities(
        &self,
        body_endpoint_id: Uuid,
        descriptors: Vec<EndpointCapabilityDescriptor>,
    ) -> Result<Vec<EndpointCapabilityDescriptor>> {
        let mut state = self.endpoint_state.lock().expect("lock poisoned");
        let endpoint = state
            .by_id
            .get_mut(&body_endpoint_id)
            .ok_or_else(|| anyhow::anyhow!("body endpoint is not registered"))?;

        let mut registered = Vec::new();
        for mut descriptor in descriptors {
            descriptor.route.endpoint_id = endpoint.body_endpoint_name.clone();
            self.upsert_route(descriptor.clone(), endpoint.dispatch.clone())
                .map_err(|err| anyhow::anyhow!(err.to_string()))?;
            endpoint.route_keys.insert(descriptor.route.clone());
            registered.push(descriptor);
        }

        Ok(registered)
    }

    pub fn remove_capabilities(
        &self,
        body_endpoint_id: Uuid,
        capability_ids: &[String],
    ) -> Vec<RouteKey> {
        let mut state = self.endpoint_state.lock().expect("lock poisoned");
        let Some(endpoint) = state.by_id.get_mut(&body_endpoint_id) else {
            return Vec::new();
        };

        let mut removed = Vec::new();
        for capability_id in capability_ids {
            let route = RouteKey {
                endpoint_id: endpoint.body_endpoint_name.clone(),
                capability_id: capability_id.clone(),
            };

            if self.remove_route(&route).is_some() {
                endpoint.route_keys.remove(&route);
                removed.push(route);
            }
        }
        removed
    }

    pub fn remove_endpoint(&self, body_endpoint_id: Uuid) -> Vec<RouteKey> {
        let mut state = self.endpoint_state.lock().expect("lock poisoned");
        let Some(endpoint) = state.by_id.remove(&body_endpoint_id) else {
            return Vec::new();
        };

        state.by_name.remove(&endpoint.body_endpoint_name);
        if let Some(channel_id) = endpoint.dispatch.channel_id()
            && let Some(ids) = state.by_channel.get_mut(&channel_id)
        {
            ids.remove(&body_endpoint_id);
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

    pub(crate) fn on_adapter_channel_closed(&self, channel_id: u64) -> Vec<RouteKey> {
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
            dropped.extend(self.remove_endpoint(endpoint_id));
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

    fn catalog_snapshot(&self) -> SpineCapabilityCatalog {
        let routing = self.routing.read().expect("lock poisoned");
        let mut entries: Vec<_> = routing
            .by_endpoint
            .values()
            .flat_map(|item| item.descriptors.values().cloned())
            .collect();
        entries.sort_by(|lhs, rhs| lhs.route.cmp(&rhs.route));

        SpineCapabilityCatalog {
            version: routing.version,
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
            return Err(backend_failure(format!(
                "failed to dispatch act {} to adapter channel {}",
                act.act_id, channel_id
            )));
        }

        Ok(ActDispatchResult::Acknowledged {
            reference_id: format!("adapter:act_sent:{}", act.act_id),
        })
    }

    fn upsert_route(
        &self,
        descriptor: EndpointCapabilityDescriptor,
        dispatch: EndpointDispatch,
    ) -> Result<(), SpineError> {
        let route = &descriptor.route;
        if route.endpoint_id.trim().is_empty() || route.capability_id.trim().is_empty() {
            return Err(registration_invalid(
                "route endpoint_id/capability_id cannot be empty",
            ));
        }

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
        entry
            .descriptors
            .insert(route.capability_id.clone(), descriptor);
        routing.version = routing.version.saturating_add(1);
        Ok(())
    }

    fn remove_route(&self, route: &RouteKey) -> Option<EndpointCapabilityDescriptor> {
        let mut routing = self.routing.write().expect("lock poisoned");
        let mut removed = None;
        let mut remove_endpoint = false;

        if let Some(entry) = routing.by_endpoint.get_mut(&route.endpoint_id) {
            removed = entry.descriptors.remove(&route.capability_id);
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

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use tokio::sync::mpsc;

    use crate::{
        config::SpineRuntimeConfig,
        spine::{
            endpoint::Endpoint,
            error::internal_error,
            types::{ActDispatchResult, CostVector},
        },
        types::RequestedResources,
    };

    struct StubInlineEndpoint;

    #[async_trait]
    impl Endpoint for StubInlineEndpoint {
        async fn invoke(&self, _act: Act) -> std::result::Result<ActDispatchResult, SpineError> {
            Ok(ActDispatchResult::Acknowledged {
                reference_id: "stub:inline".to_string(),
            })
        }
    }

    fn descriptor(capability_id: &str) -> EndpointCapabilityDescriptor {
        EndpointCapabilityDescriptor {
            route: RouteKey {
                endpoint_id: "placeholder".to_string(),
                capability_id: capability_id.to_string(),
            },
            payload_schema: serde_json::json!({"type":"object"}),
            max_payload_bytes: 1024,
            default_cost: CostVector {
                survival_micro: 1,
                time_ms: 1,
                io_units: 1,
                token_units: 1,
            },
            metadata: Default::default(),
        }
    }

    fn test_spine() -> Arc<Spine> {
        let cfg = SpineRuntimeConfig { adapters: vec![] };
        Spine::new(&cfg, SenseAfferentPathway::new(4).0)
    }

    #[test]
    fn rejects_duplicate_endpoint_names() {
        let spine = test_spine();

        spine
            .add_endpoint(
                "cli",
                EndpointBinding::Inline(Arc::new(StubInlineEndpoint)),
                vec![],
            )
            .expect("first endpoint should register");
        let err = spine
            .add_endpoint(
                "cli",
                EndpointBinding::Inline(Arc::new(StubInlineEndpoint)),
                vec![],
            )
            .expect_err("duplicate endpoint name should fail");

        assert!(err.to_string().contains("already registered"));
    }

    #[test]
    fn remove_endpoint_returns_registered_routes() {
        let spine = test_spine();
        let (tx, _rx) = mpsc::unbounded_channel();
        let channel_id = spine.on_adapter_channel_open(2, tx);
        let endpoint = spine
            .add_endpoint(
                "macos-app",
                EndpointBinding::AdapterChannel(channel_id),
                vec![],
            )
            .expect("endpoint");

        let registered = spine
            .add_capabilities(
                endpoint.body_endpoint_id,
                vec![descriptor("present.message")],
            )
            .expect("register capability");
        assert_eq!(registered[0].route.endpoint_id, "macos-app");

        let dropped = spine.remove_endpoint(endpoint.body_endpoint_id);
        assert_eq!(dropped.len(), 1);
        assert_eq!(dropped[0].endpoint_id, "macos-app");
        assert_eq!(dropped[0].capability_id, "present.message");
    }

    #[test]
    fn inline_endpoint_registration_is_spine_managed_and_not_channel_owned() {
        let spine = test_spine();

        let endpoint = spine
            .add_endpoint(
                "std-shell",
                EndpointBinding::Inline(Arc::new(StubInlineEndpoint)),
                vec![descriptor("tool.shell.exec")],
            )
            .expect("register inline endpoint");

        let state = spine.endpoint_state.lock().expect("lock poisoned");
        let ep = state
            .by_id
            .get(&endpoint.body_endpoint_id)
            .expect("endpoint state");
        assert!(ep.dispatch.channel_id().is_none());
        assert!(state.by_channel.is_empty());
    }

    #[test]
    fn dispatch_routes_by_endpoint_name() {
        let spine = test_spine();
        let handle = spine
            .add_endpoint(
                "std-web",
                EndpointBinding::Inline(Arc::new(StubInlineEndpoint)),
                vec![descriptor("tool.web.fetch")],
            )
            .expect("register inline endpoint");

        let runtime = tokio::runtime::Runtime::new().expect("runtime");
        let outcome = runtime
            .block_on(spine.dispatch_act(Act {
                act_id: "act:1".to_string(),
                based_on: vec![],
                body_endpoint_name: handle.body_endpoint_name,
                capability_id: "tool.web.fetch".to_string(),
                capability_instance_id: "cap-inst-1".to_string(),
                normalized_payload: serde_json::json!({"url":"https://example.com"}),
                requested_resources: RequestedResources {
                    survival_micro: 1,
                    time_ms: 1,
                    io_units: 1,
                    token_units: 1,
                },
            }))
            .expect("dispatch by endpoint name");

        assert!(matches!(outcome, ActDispatchResult::Acknowledged { .. }));
    }

    #[test]
    fn mode_is_serialized_deterministic() {
        let spine = test_spine();
        assert_eq!(spine.mode(), SpineExecutionMode::SerializedDeterministic);
    }

    #[tokio::test]
    async fn missing_endpoint_is_mapped_to_endpoint_not_found_rejection() {
        let spine = test_spine();

        let outcome = spine
            .dispatch_act(Act {
                act_id: "act:1".to_string(),
                based_on: vec!["sense:1".to_string()],
                body_endpoint_name: "missing.endpoint".to_string(),
                capability_id: "missing.capability".to_string(),
                capability_instance_id: "instance:1".to_string(),
                normalized_payload: serde_json::json!({"ok":true}),
                requested_resources: RequestedResources::default(),
            })
            .await
            .expect("execution should succeed with per-act rejection");

        assert!(matches!(
            outcome,
            ActDispatchResult::Rejected { ref reason_code, .. } if reason_code == "endpoint_not_found"
        ));
    }

    struct FailingEndpoint;

    #[async_trait]
    impl Endpoint for FailingEndpoint {
        async fn invoke(&self, _act: Act) -> std::result::Result<ActDispatchResult, SpineError> {
            Err(internal_error("endpoint exploded"))
        }
    }

    #[test]
    fn endpoint_error_is_mapped_to_endpoint_error_rejection() {
        let spine = test_spine();
        let handle = spine
            .add_endpoint(
                "core-mind",
                EndpointBinding::Inline(Arc::new(FailingEndpoint)),
                vec![EndpointCapabilityDescriptor {
                    route: RouteKey {
                        endpoint_id: "placeholder".to_string(),
                        capability_id: "observe.state".to_string(),
                    },
                    payload_schema: serde_json::json!({"type":"object"}),
                    max_payload_bytes: 1024,
                    default_cost: CostVector::default(),
                    metadata: Default::default(),
                }],
            )
            .expect("registration should succeed");

        let runtime = tokio::runtime::Runtime::new().expect("runtime");
        let outcome = runtime
            .block_on(spine.dispatch_act(Act {
                act_id: "act:1".to_string(),
                based_on: vec!["sense:1".to_string()],
                body_endpoint_name: handle.body_endpoint_name,
                capability_id: "observe.state".to_string(),
                capability_instance_id: "instance:1".to_string(),
                normalized_payload: serde_json::json!({"ok":true}),
                requested_resources: RequestedResources::default(),
            }))
            .expect("execution should succeed with per-act rejection");

        assert!(matches!(
            outcome,
            ActDispatchResult::Rejected { ref reason_code, .. } if reason_code == "endpoint_error"
        ));
    }

    #[test]
    fn adapter_channel_id_encodes_adapter_identity() {
        let spine = test_spine();
        let (tx1, _rx1) = mpsc::unbounded_channel();
        let (tx2, _rx2) = mpsc::unbounded_channel();

        let a1 = spine.on_adapter_channel_open(1, tx1);
        let a2 = spine.on_adapter_channel_open(2, tx2);

        assert_eq!(a1 >> 32, 1);
        assert_eq!(a2 >> 32, 2);
        assert!(a2 > a1);
    }
}
