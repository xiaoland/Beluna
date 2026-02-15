use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use anyhow::{Context, Result};
use tokio::{sync::Mutex, task::JoinHandle};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::{
    config::SpineRuntimeConfig,
    ingress::SenseIngress,
    spine::{
        EndpointRegistryPort, RoutingSpineExecutor, SpineExecutionMode, SpineExecutorPort,
        adapters::unix_socket::UnixSocketAdapter,
        ports::EndpointPort,
        registry::InMemoryEndpointRegistry,
        types::{EndpointCapabilityDescriptor, RouteKey},
    },
};

#[derive(Debug, Clone)]
pub struct BodyEndpointHandle {
    pub body_endpoint_id: Uuid,
    pub body_endpoint_name: String,
}

#[derive(Debug, Clone)]
struct RegisteredBodyEndpoint {
    body_endpoint_id: Uuid,
    body_endpoint_name: String,
    channel_id: Option<u64>,
    route_keys: BTreeSet<RouteKey>,
}

#[derive(Default)]
struct EndpointState {
    name_seq: BTreeMap<String, u64>,
    by_id: BTreeMap<Uuid, RegisteredBodyEndpoint>,
    by_name: BTreeMap<String, Uuid>,
    by_channel: BTreeMap<u64, BTreeSet<Uuid>>,
}

pub struct Spine {
    registry: Arc<InMemoryEndpointRegistry>,
    executor: Arc<dyn SpineExecutorPort>,
    shutdown: CancellationToken,
    tasks: Mutex<Vec<JoinHandle<Result<()>>>>,
    endpoint_state: Mutex<EndpointState>,
}

impl Spine {
    pub fn new(config: &SpineRuntimeConfig, ingress: SenseIngress) -> Arc<Self> {
        let registry = Arc::new(InMemoryEndpointRegistry::new());
        let registry_port: Arc<dyn EndpointRegistryPort> = registry.clone();
        let executor: Arc<dyn SpineExecutorPort> = Arc::new(RoutingSpineExecutor::new(
            SpineExecutionMode::SerializedDeterministic,
            Arc::clone(&registry_port),
        ));

        let spine = Arc::new(Self {
            registry,
            executor,
            shutdown: CancellationToken::new(),
            tasks: Mutex::new(Vec::new()),
            endpoint_state: Mutex::new(EndpointState::default()),
        });

        spine.start_adapters(config, ingress);
        spine
    }

    fn start_adapters(self: &Arc<Self>, config: &SpineRuntimeConfig, ingress: SenseIngress) {
        for (index, adapter_config) in config.adapters.iter().enumerate() {
            let adapter_id = (index as u64) + 1;
            match adapter_config {
                crate::config::SpineAdapterConfig::UnixSocketNdjson {
                    config: adapter_cfg,
                } => {
                    let adapter =
                        UnixSocketAdapter::new(adapter_cfg.socket_path.clone(), adapter_id);
                    let ingress = ingress.clone();
                    let spine = Arc::clone(self);
                    let shutdown = self.shutdown.clone();
                    let socket_path = adapter_cfg.socket_path.clone();

                    let task = tokio::spawn(async move {
                        eprintln!(
                            "[spine] adapter_started type=unix-socket-ndjson adapter_id={} socket_path={}",
                            adapter_id,
                            socket_path.display()
                        );
                        adapter.run(ingress, spine, shutdown).await
                    });
                    self.tasks.blocking_lock().push(task);
                }
            }
        }
    }

    pub fn registry_port(&self) -> Arc<dyn EndpointRegistryPort> {
        self.registry.clone()
    }

    pub fn executor_port(&self) -> Arc<dyn SpineExecutorPort> {
        Arc::clone(&self.executor)
    }

    pub(crate) fn on_adapter_channel_open(
        &self,
        adapter_id: u64,
        tx: tokio::sync::mpsc::UnboundedSender<crate::runtime_types::Act>,
    ) -> u64 {
        let channel_id = self.registry.allocate_adapter_channel_id(adapter_id);
        self.registry.attach_adapter_channel(channel_id, tx);
        channel_id
    }

    pub fn new_body_endpoint(
        &self,
        channel_id: u64,
        semantic_name: &str,
    ) -> Result<BodyEndpointHandle> {
        let mut state = self.endpoint_state.blocking_lock();
        let seq = state
            .name_seq
            .entry(semantic_name.to_string())
            .and_modify(|v| *v = v.saturating_add(1))
            .or_insert(1);
        let body_endpoint_name = format!("{}.{}", semantic_name, *seq);
        let body_endpoint_id = Uuid::now_v7();

        let endpoint = RegisteredBodyEndpoint {
            body_endpoint_id,
            body_endpoint_name: body_endpoint_name.clone(),
            channel_id: Some(channel_id),
            route_keys: BTreeSet::new(),
        };

        state
            .by_name
            .insert(body_endpoint_name.clone(), body_endpoint_id);
        state
            .by_channel
            .entry(channel_id)
            .or_default()
            .insert(body_endpoint_id);
        state.by_id.insert(body_endpoint_id, endpoint);

        Ok(BodyEndpointHandle {
            body_endpoint_id,
            body_endpoint_name,
        })
    }

