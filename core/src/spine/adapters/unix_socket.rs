use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::ErrorKind,
    os::unix::fs::FileTypeExt,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
};

use anyhow::{Context, Result, bail};
use async_trait::async_trait;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{UnixListener, UnixStream},
    sync::mpsc,
};
use tokio_util::sync::CancellationToken;

use crate::{
    ingress::SenseIngress,
    runtime_types::{CapabilityDropPatch, CapabilityPatch, Sense},
    spine::{
        error::{SpineError, backend_failure, route_not_found},
        ports::{EndpointPort, EndpointRegistryPort},
        types::{ActDispatchRequest, EndpointExecutionOutcome, EndpointInvocation, RouteKey},
    },
};

use super::wire::{
    BodyEgressMessage, InboundBodyMessage, encode_body_egress_message, parse_body_ingress_message,
};

#[derive(Default)]
struct BodyEndpointBrokerState {
    body_endpoints: BTreeMap<u64, mpsc::UnboundedSender<BodyEgressMessage>>,
    route_owner: BTreeMap<RouteKey, u64>,
    routes_by_body_endpoint: BTreeMap<u64, BTreeSet<RouteKey>>,
}

pub struct BodyEndpointBroker {
    state: Mutex<BodyEndpointBrokerState>,
    next_body_endpoint_id: AtomicU64,
}

impl BodyEndpointBroker {
    pub fn new(_invoke_timeout_ms: u64) -> Self {
        Self {
            state: Mutex::new(BodyEndpointBrokerState::default()),
            next_body_endpoint_id: AtomicU64::new(1),
        }
    }

    pub fn allocate_body_endpoint_id(&self) -> u64 {
        self.next_body_endpoint_id.fetch_add(1, Ordering::Relaxed)
    }

    pub fn attach_body_endpoint(
        &self,
        body_endpoint_id: u64,
        tx: mpsc::UnboundedSender<BodyEgressMessage>,
    ) {
        let mut state = self.state.lock().expect("lock poisoned");
        state.body_endpoints.insert(body_endpoint_id, tx);
    }

    pub fn register_route(&self, body_endpoint_id: u64, route: &RouteKey) -> Result<(), SpineError> {
        let mut state = self.state.lock().expect("lock poisoned");
        if !state.body_endpoints.contains_key(&body_endpoint_id) {
            return Err(backend_failure(format!(
                "body endpoint {} is not connected",
                body_endpoint_id
            )));
        }

        if let Some(owner) = state.route_owner.get(route).copied() {
            if owner == body_endpoint_id {
                return Ok(());
            }

            if let Some(routes) = state.routes_by_body_endpoint.get_mut(&owner) {
                routes.remove(route);
                if routes.is_empty() {
                    state.routes_by_body_endpoint.remove(&owner);
                }
            }
        }

        state.route_owner.insert(route.clone(), body_endpoint_id);
        state
            .routes_by_body_endpoint
            .entry(body_endpoint_id)
            .or_default()
            .insert(route.clone());
        Ok(())
    }

    pub fn unregister_route(&self, body_endpoint_id: u64, route: &RouteKey) {
        let mut state = self.state.lock().expect("lock poisoned");
        if state.route_owner.get(route).copied() == Some(body_endpoint_id) {
            state.route_owner.remove(route);
        }
        if let Some(routes) = state.routes_by_body_endpoint.get_mut(&body_endpoint_id) {
            routes.remove(route);
            if routes.is_empty() {
                state.routes_by_body_endpoint.remove(&body_endpoint_id);
            }
        }
    }

    pub fn detach_body_endpoint(&self, body_endpoint_id: u64) -> Vec<RouteKey> {
        let mut state = self.state.lock().expect("lock poisoned");
        state.body_endpoints.remove(&body_endpoint_id);

        let routes: Vec<RouteKey> = state
            .routes_by_body_endpoint
            .remove(&body_endpoint_id)
            .map(|set| set.into_iter().collect())
            .unwrap_or_default();

        for route in &routes {
            state.route_owner.remove(route);
        }

        routes
    }

