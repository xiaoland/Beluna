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
    channel_id: u64,
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
            channel_id,
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
        self.registry
            .register_adapter_route(endpoint.channel_id, descriptor.clone())
            .map_err(|err| anyhow::anyhow!(err.to_string()))?;

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

        if self
            .registry
            .unregister_adapter_route(endpoint.channel_id, &route)
            .is_some()
        {
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
        if let Some(ids) = state.by_channel.get_mut(&endpoint.channel_id) {
            ids.remove(&body_endpoint_id);
            if ids.is_empty() {
                state.by_channel.remove(&endpoint.channel_id);
            }
        }

        let mut dropped = Vec::new();
        for route in endpoint.route_keys {
            if self
                .registry
                .unregister_adapter_route(endpoint.channel_id, &route)
                .is_some()
            {
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
