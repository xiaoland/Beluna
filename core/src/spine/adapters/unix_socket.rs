use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::ErrorKind,
    os::unix::fs::FileTypeExt,
    path::{Path, PathBuf},
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{UnixListener, UnixStream},
    sync::mpsc,
    time::{Duration, Instant, timeout},
};
use tokio_util::sync::CancellationToken;
use tracing::Instrument;

use crate::{
    spine::{EndpointBinding, SpineControlPort, runtime::Spine, types::NeuralSignalDescriptor},
    types::{Act, Sense, default_sense_weight, is_uuid_v4, is_uuid_v7},
};

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
    spine: &Arc<Spine>,
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
    spine.apply_proprioception_patch(entries).await;
}

async fn emit_proprioception_drop(spine: &Arc<Spine>, keys: Vec<String>, reason: &'static str) {
    if keys.is_empty() {
        return;
    }

    tracing::debug!(
        target: "spine.unix_socket",
        reason = reason,
        "apply_proprioception_drop"
    );
    spine.apply_proprioception_drop(keys).await;
}

async fn emit_spine_topology_proprioception(spine: &Arc<Spine>) {
    spine.refresh_topology_proprioception().await;
}

pub struct UnixSocketAdapter {
    pub socket_path: PathBuf,
    pub adapter_id: u64,
}

impl UnixSocketAdapter {
    pub fn new(socket_path: PathBuf, adapter_id: u64) -> Self {
        Self {
            socket_path,
            adapter_id,
        }
    }