    pub async fn invoke_route(
        &self,
        route: &RouteKey,
        request: ActDispatchRequest,
    ) -> Result<EndpointExecutionOutcome, SpineError> {
        let body_endpoint_tx = {
            let state = self.state.lock().expect("lock poisoned");
            let Some(body_endpoint_id) = state.route_owner.get(route).copied() else {
                return Err(route_not_found(format!(
                    "route is not registered: {}::{}",
                    route.endpoint_id, route.capability_id
                )));
            };

            let Some(body_endpoint_tx) = state.body_endpoints.get(&body_endpoint_id).cloned()
            else {
                return Err(backend_failure(format!(
                    "body endpoint {} is unavailable",
                    body_endpoint_id
                )));
            };
            body_endpoint_tx
        };

        if body_endpoint_tx
            .send(BodyEgressMessage::Act {
                request: request.clone(),
            })
            .is_err()
        {
            return Err(backend_failure(format!(
                "failed to send act {}",
                request.act.act_id
            )));
        }

        Ok(EndpointExecutionOutcome::Applied {
            actual_cost_micro: request.act.requested_resources.survival_micro.max(0),
            reference_id: format!("remote:act_sent:{}", request.act.act_id),
        })
    }
}

pub struct RemoteBodyEndpointPort {
    route: RouteKey,
    broker: Arc<BodyEndpointBroker>,
}

impl RemoteBodyEndpointPort {
    pub fn new(route: RouteKey, broker: Arc<BodyEndpointBroker>) -> Self {
        Self { route, broker }
    }
}

#[async_trait]
impl EndpointPort for RemoteBodyEndpointPort {
    async fn invoke(
        &self,
        invocation: EndpointInvocation,
    ) -> Result<EndpointExecutionOutcome, SpineError> {
        self.broker
            .invoke_route(&self.route, invocation.request)
            .await
    }
}

pub struct UnixSocketAdapter {
    pub socket_path: PathBuf,
}

impl UnixSocketAdapter {
    pub fn new(socket_path: PathBuf) -> Self {
        Self { socket_path }
    }

    pub async fn run(
        &self,
        ingress: SenseIngress,
        registry: Arc<dyn EndpointRegistryPort>,
        broker: Arc<BodyEndpointBroker>,
        shutdown: CancellationToken,
    ) -> Result<()> {
        Self::prepare_socket_path(&self.socket_path)?;
        let listener = UnixListener::bind(&self.socket_path)
            .with_context(|| format!("unable to bind socket {}", self.socket_path.display()))?;

        loop {
            tokio::select! {
                _ = shutdown.cancelled() => {
                    break;
                }
                accept_result = listener.accept() => {
                    match accept_result {
                        Ok((stream, _)) => {
                            let ingress = ingress.clone();
                            let registry = Arc::clone(&registry);
                            let broker_ref = Arc::clone(&broker);
                            tokio::spawn(async move {
                                if let Err(err) = handle_body_endpoint(stream, ingress, registry, broker_ref).await {
                                    eprintln!("body endpoint handling failed: {err:#}");
                                }
                            });
                        }
                        Err(err) => {
                            eprintln!("accept failed: {err}");
                        }
                    }
                }
            }
        }

        Self::cleanup_socket_path(&self.socket_path)?;
        Ok(())
    }

    fn prepare_socket_path(path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("unable to create {}", parent.display()))?;
        }

        match fs::symlink_metadata(path) {
            Ok(metadata) => {
                if metadata.file_type().is_socket() || metadata.is_file() {
                    fs::remove_file(path).with_context(|| {
                        format!("unable to remove stale socket {}", path.display())
                    })?;
                } else {
                    bail!(
                        "socket path exists but is not removable as file/socket: {}",
                        path.display()
                    );
                }
            }
            Err(err) if err.kind() == ErrorKind::NotFound => {}
            Err(err) => {
                return Err(err).with_context(|| format!("unable to inspect {}", path.display()));
            }
        }

        Ok(())
    }

    fn cleanup_socket_path(path: &Path) -> Result<()> {
        match fs::remove_file(path) {
            Ok(_) => Ok(()),
            Err(err) if err.kind() == ErrorKind::NotFound => Ok(()),
            Err(err) => Err(err).with_context(|| format!("unable to remove {}", path.display())),
        }
    }
}

