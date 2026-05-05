use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::ErrorKind,
    os::unix::fs::FileTypeExt,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{UnixListener, UnixStream},
    sync::mpsc,
    task::JoinHandle,
    time::{Duration, Instant, timeout},
};
use tokio_util::sync::CancellationToken;
use tracing::Instrument;

use crate::{
    observability::runtime::{self as observability_runtime, AdapterLifecycleState},
    spine::{AdapterContext, SpineAdapterPort, types::NeuralSignalDescriptor},
    types::{Act, Sense, default_sense_weight, is_uuid_v4, is_uuid_v7},
};

pub mod config;
pub use config::UnixSocketNdjsonAdapterConfig;

type SessionActSenders = Arc<Mutex<BTreeMap<String, mpsc::UnboundedSender<Act>>>>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct NdjsonEnvelope<T> {
    method: String,
    id: String,
    timestamp: u64,
    body: T,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct OutboundActBody {
    act: Act,
}

#[derive(Debug, Clone, PartialEq)]
enum InboundBodyMessage {
    Auth {
        endpoint_name: String,
        ns_descriptors: Vec<NeuralSignalDescriptor>,
        proprioceptions: BTreeMap<String, String>,
    },
    Sense(InboundSenseFrame),
    NewProprioceptions {
        entries: BTreeMap<String, String>,
    },
    DropProprioceptions {
        keys: Vec<String>,
    },
    ActAck {
        act_instance_id: String,
    },
    Unplug,
}

#[derive(Debug, Clone, PartialEq)]
struct InboundSenseFrame {
    sense_instance_id: String,
    neural_signal_descriptor_id: String,
    payload: String,
    weight: f64,
    act_instance_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct InboundAuthBody {
    endpoint_name: String,
    #[serde(default)]
    ns_descriptors: Vec<NeuralSignalDescriptor>,
    #[serde(default)]
    proprioceptions: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct InboundNewProprioceptionsBody {
    #[serde(default)]
    entries: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct InboundDropProprioceptionsBody {
    #[serde(default)]
    keys: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct InboundSenseBody {
    sense_instance_id: String,
    neural_signal_descriptor_id: String,
    payload: String,
    #[serde(default = "default_sense_weight")]
    weight: f64,
    #[serde(default)]
    act_instance_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct InboundActAckBody {
    act_instance_id: String,
}

fn parse_body_afferent_message(line: &str) -> Result<InboundBodyMessage, serde_json::Error> {
    let wire: NdjsonEnvelope<serde_json::Value> = serde_json::from_str(line)?;
    if !is_uuid_v4(&wire.id) {
        return Err(invalid_correlated_sense_error(
            "id must be a valid uuid-v4 string",
        ));
    }
    if wire.timestamp == 0 {
        return Err(invalid_correlated_sense_error(
            "timestamp must be a non-zero utc epoch milliseconds integer",
        ));
    }

    let message = match wire.method.as_str() {
        "auth" => {
            let body: InboundAuthBody = decode_envelope_body(wire.body)?;
            InboundBodyMessage::Auth {
                endpoint_name: body.endpoint_name,
                ns_descriptors: body.ns_descriptors,
                proprioceptions: body.proprioceptions,
            }
        }
        "new_proprioceptions" => {
            let body: InboundNewProprioceptionsBody = decode_envelope_body(wire.body)?;
            InboundBodyMessage::NewProprioceptions {
                entries: body.entries,
            }
        }
        "drop_proprioceptions" => {
            let body: InboundDropProprioceptionsBody = decode_envelope_body(wire.body)?;
            InboundBodyMessage::DropProprioceptions { keys: body.keys }
        }
        "sense" => {
            let body: InboundSenseBody = decode_envelope_body(wire.body)?;
            if !is_uuid_v4(&body.sense_instance_id) {
                return Err(invalid_correlated_sense_error(
                    "sense_instance_id must be a valid uuid-v4 string",
                ));
            }
            if body.neural_signal_descriptor_id.trim().is_empty() {
                return Err(invalid_correlated_sense_error(
                    "neural_signal_descriptor_id must be a non-empty string",
                ));
            }
            if body.payload.trim().is_empty() {
                return Err(invalid_correlated_sense_error(
                    "payload must be a non-empty string",
                ));
            }
            if !(0.0..=1.0).contains(&body.weight) {
                return Err(invalid_correlated_sense_error(
                    "weight must be within [0, 1]",
                ));
            }
            if let Some(act_instance_id) = body.act_instance_id.as_deref()
                && !is_uuid_v7(act_instance_id)
            {
                return Err(invalid_correlated_sense_error(
                    "act_instance_id must be a valid uuid-v7 string",
                ));
            }
            InboundBodyMessage::Sense(InboundSenseFrame {
                sense_instance_id: body.sense_instance_id,
                neural_signal_descriptor_id: body.neural_signal_descriptor_id,
                payload: body.payload,
                weight: body.weight,
                act_instance_id: body.act_instance_id,
            })
        }
        "act_ack" => {
            let body: InboundActAckBody = decode_envelope_body(wire.body)?;
            if !is_uuid_v7(&body.act_instance_id) {
                return Err(invalid_correlated_sense_error(
                    "act_ack.act_instance_id must be a valid uuid-v7 string",
                ));
            }
            InboundBodyMessage::ActAck {
                act_instance_id: body.act_instance_id,
            }
        }
        "unplug" => InboundBodyMessage::Unplug,
        "act" => {
            return Err(invalid_correlated_sense_error(
                "direction violation: endpoint cannot send method 'act'",
            ));
        }
        _ => {
            return Err(invalid_correlated_sense_error(
                "unsupported method, expected one of: auth|sense|act_ack|unplug|new_proprioceptions|drop_proprioceptions",
            ));
        }
    };
    Ok(message)
}

fn decode_envelope_body<T: DeserializeOwned>(
    body: serde_json::Value,
) -> Result<T, serde_json::Error> {
    serde_json::from_value(body)
}

fn invalid_correlated_sense_error(message: &str) -> serde_json::Error {
    serde_json::Error::io(std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        message.to_string(),
    ))
}

fn encode_body_egress_act_message(act: &Act) -> Result<String, serde_json::Error> {
    let encoded = serde_json::to_string(&NdjsonEnvelope {
        method: "act".to_string(),
        id: uuid::Uuid::new_v4().to_string(),
        timestamp: timestamp_millis(),
        body: OutboundActBody { act: act.clone() },
    })?;
    Ok(format!("{encoded}\n"))
}

fn timestamp_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn normalize_proprioception_key(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn namespaced_body_proprioception_key(body_endpoint_id: &str, key: &str) -> Option<String> {
    let normalized = normalize_proprioception_key(key)?;
    Some(format!("body.{body_endpoint_id}.{normalized}"))
}

fn namespaced_body_proprioception_entries(
    body_endpoint_id: &str,
    entries: &BTreeMap<String, String>,
) -> BTreeMap<String, String> {
    let mut mapped = BTreeMap::new();
    for (key, value) in entries {
        if let Some(namespaced) = namespaced_body_proprioception_key(body_endpoint_id, key) {
            mapped.insert(namespaced, value.clone());
        }
    }
    mapped
}

fn namespaced_body_proprioception_drop_keys(
    body_endpoint_id: &str,
    keys: &[String],
) -> Vec<String> {
    let mut mapped = Vec::new();
    for key in keys {
        if let Some(namespaced) = namespaced_body_proprioception_key(body_endpoint_id, key) {
            mapped.push(namespaced);
        }
    }
    mapped
}

async fn emit_proprioception_patch(
    port: &Arc<dyn SpineAdapterPort>,
    entries: BTreeMap<String, String>,
    reason: &'static str,
) {
    if entries.is_empty() {
        return;
    }

    tracing::debug!(
        target: "spine.unix_socket",
        reason = reason,
        "apply_proprioception_patch"
    );
    port.apply_proprioception_patch(entries).await;
}

async fn emit_proprioception_drop(
    port: &Arc<dyn SpineAdapterPort>,
    keys: Vec<String>,
    reason: &'static str,
) {
    if keys.is_empty() {
        return;
    }

    tracing::debug!(
        target: "spine.unix_socket",
        reason = reason,
        "apply_proprioception_drop"
    );
    port.apply_proprioception_drop(keys).await;
}

async fn emit_spine_topology_proprioception(port: &Arc<dyn SpineAdapterPort>) {
    port.publish_topology_proprioception_snapshot().await;
}

pub struct UnixSocketAdapter {
    pub socket_path: PathBuf,
    pub adapter_id: u64,
}

impl UnixSocketAdapter {
    pub fn from_config(adapter_id: u64, config: UnixSocketNdjsonAdapterConfig) -> Self {
        Self {
            socket_path: config.socket_path,
            adapter_id,
        }
    }

    pub fn new(socket_path: PathBuf, adapter_id: u64) -> Self {
        Self {
            socket_path,
            adapter_id,
        }
    }

    pub async fn run(&self, context: AdapterContext) -> Result<()> {
        Self::prepare_socket_path(&self.socket_path)?;
        let listener = UnixListener::bind(&self.socket_path)
            .with_context(|| format!("unable to bind socket {}", self.socket_path.display()))?;

        let AdapterContext {
            adapter_id,
            shutdown,
            act_rx,
            sense_tx,
            port,
        } = context;
        let sessions = Arc::new(Mutex::new(BTreeMap::new()));
        let dispatch_task = tokio::spawn(dispatch_adapter_acts(
            act_rx,
            Arc::clone(&sessions),
            Arc::clone(&port),
            shutdown.clone(),
        ));
        let mut next_session_id = 0_u64;

        loop {
            tokio::select! {
                _ = shutdown.cancelled() => {
                    break;
                }
                accept_result = listener.accept() => {
                    match accept_result {
                        Ok((stream, _)) => {
                            next_session_id = next_session_id.saturating_add(1);
                            let session_id = next_session_id;
                            let port = Arc::clone(&port);
                            let sense_tx = sense_tx.clone();
                            let sessions = Arc::clone(&sessions);
                            let session_span = tracing::info_span!(
                                target: "spine.unix_socket",
                                "body_endpoint_session",
                                adapter_id = adapter_id,
                                session_id = session_id
                            );
                            tokio::spawn(async move {
                                if let Err(err) =
                                    handle_body_endpoint(
                                        stream,
                                        port,
                                        sense_tx,
                                        sessions,
                                        adapter_id,
                                        session_id,
                                    )
                                        .await
                                {
                                    tracing::warn!(
                                        target: "spine.unix_socket",
                                        error = ?err,
                                        "body_endpoint_handling_failed"
                                    );
                                }
                            }.instrument(session_span));
                        }
                        Err(err) => {
                            tracing::warn!(
                                target: "spine.unix_socket",
                                error = %err,
                                "accept_failed"
                            );
                        }
                    }
                }
            }
        }

        dispatch_task.abort();
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

pub fn spawn_adapter_task(
    config: UnixSocketNdjsonAdapterConfig,
    context: AdapterContext,
) -> JoinHandle<Result<()>> {
    let adapter_id = context.adapter_id;
    let adapter = UnixSocketAdapter::from_config(adapter_id, config);
    let socket_path = adapter.socket_path.clone();
    let adapter_span = tracing::info_span!(
        target: "spine",
        "unix_socket_adapter_task",
        adapter_id = adapter_id,
        socket_path = %socket_path.display()
    );

    tokio::spawn(
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
            let result = adapter.run(context).await;
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
    )
}

const ACT_ACK_TIMEOUT_MS: u64 = 1_500;
const ACT_ACK_MAX_RETRIES: usize = 2;

async fn dispatch_adapter_acts(
    mut act_rx: mpsc::UnboundedReceiver<Act>,
    sessions: SessionActSenders,
    port: Arc<dyn SpineAdapterPort>,
    shutdown: CancellationToken,
) -> Result<()> {
    loop {
        tokio::select! {
            _ = shutdown.cancelled() => {
                break;
            }
            maybe_act = act_rx.recv() => {
                let Some(act) = maybe_act else {
                    break;
                };
                let tx = {
                    sessions
                        .lock()
                        .expect("lock poisoned")
                        .get(&act.endpoint_id)
                        .cloned()
                };
                let Some(tx) = tx else {
                    tracing::warn!(
                        target: "spine.unix_socket",
                        endpoint_id = %act.endpoint_id,
                        act_instance_id = %act.act_instance_id,
                        "unix_socket_session_not_found_for_dispatch"
                    );
                    port.drop_endpoint(&act.endpoint_id).await;
                    port.publish_topology_proprioception_snapshot().await;
                    continue;
                };
                if tx.send(act.clone()).is_err() {
                    tracing::warn!(
                        target: "spine.unix_socket",
                        endpoint_id = %act.endpoint_id,
                        act_instance_id = %act.act_instance_id,
                        "unix_socket_session_closed_for_dispatch"
                    );
                    sessions
                        .lock()
                        .expect("lock poisoned")
                        .remove(&act.endpoint_id);
                    port.drop_endpoint(&act.endpoint_id).await;
                    port.publish_topology_proprioception_snapshot().await;
                }
            }
        }
    }
    Ok(())
}

async fn wait_for_act_ack(
    ack_rx: &mut mpsc::UnboundedReceiver<String>,
    act_instance_id: &str,
    timeout_ms: u64,
) -> bool {
    let deadline = Instant::now() + Duration::from_millis(timeout_ms);
    loop {
        let now = Instant::now();
        if now >= deadline {
            return false;
        }

        let remaining = deadline.duration_since(now);
        match timeout(remaining, ack_rx.recv()).await {
            Ok(Some(received_act_instance_id)) => {
                if received_act_instance_id == act_instance_id {
                    return true;
                }
            }
            Ok(None) => return false,
            Err(_) => return false,
        }
    }
}

#[tracing::instrument(
    name = "handle_body_endpoint",
    target = "spine.unix_socket",
    skip(stream, port, sense_tx, sessions),
    fields(adapter_id = adapter_id, session_id = session_id)
)]
async fn handle_body_endpoint(
    stream: UnixStream,
    port: Arc<dyn SpineAdapterPort>,
    sense_tx: mpsc::UnboundedSender<Sense>,
    sessions: SessionActSenders,
    adapter_id: u64,
    session_id: u64,
) -> Result<()> {
    let (read_half, mut write_half) = stream.into_split();

    let (outbound_tx, mut outbound_rx) = mpsc::unbounded_channel::<Act>();
    let (ack_tx, mut ack_rx) = mpsc::unbounded_channel::<String>();

    let writer_span = tracing::debug_span!(
        target: "spine.unix_socket",
        "body_endpoint_writer_task",
        session_id = session_id
    );
    let writer_task = tokio::spawn(
        async move {
            while let Some(act) = outbound_rx.recv().await {
                let dispatch_started_at = Instant::now();
                tracing::debug!(
                    target: "spine.unix_socket",
                    session_id = session_id,
                    act_instance_id = %act.act_instance_id,
                    endpoint_id = %act.endpoint_id,
                    neural_signal_descriptor_id = %act.neural_signal_descriptor_id,
                    "dispatching_act_to_unix_socket_endpoint"
                );
                let mut acknowledged = false;
                for attempt in 0..=ACT_ACK_MAX_RETRIES {
                    let encoded = encode_body_egress_act_message(&act)?;
                    write_half.write_all(encoded.as_bytes()).await?;
                    write_half.flush().await?;

                    if wait_for_act_ack(&mut ack_rx, &act.act_instance_id, ACT_ACK_TIMEOUT_MS).await
                    {
                        acknowledged = true;
                        tracing::info!(
                            target: "spine.unix_socket",
                            session_id = session_id,
                            act_instance_id = %act.act_instance_id,
                            attempts = attempt + 1,
                            latency_ms = dispatch_started_at.elapsed().as_millis() as u64,
                            "act_dispatch_acknowledged_by_body_endpoint"
                        );
                        break;
                    }

                    if attempt < ACT_ACK_MAX_RETRIES {
                        tracing::warn!(
                            target: "spine.unix_socket",
                            act_instance_id = %act.act_instance_id,
                            attempt = attempt + 1,
                            "act_ack_timeout_retrying_dispatch"
                        );
                    }
                }

                if !acknowledged {
                    tracing::error!(
                        target: "spine.unix_socket",
                        session_id = session_id,
                        act_instance_id = %act.act_instance_id,
                        attempts = ACT_ACK_MAX_RETRIES + 1,
                        latency_ms = dispatch_started_at.elapsed().as_millis() as u64,
                        "act_dispatch_failed_after_ack_retries"
                    );
                    return Err(anyhow::anyhow!(
                        "failed to receive act_ack after retries for act_instance_id={}",
                        act.act_instance_id
                    ));
                }
            }

            Ok::<(), anyhow::Error>(())
        }
        .instrument(writer_span),
    );

    let mut lines = BufReader::new(read_half).lines();
    let mut auth_endpoint_id: Option<String> = None;
    let mut endpoint_proprioception_keys = BTreeSet::new();

    while let Some(line) = lines.next_line().await? {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        match parse_body_afferent_message(line) {
            Ok(message) => match message {
                InboundBodyMessage::Auth {
                    endpoint_name,
                    ns_descriptors,
                    proprioceptions,
                } => {
                    if auth_endpoint_id.is_some() {
                        tracing::warn!(
                            target: "spine.unix_socket",
                            "auth_ignored_endpoint_already_authenticated_on_session"
                        );
                        continue;
                    }
                    if endpoint_name.trim().is_empty() {
                        tracing::warn!(
                            target: "spine.unix_socket",
                            "auth_rejected_endpoint_name_cannot_be_empty"
                        );
                        continue;
                    }

                    let handle = match port.register_endpoint(adapter_id, &endpoint_name).await {
                        Ok(handle) => handle,
                        Err(err) => {
                            tracing::warn!(
                                target: "spine.unix_socket",
                                error = ?err,
                                "body_endpoint_registration_failed_during_auth"
                            );
                            continue;
                        }
                    };
                    auth_endpoint_id = Some(handle.body_endpoint_id.clone());
                    sessions
                        .lock()
                        .expect("lock poisoned")
                        .insert(handle.body_endpoint_id.clone(), outbound_tx.clone());

                    if let Err(err) = port
                        .add_ns_descriptors(&handle.body_endpoint_id, ns_descriptors)
                        .await
                    {
                        tracing::warn!(
                            target: "spine.unix_socket",
                            error = ?err,
                            "body_endpoint_ns_descriptor_registration_failed_during_auth"
                        );
                    }

                    let namespaced_entries = namespaced_body_proprioception_entries(
                        &handle.body_endpoint_id,
                        &proprioceptions,
                    );
                    endpoint_proprioception_keys.extend(namespaced_entries.keys().cloned());
                    emit_proprioception_patch(
                        &port,
                        namespaced_entries,
                        "auth_proprioception_patch",
                    )
                    .await;
                    emit_spine_topology_proprioception(&port).await;
                }
                InboundBodyMessage::ActAck { act_instance_id } => {
                    tracing::debug!(
                        target: "spine.unix_socket",
                        session_id = session_id,
                        act_instance_id = %act_instance_id,
                        "received_act_ack_from_body_endpoint"
                    );
                    if ack_tx.send(act_instance_id).is_err() {
                        tracing::warn!(
                            target: "spine.unix_socket",
                            "dropping_act_ack_because_writer_has_closed"
                        );
                    }
                }
                InboundBodyMessage::Unplug => {
                    if let Some(body_endpoint_id) = auth_endpoint_id.take() {
                        sessions
                            .lock()
                            .expect("lock poisoned")
                            .remove(&body_endpoint_id);
                        port.drop_endpoint(&body_endpoint_id).await;
                        let drop_keys = endpoint_proprioception_keys
                            .iter()
                            .cloned()
                            .collect::<Vec<_>>();
                        endpoint_proprioception_keys.clear();
                        emit_proprioception_drop(&port, drop_keys, "unplug_proprioception_drop")
                            .await;
                        emit_spine_topology_proprioception(&port).await;
                    }
                    break;
                }
                InboundBodyMessage::Sense(sense) => {
                    let Some(body_endpoint_id) = auth_endpoint_id.as_deref() else {
                        tracing::warn!(
                            target: "spine.unix_socket",
                            "sense_rejected_endpoint_must_auth_first"
                        );
                        continue;
                    };

                    // Adapter injects endpoint_id from authenticated endpoint binding.
                    let sense = Sense {
                        sense_instance_id: sense.sense_instance_id,
                        endpoint_id: body_endpoint_id.to_string(),
                        neural_signal_descriptor_id: sense.neural_signal_descriptor_id,
                        payload: sense.payload,
                        weight: sense.weight.clamp(0.0, 1.0),
                        act_instance_id: sense.act_instance_id,
                    };
                    if sense_tx.send(sense).is_err() {
                        break;
                    }
                }
                InboundBodyMessage::NewProprioceptions { entries } => {
                    let Some(body_endpoint_id) = auth_endpoint_id.as_deref() else {
                        tracing::warn!(
                            target: "spine.unix_socket",
                            "new_proprioceptions_rejected_endpoint_must_auth_first"
                        );
                        continue;
                    };

                    let namespaced_entries =
                        namespaced_body_proprioception_entries(body_endpoint_id, &entries);
                    endpoint_proprioception_keys.extend(namespaced_entries.keys().cloned());
                    emit_proprioception_patch(
                        &port,
                        namespaced_entries,
                        "runtime_proprioception_patch",
                    )
                    .await;
                }
                InboundBodyMessage::DropProprioceptions { keys } => {
                    let Some(body_endpoint_id) = auth_endpoint_id.as_deref() else {
                        tracing::warn!(
                            target: "spine.unix_socket",
                            "drop_proprioceptions_rejected_endpoint_must_auth_first"
                        );
                        continue;
                    };

                    let namespaced_keys =
                        namespaced_body_proprioception_drop_keys(body_endpoint_id, &keys);
                    for key in &namespaced_keys {
                        endpoint_proprioception_keys.remove(key);
                    }
                    emit_proprioception_drop(&port, namespaced_keys, "runtime_proprioception_drop")
                        .await;
                }
            },
            Err(err) => {
                tracing::warn!(
                    target: "spine.unix_socket",
                    error = %err,
                    "invalid_afferent_message"
                );
            }
        }
    }

    if let Some(body_endpoint_id) = auth_endpoint_id.take() {
        sessions
            .lock()
            .expect("lock poisoned")
            .remove(&body_endpoint_id);
        port.drop_endpoint(&body_endpoint_id).await;
    }

    if !endpoint_proprioception_keys.is_empty() {
        let drop_keys = endpoint_proprioception_keys.into_iter().collect::<Vec<_>>();
        emit_proprioception_drop(&port, drop_keys, "disconnect_proprioception_drop").await;
    }
    emit_spine_topology_proprioception(&port).await;

    writer_task.await??;

    Ok(())
}