    pub async fn run(&self, spine: Arc<Spine>, shutdown: CancellationToken) -> Result<()> {
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
                            let spine = Arc::clone(&spine);
                            let adapter_id = self.adapter_id;
                            let session_span = tracing::info_span!(
                                target: "spine.unix_socket",
                                "body_endpoint_session",
                                adapter_id = adapter_id
                            );
                            tokio::spawn(async move {
                                if let Err(err) =
                                    handle_body_endpoint(stream, spine, adapter_id)
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

const ACT_ACK_TIMEOUT_MS: u64 = 1_500;
const ACT_ACK_MAX_RETRIES: usize = 2;

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
    skip(stream, spine),
    fields(adapter_id = adapter_id)
)]
async fn handle_body_endpoint(
    stream: UnixStream,
    spine: Arc<Spine>,
    adapter_id: u64,
) -> Result<()> {
    let (read_half, mut write_half) = stream.into_split();

    let (outbound_tx, mut outbound_rx) = mpsc::unbounded_channel::<Act>();
    let channel_id = spine.on_adapter_channel_open(adapter_id, outbound_tx);
    let (ack_tx, mut ack_rx) = mpsc::unbounded_channel::<String>();

    let writer_span = tracing::debug_span!(
        target: "spine.unix_socket",
        "body_endpoint_writer_task",
        channel_id = channel_id
    );
    let writer_task = tokio::spawn(
        async move {
            while let Some(act) = outbound_rx.recv().await {
                let dispatch_started_at = Instant::now();
                tracing::debug!(
                    target: "spine.unix_socket",
                    channel_id = channel_id,
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
                            channel_id = channel_id,
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
                        channel_id = channel_id,
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
                            "auth_ignored_endpoint_already_authenticated_on_channel"
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

                    let handle = match spine
                        .add_endpoint(&endpoint_name, EndpointBinding::AdapterChannel(channel_id))
                    {
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

                    if let Err(err) = spine
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
                        &spine,
                        namespaced_entries,
                        "auth_proprioception_patch",
                    )
                    .await;
                    emit_spine_topology_proprioception(&spine).await;
                }
                InboundBodyMessage::ActAck { act_instance_id } => {
                    tracing::debug!(
                        target: "spine.unix_socket",
                        channel_id = channel_id,
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
                    if let Some(body_endpoint_id) = auth_endpoint_id.as_deref() {
                        spine.remove_endpoint(body_endpoint_id).await;
                        let drop_keys = endpoint_proprioception_keys
                            .iter()
                            .cloned()
                            .collect::<Vec<_>>();
                        endpoint_proprioception_keys.clear();
                        emit_proprioception_drop(&spine, drop_keys, "unplug_proprioception_drop")
                            .await;
                        emit_spine_topology_proprioception(&spine).await;
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
                    spine.publish_sense(sense).await;
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
                        &spine,
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
                    emit_proprioception_drop(
                        &spine,
                        namespaced_keys,
                        "runtime_proprioception_drop",
                    )
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

    spine.on_adapter_channel_closed(channel_id).await;

    if !endpoint_proprioception_keys.is_empty() {
        let drop_keys = endpoint_proprioception_keys.into_iter().collect::<Vec<_>>();
        emit_proprioception_drop(&spine, drop_keys, "disconnect_proprioception_drop").await;
    }
    emit_spine_topology_proprioception(&spine).await;

    writer_task.await??;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{
        spine::{
            adapters::unix_socket::{
                InboundBodyMessage, NdjsonEnvelope, OutboundActBody,
                encode_body_egress_act_message, parse_body_afferent_message,
            },
            types::NeuralSignalDescriptor,
        },
        types::{Act, NeuralSignalType},
    };

    #[test]
    fn accepts_sense_message() {
        let parsed = parse_body_afferent_message(
            r#"{"method":"sense","id":"2f8daebf-f529-4ea4-b322-7df109e86d66","timestamp":1739500000000,"body":{"sense_instance_id":"41f25f33-99f5-4250-99c3-020f8a92e199","neural_signal_descriptor_id":"chat.message","payload":{"v":1}}}"#,
        )
        .expect("sense message should parse");
        assert!(matches!(parsed, InboundBodyMessage::Sense(_)));
    }

    #[test]
    fn accepts_correlated_sense_message_without_legacy_echo_fields() {
        let parsed = parse_body_afferent_message(
            r#"{"method":"sense","id":"2f8daebf-f529-4ea4-b322-7df109e86d66","timestamp":1739500000000,"body":{"sense_instance_id":"9d60f110-af6d-42cb-853b-bf6f6ce6f0dc","neural_signal_descriptor_id":"present.message.result","payload":{"kind":"present_message_result"},"metadata":{"act_instance_id":"0194f1f3-cc2f-7aa7-8d4c-486f9f2f7c0a"}}}"#,
        )
        .expect("correlated sense should parse");
        assert!(matches!(parsed, InboundBodyMessage::Sense(_)));
    }

    #[test]
    fn accepts_sense_message_without_metadata() {
        let parsed = parse_body_afferent_message(
            r#"{"method":"sense","id":"2f8daebf-f529-4ea4-b322-7df109e86d66","timestamp":1739500000000,"body":{"sense_instance_id":"9d60f110-af6d-42cb-853b-bf6f6ce6f0dc","neural_signal_descriptor_id":"present.message.result","payload":{"kind":"present_message_result"}}}"#,
        )
        .expect("sense should parse");

        match parsed {
            InboundBodyMessage::Sense(sense) => {
                assert_eq!(sense.metadata, serde_json::json!({}));
            }
            _ => panic!("expected sense message"),
        }
    }

    #[test]
    fn rejects_correlated_sense_message_invalid_act_instance_id() {
        assert!(
            parse_body_afferent_message(
                r#"{"method":"sense","id":"2f8daebf-f529-4ea4-b322-7df109e86d66","timestamp":1739500000000,"body":{"sense_instance_id":"01234567-89ab-4cde-8f01-23456789abcd","neural_signal_descriptor_id":"present.message.result","payload":{"kind":"present_message_result"},"metadata":{"act_instance_id":"not-a-uuid-v7"}}}"#
            )
            .is_err()
        );
    }

    #[test]
    fn rejects_sense_message_with_non_object_metadata() {
        assert!(
            parse_body_afferent_message(
                r#"{"method":"sense","id":"2f8daebf-f529-4ea4-b322-7df109e86d66","timestamp":1739500000000,"body":{"sense_instance_id":"01234567-89ab-4cde-8f01-23456789abcd","neural_signal_descriptor_id":"present.message.result","payload":{"kind":"present_message_result"},"metadata":"invalid"}}"#
            )
            .is_err()
        );
    }

    #[test]
    fn rejects_direction_violation_for_inbound_act_method() {
        assert!(
            parse_body_afferent_message(
                r#"{"method":"act","id":"2f8daebf-f529-4ea4-b322-7df109e86d66","timestamp":1739500000000,"body":{}}"#
            )
            .is_err()
        );
    }

    #[test]
    fn act_encoding_contains_act_and_target_fields() {
        let act = Act {
            act_instance_id: "0194f1f3-cc2f-7aa7-8d4c-486f9f2f7c0a".to_string(),
            endpoint_id: "macos-app.01".to_string(),
            neural_signal_descriptor_id: "present.message".to_string(),
            might_emit_sense_ids: vec![],
            payload: serde_json::json!({"ok": true}),
        };

        let encoded = encode_body_egress_act_message(&act).expect("act should encode");
        let message: NdjsonEnvelope<OutboundActBody> =
            serde_json::from_str(encoded.trim_end()).expect("message should decode");
        assert_eq!(message.method, "act");
        assert!(uuid::Uuid::parse_str(&message.id).is_ok());
        assert!(message.timestamp > 0);
        assert_eq!(
            message.body.act.act_instance_id,
            "0194f1f3-cc2f-7aa7-8d4c-486f9f2f7c0a"
        );
        assert_eq!(
            message.body.act.neural_signal_descriptor_id,
            "present.message"
        );
        assert_eq!(message.body.act.payload, serde_json::json!({"ok": true}));
        assert!(encoded.contains("\"method\":\"act\""));
        assert!(encoded.contains("\"body\":{"));
        assert!(encoded.contains("\"act_instance_id\":\"0194f1f3-cc2f-7aa7-8d4c-486f9f2f7c0a\""));
        assert!(encoded.contains("\"neural_signal_descriptor_id\":\"present.message\""));
    }

    #[test]
    fn accepts_auth_message_with_ns_descriptors() {
        let descriptor: NeuralSignalDescriptor = serde_json::from_value(serde_json::json!({
            "type": "act",
            "endpoint_id": "macos-app.01",
            "neural_signal_descriptor_id": "present.message",
            "payload_schema": {"type":"object"}
        }))
        .expect("descriptor should decode");
        assert_eq!(descriptor.r#type, NeuralSignalType::Act);

        let wire = serde_json::json!({
            "method": "auth",
            "id": "2f8daebf-f529-4ea4-b322-7df109e86d66",
            "timestamp": 1739500000000_u64,
            "body": {
                "endpoint_name": "macos-app",
                "ns_descriptors": [descriptor]
            }
        });

        let parsed = parse_body_afferent_message(&wire.to_string()).expect("auth should parse");
        assert!(matches!(parsed, InboundBodyMessage::Auth { .. }));
    }

    #[test]
    fn accepts_auth_message_without_ns_descriptors() {
        let wire = serde_json::json!({
            "method": "auth",
            "id": "2f8daebf-f529-4ea4-b322-7df109e86d66",
            "timestamp": 1739500000000_u64,
            "body": {
                "endpoint_name": "cli"
            }
        });

        let parsed = parse_body_afferent_message(&wire.to_string()).expect("auth should parse");
        match parsed {
            InboundBodyMessage::Auth {
                endpoint_name,
                ns_descriptors,
                proprioceptions,
            } => {
                assert_eq!(endpoint_name, "cli");
                assert!(ns_descriptors.is_empty());
                assert!(proprioceptions.is_empty());
            }
            _ => panic!("expected auth message"),
        }
    }

    #[test]
    fn accepts_new_proprioceptions_message() {
        let wire = serde_json::json!({
            "method": "new_proprioceptions",
            "id": "2f8daebf-f529-4ea4-b322-7df109e86d66",
            "timestamp": 1739500000000_u64,
            "body": {
                "entries": {
                    "platform": "macos",
                    "window": "1440x900"
                }
            }
        });

        let parsed = parse_body_afferent_message(&wire.to_string())
            .expect("new_proprioceptions should parse");
        assert!(matches!(
            parsed,
            InboundBodyMessage::NewProprioceptions { .. }
        ));
    }

    #[test]
    fn accepts_drop_proprioceptions_message() {
        let wire = serde_json::json!({
            "method": "drop_proprioceptions",
            "id": "2f8daebf-f529-4ea4-b322-7df109e86d66",
            "timestamp": 1739500000000_u64,
            "body": {
                "keys": ["platform", "window"]
            }
        });

        let parsed = parse_body_afferent_message(&wire.to_string())
            .expect("drop_proprioceptions should parse");
        assert!(matches!(
            parsed,
            InboundBodyMessage::DropProprioceptions { .. }
        ));
    }

    #[test]
    fn rejects_non_ws6_methods() {
        for method in ["new_ns_descriptors", "drop_ns_descriptors"] {
            let wire = serde_json::json!({
                "method": method,
                "id": "2f8daebf-f529-4ea4-b322-7df109e86d66",
                "timestamp": 1739500000000_u64,
                "body": {}
            });
            assert!(parse_body_afferent_message(&wire.to_string()).is_err());
        }
    }
}