    pub fn register_inline_body_endpoint(
        &self,
        semantic_name: &str,
        endpoint: Arc<dyn EndpointPort>,
        descriptors: Vec<EndpointCapabilityDescriptor>,
    ) -> Result<BodyEndpointHandle> {
        let mut state = self.endpoint_state.blocking_lock();
        let seq = state
            .name_seq
            .entry(semantic_name.to_string())
            .and_modify(|v| *v = v.saturating_add(1))
            .or_insert(1);
        let body_endpoint_name = format!("{}.{}", semantic_name, *seq);
        let body_endpoint_id = Uuid::now_v7();

        let mut route_keys = BTreeSet::new();
        for mut descriptor in descriptors {
            descriptor.route.endpoint_id = body_endpoint_name.clone();
            self.registry
                .register(descriptor.clone(), Arc::clone(&endpoint))
                .map_err(|err| anyhow::anyhow!(err.to_string()))?;
            route_keys.insert(descriptor.route);
        }

        let registered = RegisteredBodyEndpoint {
            body_endpoint_id,
            body_endpoint_name: body_endpoint_name.clone(),
            channel_id: None,
            route_keys,
        };
        state
            .by_name
            .insert(body_endpoint_name.clone(), body_endpoint_id);
        state.by_id.insert(body_endpoint_id, registered);

        Ok(BodyEndpointHandle {
            body_endpoint_id,
            body_endpoint_name,
        })
    }

    pub fn register_body_endpoint_capability(
        &self,
        body_endpoint_id: Uuid,
        mut descriptor: EndpointCapabilityDescriptor,
    ) -> Result<EndpointCapabilityDescriptor> {
        let mut state = self.endpoint_state.blocking_lock();
        let endpoint = state
            .by_id
            .get_mut(&body_endpoint_id)
            .ok_or_else(|| anyhow::anyhow!("body endpoint is not registered"))?;

        descriptor.route.endpoint_id = endpoint.body_endpoint_name.clone();
        if let Some(channel_id) = endpoint.channel_id {
            self.registry
                .register_adapter_route(channel_id, descriptor.clone())
                .map_err(|err| anyhow::anyhow!(err.to_string()))?;
        } else {
            let endpoint_port = self
                .registry
                .resolve(&endpoint.body_endpoint_name)
                .ok_or_else(|| anyhow::anyhow!("inline body endpoint port is unavailable"))?;
            self.registry
                .register(descriptor.clone(), endpoint_port)
                .map_err(|err| anyhow::anyhow!(err.to_string()))?;
        }

        endpoint.route_keys.insert(descriptor.route.clone());
        Ok(descriptor)
    }

    pub fn unregister_body_endpoint_capability(
        &self,
        body_endpoint_id: Uuid,
        capability_id: &str,
    ) -> Option<RouteKey> {
        let mut state = self.endpoint_state.blocking_lock();
        let endpoint = state.by_id.get_mut(&body_endpoint_id)?;
        let route = RouteKey {
            endpoint_id: endpoint.body_endpoint_name.clone(),
            capability_id: capability_id.to_string(),
        };

        let removed = if let Some(channel_id) = endpoint.channel_id {
            self.registry
                .unregister_adapter_route(channel_id, &route)
                .is_some()
        } else {
            self.registry.unregister(&route).is_some()
        };
        if removed {
            endpoint.route_keys.remove(&route);
            return Some(route);
        }
        None
    }

    pub fn remove_body_endpoint(&self, body_endpoint_id: Uuid) -> Vec<RouteKey> {
        let mut state = self.endpoint_state.blocking_lock();
        let Some(endpoint) = state.by_id.remove(&body_endpoint_id) else {
            return Vec::new();
        };

        state.by_name.remove(&endpoint.body_endpoint_name);
        if let Some(channel_id) = endpoint.channel_id
            && let Some(ids) = state.by_channel.get_mut(&channel_id)
        {
            ids.remove(&body_endpoint_id);
            if ids.is_empty() {
                state.by_channel.remove(&channel_id);
            }
        }

        let mut dropped = Vec::new();
        for route in endpoint.route_keys {
            let removed = if let Some(channel_id) = endpoint.channel_id {
                self.registry
                    .unregister_adapter_route(channel_id, &route)
                    .is_some()
            } else {
                self.registry.unregister(&route).is_some()
            };
            if removed {
                dropped.push(route);
            }
        }
        dropped
    }

    pub(crate) fn on_adapter_channel_closed(&self, channel_id: u64) -> Vec<RouteKey> {
        let endpoint_ids = {
            let mut state = self.endpoint_state.blocking_lock();
            state.by_channel.remove(&channel_id).unwrap_or_default()
        };

        let mut dropped = Vec::new();
        for endpoint_id in endpoint_ids {
            dropped.extend(self.remove_body_endpoint(endpoint_id));
        }
        dropped.extend(self.registry.detach_adapter_channel(channel_id));
        dropped
    }

