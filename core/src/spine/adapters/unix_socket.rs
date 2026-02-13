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

use crate::spine::{
    error::{SpineError, backend_failure, route_not_found},
    ports::EndpointPort,
    types::{AdmittedAction, EndpointExecutionOutcome, EndpointInvocation, RouteKey},
};

use super::wire::{
    BodyEgressMessage, BodyIngressMessage, InboundBodyMessage, encode_body_egress_message,
    parse_body_ingress_message,
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

        if let Some(owner) = state.route_owner.get(route).copied() {
            if owner == body_endpoint_id {
                return Ok(());
            }

            // Newer registrations take ownership. This prevents stale client
            // connections from permanently blocking route recovery.
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
        action: AdmittedAction,
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
                action: action.clone(),
            })
            .is_err()
        {
            return Err(backend_failure(format!(
                "failed to send act {}",
                action.neural_signal_id
            )));
        }

        Ok(EndpointExecutionOutcome::Applied {
            actual_cost_micro: action.reserved_cost.survival_micro.max(0),
            reference_id: format!("remote:act_sent:{}", action.neural_signal_id),
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

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use tokio::sync::mpsc;

    use super::*;
    use crate::spine::types::CostVector;

    fn test_route() -> RouteKey {
        RouteKey {
            endpoint_id: "macos-app.01".to_string(),
            capability_id: "present.message".to_string(),
        }
    }

    fn test_action(route: &RouteKey) -> AdmittedAction {
        AdmittedAction {
            neural_signal_id: "ns_test_01".to_string(),
            capability_instance_id: "cap_inst_test_01".to_string(),
            source_attempt_id: "attempt_test_01".to_string(),
            reserve_entry_id: "reserve_test_01".to_string(),
            cost_attribution_id: "cost_test_01".to_string(),
            endpoint_id: route.endpoint_id.clone(),
            capability_id: route.capability_id.clone(),
            normalized_payload: serde_json::json!({"type":"noop"}),
            reserved_cost: CostVector::default(),
            degraded: false,
            degradation_profile_id: None,
            admission_cycle: 1,
            metadata: BTreeMap::new(),
        }
    }

    #[tokio::test]
    async fn route_registration_transfers_to_newest_endpoint() {
        let broker = BodyEndpointBroker::new(30_000);

        let endpoint_a = broker.allocate_body_endpoint_id();
        let (tx_a, mut rx_a) = mpsc::unbounded_channel();
        broker.attach_body_endpoint(endpoint_a, tx_a);

        let endpoint_b = broker.allocate_body_endpoint_id();
        let (tx_b, mut rx_b) = mpsc::unbounded_channel();
        broker.attach_body_endpoint(endpoint_b, tx_b);

        let route = test_route();
        broker.register_route(endpoint_a, &route).expect("register A");
        broker.register_route(endpoint_b, &route).expect("register B");

        // Disconnecting the old endpoint must not remove the route after handoff.
        let detached_routes = broker.detach_body_endpoint(endpoint_a);
        assert!(
            detached_routes.is_empty(),
            "old owner should not still own route after transfer"
        );

        broker
            .invoke_route(&route, test_action(&route))
            .await
            .expect("route invoke");

        let sent_to_b = rx_b.try_recv().expect("new owner should receive act");
        assert!(
            matches!(sent_to_b, BodyEgressMessage::Act { .. }),
            "expected act message to be delivered"
        );
        assert!(
            rx_a.try_recv().is_err(),
            "old owner should not receive act after transfer"
        );
    }
}
