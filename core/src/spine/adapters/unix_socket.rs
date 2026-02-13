use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    fs,
    io::ErrorKind,
    os::unix::fs::FileTypeExt,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};

use anyhow::{Context, Result, bail};
use async_trait::async_trait;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{UnixListener, UnixStream},
    sync::{mpsc, oneshot},
    time::timeout,
};
use tokio_util::sync::CancellationToken;

use crate::spine::{
    error::{SpineError, backend_failure, route_conflict, route_not_found},
    ports::EndpointPort,
    types::{AdmittedAction, EndpointExecutionOutcome, EndpointInvocation, RouteKey},
};

use super::wire::{
    BodyEgressMessage, BodyEndpointResultMessage, BodyEndpointResultOutcome, BodyIngressMessage,
    InboundBodyMessage, encode_body_egress_message, parse_body_ingress_message,
};

struct PendingInvoke {
    body_endpoint_id: u64,
    response_tx: oneshot::Sender<EndpointExecutionOutcome>,
}

#[derive(Default)]
struct BodyEndpointBrokerState {
    body_endpoints: BTreeMap<u64, mpsc::UnboundedSender<BodyEgressMessage>>,
    route_owner: BTreeMap<RouteKey, u64>,
    routes_by_body_endpoint: BTreeMap<u64, BTreeSet<RouteKey>>,
    pending: HashMap<String, PendingInvoke>,
}

pub struct BodyEndpointBroker {
    state: Mutex<BodyEndpointBrokerState>,
    next_body_endpoint_id: AtomicU64,
    next_request_id: AtomicU64,
    invoke_timeout: Duration,
}