    pub async fn shutdown(self) {
        self.shutdown.cancel();
        for task in self.tasks.into_inner() {
            match task.await {
                Ok(Ok(())) => {}
                Ok(Err(err)) => eprintln!("spine adapter exited with error: {err:#}"),
                Err(err) => eprintln!("spine adapter task join failed: {err}"),
            }
        }
    }
}

pub struct SpineHandle {
    inner: Arc<Spine>,
}

impl SpineHandle {
    pub fn new(inner: Arc<Spine>) -> Self {
        Self { inner }
    }

    pub fn registry_port(&self) -> Arc<dyn EndpointRegistryPort> {
        self.inner.registry_port()
    }

    pub fn executor_port(&self) -> Arc<dyn SpineExecutorPort> {
        self.inner.executor_port()
    }
}

pub async fn shutdown_global_spine(spine: Arc<Spine>) -> Result<()> {
    match Arc::try_unwrap(spine) {
        Ok(spine) => {
            spine.shutdown().await;
            Ok(())
        }
        Err(_) => Err(anyhow::anyhow!(
            "failed to shutdown spine: outstanding references still exist"
        ))
        .context("spine shutdown requires exclusive ownership"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use tokio::sync::mpsc;

    use crate::{
        config::SpineRuntimeConfig,
        runtime_types::{Act, RequestedResources},
        spine::{
            error::SpineError,
            ports::EndpointPort,
            types::{CostVector, EndpointExecutionOutcome},
        },
    };

    struct StubInlineEndpoint;

    #[async_trait]
    impl EndpointPort for StubInlineEndpoint {
        async fn invoke(
            &self,
            _act: Act,
        ) -> std::result::Result<EndpointExecutionOutcome, SpineError> {
            Ok(EndpointExecutionOutcome::Applied {
                actual_cost_micro: 0,
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
        Spine::new(&cfg, SenseIngress::new(mpsc::channel(4).0))
    }

    #[test]
    fn assigns_monotonic_fully_qualified_endpoint_names() {
        let spine = test_spine();
        let (tx, _rx) = mpsc::unbounded_channel();
        let ch = spine.on_adapter_channel_open(1, tx);

        let a = spine.new_body_endpoint(ch, "cli").expect("first endpoint");
        let b = spine.new_body_endpoint(ch, "cli").expect("second endpoint");

        assert_eq!(a.body_endpoint_name, "cli.1");
        assert_eq!(b.body_endpoint_name, "cli.2");
    }

    #[test]
    fn remove_body_endpoint_returns_registered_routes() {
        let spine = test_spine();
        let (tx, _rx) = mpsc::unbounded_channel();
        let ch = spine.on_adapter_channel_open(2, tx);
        let endpoint = spine.new_body_endpoint(ch, "macos-app").expect("endpoint");

        let registered = spine
            .register_body_endpoint_capability(
                endpoint.body_endpoint_id,
                descriptor("present.message"),
            )
            .expect("register capability");
        assert_eq!(registered.route.endpoint_id, "macos-app.1");

        let dropped = spine.remove_body_endpoint(endpoint.body_endpoint_id);
        assert_eq!(dropped.len(), 1);
        assert_eq!(dropped[0].endpoint_id, "macos-app.1");
        assert_eq!(dropped[0].capability_id, "present.message");
    }

    #[test]
    fn inline_endpoint_registration_is_spine_managed_and_not_channel_owned() {
        let spine = test_spine();
        let endpoint: Arc<dyn EndpointPort> = Arc::new(StubInlineEndpoint);

        let h1 = spine
            .register_inline_body_endpoint(
                "std-shell",
                Arc::clone(&endpoint),
                vec![descriptor("tool.shell.exec")],
            )
            .expect("register first inline endpoint");
        let h2 = spine
            .register_inline_body_endpoint(
                "std-shell",
                endpoint,
                vec![descriptor("tool.shell.exec")],
            )
            .expect("register second inline endpoint");

        assert_eq!(h1.body_endpoint_name, "std-shell.1");
        assert_eq!(h2.body_endpoint_name, "std-shell.2");

        let state = spine.endpoint_state.blocking_lock();
        let ep1 = state.by_id.get(&h1.body_endpoint_id).expect("ep1 state");
        assert!(ep1.channel_id.is_none());
        assert!(state.by_channel.is_empty());
    }

    #[test]
    fn dispatch_routes_by_fully_qualified_endpoint_name() {
        let spine = test_spine();
        let endpoint: Arc<dyn EndpointPort> = Arc::new(StubInlineEndpoint);
        let handle = spine
            .register_inline_body_endpoint("std-web", endpoint, vec![descriptor("tool.web.fetch")])
            .expect("register inline endpoint");

        let runtime = tokio::runtime::Runtime::new().expect("runtime");
        let outcome = runtime
            .block_on(spine.executor_port().dispatch_act(Act {
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
            .expect("dispatch by fully-qualified endpoint name");

        assert!(matches!(outcome, EndpointExecutionOutcome::Applied { .. }));
    }
}