async fn handle_body_endpoint(
    stream: UnixStream,
    ingress: SenseIngress,
    registry: Arc<dyn EndpointRegistryPort>,
    broker: Arc<BodyEndpointBroker>,
) -> Result<()> {
    let body_endpoint_id = broker.allocate_body_endpoint_id();
    let (read_half, mut write_half) = stream.into_split();

    let (outbound_tx, mut outbound_rx) = mpsc::unbounded_channel::<BodyEgressMessage>();
    broker.attach_body_endpoint(body_endpoint_id, outbound_tx);

    let writer_task = tokio::spawn(async move {
        while let Some(message) = outbound_rx.recv().await {
            let encoded = encode_body_egress_message(&message)?;
            write_half.write_all(encoded.as_bytes()).await?;
            write_half.flush().await?;
        }

        Ok::<(), anyhow::Error>(())
    });

    let mut lines = BufReader::new(read_half).lines();

    while let Some(line) = lines.next_line().await? {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        match parse_body_ingress_message(line) {
            Ok(message) => match message {
                InboundBodyMessage::BodyEndpointRegister {
                    endpoint_id,
                    descriptor,
                } => {
                    let route = descriptor.route.clone();
                    if let Err(err) = broker.register_route(body_endpoint_id, &route) {
                        eprintln!("body endpoint register rejected: {err}");
                        continue;
                    }

                    let _ = registry.unregister(&route);

                    let endpoint: Arc<dyn EndpointPort> = Arc::new(RemoteBodyEndpointPort::new(
                        route.clone(),
                        Arc::clone(&broker),
                    ));

                    if let Err(err) = registry.register(
                        crate::spine::types::EndpointRegistration {
                            endpoint_id,
                            descriptor: descriptor.clone(),
                        },
                        endpoint,
                    ) {
                        broker.unregister_route(body_endpoint_id, &route);
                        eprintln!("body endpoint route registration failed: {err}");
                        continue;
                    }

                    if let Err(err) = ingress
                        .send(Sense::NewCapabilities(CapabilityPatch {
                            entries: vec![descriptor],
                        }))
                        .await
                    {
                        eprintln!("dropping capability patch after register: {err}");
                    }
                }
                InboundBodyMessage::BodyEndpointUnregister { route } => {
                    broker.unregister_route(body_endpoint_id, &route);
                    let _ = registry.unregister(&route);
                    if let Err(err) = ingress
                        .send(Sense::DropCapabilities(CapabilityDropPatch {
                            routes: vec![route],
                        }))
                        .await
                    {
                        eprintln!("dropping capability drop after unregister: {err}");
                    }
                }
                InboundBodyMessage::Sense(sense) => {
                    if let Err(err) = ingress.send(Sense::Domain(sense)).await {
                        eprintln!("dropping sense due to closed ingress: {err}");
                    }
                }
                InboundBodyMessage::NewCapabilities(patch) => {
                    if let Err(err) = ingress.send(Sense::NewCapabilities(patch)).await {
                        eprintln!("dropping new_capabilities due to closed ingress: {err}");
                    }
                }
                InboundBodyMessage::DropCapabilities(drop_patch) => {
                    if let Err(err) = ingress.send(Sense::DropCapabilities(drop_patch)).await {
                        eprintln!("dropping drop_capabilities due to closed ingress: {err}");
                    }
                }
            },
            Err(err) => {
                eprintln!("invalid ingress message: {err}");
            }
        }
    }

    let routes = broker.detach_body_endpoint(body_endpoint_id);
    for route in &routes {
        let _ = registry.unregister(route);
    }
    if !routes.is_empty() {
        let _ = ingress
            .send(Sense::DropCapabilities(CapabilityDropPatch { routes }))
            .await;
    }

    writer_task.await??;

    Ok(())
}