impl BodyEndpointBroker {
    pub fn new(invoke_timeout_ms: u64) -> Self {
        Self {
            state: Mutex::new(BodyEndpointBrokerState::default()),
            next_body_endpoint_id: AtomicU64::new(1),
            next_request_id: AtomicU64::new(1),
            invoke_timeout: Duration::from_millis(invoke_timeout_ms.max(1)),
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

    pub fn register_route(
        &self,
        body_endpoint_id: u64,
        route: &RouteKey,
    ) -> Result<(), SpineError> {
        let mut state = self.state.lock().expect("lock poisoned");
        if !state.body_endpoints.contains_key(&body_endpoint_id) {
            return Err(backend_failure(format!(
                "body endpoint {} is not connected",
                body_endpoint_id
            )));
        }

        if let Some(owner) = state.route_owner.get(route) {
            if *owner != body_endpoint_id {
                return Err(route_conflict(format!(
                    "route already owned by another body endpoint: {}::{}",
                    route.affordance_key, route.capability_handle
                )));
            }
            return Ok(());
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

        let mut stale_ids = Vec::new();
        for (request_id, pending) in &state.pending {
            if pending.body_endpoint_id == body_endpoint_id {
                stale_ids.push(request_id.clone());
            }
        }

        for request_id in stale_ids {
            if let Some(pending) = state.pending.remove(&request_id) {
                let _ = pending
                    .response_tx
                    .send(EndpointExecutionOutcome::Rejected {
                        reason_code: "body_endpoint_disconnected".to_string(),
                        reference_id: format!("remote:disconnected:{request_id}"),
                    });
            }
        }

        routes
    }

    pub fn resolve_result(&self, body_endpoint_id: u64, result: BodyEndpointResultMessage) {
        let mut state = self.state.lock().expect("lock poisoned");
        let Some(pending) = state.pending.remove(&result.request_id) else {
            return;
        };

        if pending.body_endpoint_id != body_endpoint_id {
            return;
        }

        let mapped = match result.outcome {
            BodyEndpointResultOutcome::Applied {
                actual_cost_micro,
                reference_id,
            } => EndpointExecutionOutcome::Applied {
                actual_cost_micro,
                reference_id,
            },
            BodyEndpointResultOutcome::Rejected {
                reason_code,
                reference_id,
            } => EndpointExecutionOutcome::Rejected {
                reason_code,
                reference_id,
            },
            BodyEndpointResultOutcome::Deferred { reason_code } => {
                EndpointExecutionOutcome::Deferred { reason_code }
            }
        };

        let _ = pending.response_tx.send(mapped);
    }

    pub async fn invoke_route(
        &self,
        route: &RouteKey,
        action: AdmittedAction,
    ) -> Result<EndpointExecutionOutcome, SpineError> {
        let (request_id, body_endpoint_tx, response_rx) = {
            let mut state = self.state.lock().expect("lock poisoned");
            let Some(body_endpoint_id) = state.route_owner.get(route).copied() else {
                return Err(route_not_found(format!(
                    "route is not registered: {}::{}",
                    route.affordance_key, route.capability_handle
                )));
            };

            let Some(body_endpoint_tx) = state.body_endpoints.get(&body_endpoint_id).cloned()
            else {
                return Err(backend_failure(format!(
                    "body endpoint {} is unavailable",
                    body_endpoint_id
                )));
            };

            let request_id = format!(
                "req:{}",
                self.next_request_id.fetch_add(1, Ordering::Relaxed)
            );
            let (response_tx, response_rx) = oneshot::channel();
            state.pending.insert(
                request_id.clone(),
                PendingInvoke {
                    body_endpoint_id,
                    response_tx,
                },
            );

            (request_id, body_endpoint_tx, response_rx)
        };

        if body_endpoint_tx
            .send(BodyEgressMessage::BodyEndpointInvoke {
                request_id: request_id.clone(),
                action,
            })
            .is_err()
        {
            let mut state = self.state.lock().expect("lock poisoned");
            state.pending.remove(&request_id);
            return Err(backend_failure(format!(
                "failed to send invoke request {}",
                request_id
            )));
        }

        match timeout(self.invoke_timeout, response_rx).await {
            Ok(Ok(outcome)) => Ok(outcome),
            Ok(Err(_)) => Err(backend_failure(format!(
                "remote invoke channel closed for {}",
                request_id
            ))),
            Err(_) => {
                let mut state = self.state.lock().expect("lock poisoned");
                state.pending.remove(&request_id);
                Err(backend_failure(format!(
                    "remote invoke timed out for {}",
                    request_id
                )))
            }
        }
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
            .invoke_route(&self.route, invocation.action)
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
        message_tx: mpsc::UnboundedSender<BodyIngressMessage>,
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
                            let sender = message_tx.clone();
                            let broker_ref = Arc::clone(&broker);
                            tokio::spawn(async move {
                                if let Err(err) = handle_body_endpoint(stream, sender, broker_ref).await {
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
    message_tx: mpsc::UnboundedSender<BodyIngressMessage>,
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
                InboundBodyMessage::BodyEndpointResult(result) => {
                    broker.resolve_result(body_endpoint_id, result);
                }
                InboundBodyMessage::BodyEndpointRegister {
                    endpoint_id,
                    descriptor,
                } => {
                    let _ = message_tx.send(BodyIngressMessage::BodyEndpointRegister {
                        body_endpoint_id,
                        endpoint_id,
                        descriptor,
                    });
                }
                InboundBodyMessage::BodyEndpointUnregister { route } => {
                    let _ = message_tx.send(BodyIngressMessage::BodyEndpointUnregister {
                        body_endpoint_id,
                        route,
                    });
                }
                InboundBodyMessage::Sense(sense) => {
                    let _ = message_tx.send(BodyIngressMessage::Sense(sense));
                }
                InboundBodyMessage::EnvSnapshot(snapshot) => {
                    let _ = message_tx.send(BodyIngressMessage::EnvSnapshot(snapshot));
                }
                InboundBodyMessage::AdmissionFeedback(feedback) => {
                    let _ = message_tx.send(BodyIngressMessage::AdmissionFeedback(feedback));
                }
                InboundBodyMessage::CapabilityCatalogUpdate(catalog) => {
                    let _ = message_tx.send(BodyIngressMessage::CapabilityCatalogUpdate(catalog));
                }
                InboundBodyMessage::CortexLimitsUpdate(limits) => {
                    let _ = message_tx.send(BodyIngressMessage::CortexLimitsUpdate(limits));
                }
                InboundBodyMessage::IntentContextUpdate(context) => {
                    let _ = message_tx.send(BodyIngressMessage::IntentContextUpdate(context));
                }
            },
            Err(err) => eprintln!("ignoring invalid protocol message: {err}"),
        }
    }

    let routes = broker.detach_body_endpoint(body_endpoint_id);
    if !routes.is_empty() {
        let _ = message_tx.send(BodyIngressMessage::BodyEndpointDisconnected {
            body_endpoint_id,
            routes,
        });
    }

    match writer_task.await {
        Ok(Ok(())) => {}
        Ok(Err(err)) => eprintln!("socket writer failed: {err:#}"),
        Err(err) => eprintln!("socket writer task join failed: {err}"),
    }

    Ok(())
}
