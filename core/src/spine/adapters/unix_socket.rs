use std::{
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
    afferent_pathway::SenseAfferentPathway,
    cortex::{is_uuid_v4, is_uuid_v7},
    spine::{EndpointBinding, runtime::Spine, types::EndpointCapabilityDescriptor},
    types::{Act, CapabilityDropPatch, CapabilityPatch, Sense, SenseDatum},
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
        capabilities: Vec<EndpointCapabilityDescriptor>,
    },
    Sense(SenseDatum),
    ActAck {
        act_id: String,
    },
    Unplug,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct InboundAuthBody {
    endpoint_name: String,
    #[serde(default)]
    capabilities: Vec<EndpointCapabilityDescriptor>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct InboundSenseBody {
    sense_id: String,
    source: String,
    payload: serde_json::Value,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct InboundActAckBody {
    act_id: String,
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
                capabilities: body.capabilities,
            }
        }
        "sense" => {
            let body: InboundSenseBody = decode_envelope_body(wire.body)?;
            let mut payload = body.payload;
            normalize_correlated_sense_payload(&mut payload);
            validate_correlated_sense_payload(&payload)?;
            if !is_uuid_v4(&body.sense_id) {
                return Err(invalid_correlated_sense_error(
                    "sense_id must be a valid uuid-v4 string",
                ));
            }
            InboundBodyMessage::Sense(SenseDatum {
                sense_id: body.sense_id,
                source: body.source,
                payload,
            })
        }
        "act_ack" => {
            let body: InboundActAckBody = decode_envelope_body(wire.body)?;
            if !is_uuid_v7(&body.act_id) {
                return Err(invalid_correlated_sense_error(
                    "act_ack.act_id must be a valid uuid-v7 string",
                ));
            }
            InboundBodyMessage::ActAck {
                act_id: body.act_id,
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
                "unsupported method, expected one of: auth|sense|act_ack|unplug",
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

fn normalize_correlated_sense_payload(payload: &mut serde_json::Value) {
    let Some(object) = payload.as_object_mut() else {
        return;
    };

    if object.contains_key("act_id") {
        return;
    }

    let Some(neural_signal_id) = object
        .get("neural_signal_id")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return;
    };

    object.insert(
        "act_id".to_string(),
        serde_json::Value::String(neural_signal_id.to_string()),
    );
}

fn validate_correlated_sense_payload(payload: &serde_json::Value) -> Result<(), serde_json::Error> {
    let Some(object) = payload.as_object() else {
        return Ok(());
    };

    let Some(act_id) = object.get("act_id") else {
        return Ok(());
    };

    if !act_id.as_str().map(is_uuid_v7).unwrap_or(false) {
        return Err(invalid_correlated_sense_error(
            "act_id must be a valid uuid-v7 string",
        ));
    }

    for field in ["capability_instance_id", "endpoint_id", "capability_id"] {
        let valid = object
            .get(field)
            .and_then(|value| value.as_str())
            .map(|value| !value.trim().is_empty())
            .unwrap_or(false);
        if !valid {
            return Err(invalid_correlated_sense_error(&format!(
                "correlated sense missing required field '{}'",
                field
            )));
        }
    }

    Ok(())
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

    pub async fn run(
        &self,
        afferent_pathway: SenseAfferentPathway,
        spine: Arc<Spine>,
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
                            let afferent_pathway = afferent_pathway.clone();
                            let spine = Arc::clone(&spine);
                            let adapter_id = self.adapter_id;
                            let session_span = tracing::info_span!(
                                target: "spine.unix_socket",
                                "body_endpoint_session",
                                adapter_id = adapter_id
                            );
                            tokio::spawn(async move {
                                if let Err(err) =
                                    handle_body_endpoint(stream, afferent_pathway, spine, adapter_id)
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
    act_id: &str,
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
            Ok(Some(received_act_id)) => {
                if received_act_id == act_id {
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
    skip(stream, afferent_pathway, spine),
    fields(adapter_id = adapter_id)
)]
async fn handle_body_endpoint(
    stream: UnixStream,
    afferent_pathway: SenseAfferentPathway,
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
                let mut acknowledged = false;
                for attempt in 0..=ACT_ACK_MAX_RETRIES {
                    let encoded = encode_body_egress_act_message(&act)?;
                    write_half.write_all(encoded.as_bytes()).await?;
                    write_half.flush().await?;

                    if wait_for_act_ack(&mut ack_rx, &act.act_id, ACT_ACK_TIMEOUT_MS).await {
                        acknowledged = true;
                        break;
                    }

                    if attempt < ACT_ACK_MAX_RETRIES {
                        tracing::warn!(
                            target: "spine.unix_socket",
                            act_id = %act.act_id,
                            attempt = attempt + 1,
                            "act_ack_timeout_retrying_dispatch"
                        );
                    }
                }

                if !acknowledged {
                    return Err(anyhow::anyhow!(
                        "failed to receive act_ack after retries for act_id={}",
                        act.act_id
                    ));
                }
            }

            Ok::<(), anyhow::Error>(())
        }
        .instrument(writer_span),
    );

    let mut lines = BufReader::new(read_half).lines();
    let mut auth_endpoint_id: Option<uuid::Uuid> = None;

    while let Some(line) = lines.next_line().await? {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        match parse_body_afferent_message(line) {
            Ok(message) => match message {
                InboundBodyMessage::Auth {
                    endpoint_name,
                    capabilities,
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

                    let handle = match spine.add_endpoint(
                        &endpoint_name,
                        EndpointBinding::AdapterChannel(channel_id),
                        vec![],
                    ) {
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
                    auth_endpoint_id = Some(handle.body_endpoint_id);

                    let registered_entries =
                        match spine.add_capabilities(handle.body_endpoint_id, capabilities) {
                            Ok(entries) => entries,
                            Err(err) => {
                                tracing::warn!(
                                    target: "spine.unix_socket",
                                    error = ?err,
                                    "body_endpoint_capability_registration_failed_during_auth"
                                );
                                Vec::new()
                            }
                        };

                    if !registered_entries.is_empty()
                        && let Err(err) = afferent_pathway
                            .send(Sense::NewCapabilities(CapabilityPatch {
                                entries: registered_entries,
                            }))
                            .await
                    {
                        tracing::warn!(
                            target: "spine.unix_socket",
                            error = %err,
                            "dropping_capability_patch_after_auth"
                        );
                    }
                }
                InboundBodyMessage::ActAck { act_id } => {
                    if ack_tx.send(act_id).is_err() {
                        tracing::warn!(
                            target: "spine.unix_socket",
                            "dropping_act_ack_because_writer_has_closed"
                        );
                    }
                }
                InboundBodyMessage::Unplug => {
                    if let Some(body_endpoint_id) = auth_endpoint_id {
                        let routes = spine.remove_endpoint(body_endpoint_id);
                        if !routes.is_empty()
                            && let Err(err) = afferent_pathway
                                .send(Sense::DropCapabilities(CapabilityDropPatch { routes }))
                                .await
                        {
                            tracing::warn!(
                                target: "spine.unix_socket",
                                error = %err,
                                "dropping_capability_drop_after_unplug"
                            );
                        }
                    }
                    break;
                }
                InboundBodyMessage::Sense(sense) => {
                    if auth_endpoint_id.is_none() {
                        tracing::warn!(
                            target: "spine.unix_socket",
                            "sense_rejected_endpoint_must_auth_first"
                        );
                        continue;
                    }
                    if let Err(err) = afferent_pathway.send(Sense::Domain(sense)).await {
                        tracing::warn!(
                            target: "spine.unix_socket",
                            error = %err,
                            "dropping_sense_due_to_closed_afferent_pathway"
                        );
                    }
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

    let routes = spine.on_adapter_channel_closed(channel_id);
    if !routes.is_empty() {
        let _ = afferent_pathway
            .send(Sense::DropCapabilities(CapabilityDropPatch { routes }))
            .await;
    }

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
            types::EndpointCapabilityDescriptor,
        },
        types::{Act, RequestedResources},
    };

    #[test]
    fn accepts_sense_message() {
        let parsed = parse_body_afferent_message(
            r#"{"method":"sense","id":"2f8daebf-f529-4ea4-b322-7df109e86d66","timestamp":1739500000000,"body":{"sense_id":"41f25f33-99f5-4250-99c3-020f8a92e199","source":"sensor","payload":{"v":1}}}"#,
        )
        .expect("sense message should parse");
        assert!(matches!(parsed, InboundBodyMessage::Sense(_)));
    }

    #[test]
    fn accepts_correlated_sense_message_with_required_echo_fields() {
        let parsed = parse_body_afferent_message(
            r#"{"method":"sense","id":"2f8daebf-f529-4ea4-b322-7df109e86d66","timestamp":1739500000000,"body":{"sense_id":"9d60f110-af6d-42cb-853b-bf6f6ce6f0dc","source":"body","payload":{"kind":"present_message_result","act_id":"0194f1f3-cc2f-7aa7-8d4c-486f9f2f7c0a","capability_instance_id":"chat.1","endpoint_id":"macos-app.01","capability_id":"present.message"}}}"#,
        )
        .expect("correlated sense should parse");
        assert!(matches!(parsed, InboundBodyMessage::Sense(_)));
    }

    #[test]
    fn accepts_legacy_correlated_sense_message_with_neural_signal_alias() {
        let parsed = parse_body_afferent_message(
            r#"{"method":"sense","id":"2f8daebf-f529-4ea4-b322-7df109e86d66","timestamp":1739500000000,"body":{"sense_id":"9d60f110-af6d-42cb-853b-bf6f6ce6f0dc","source":"body","payload":{"kind":"present_message_result","neural_signal_id":"0194f1f3-cc2f-7aa7-8d4c-486f9f2f7c0a","capability_instance_id":"chat.1","endpoint_id":"macos-app.01","capability_id":"present.message"}}}"#,
        )
        .expect("legacy correlated sense should parse");

        match parsed {
            InboundBodyMessage::Sense(sense) => {
                assert_eq!(
                    sense.payload.get("act_id"),
                    Some(&serde_json::json!("0194f1f3-cc2f-7aa7-8d4c-486f9f2f7c0a"))
                );
            }
            _ => panic!("expected sense message"),
        }
    }

    #[test]
    fn rejects_correlated_sense_message_missing_echo_fields() {
        assert!(
            parse_body_afferent_message(
                r#"{"method":"sense","id":"2f8daebf-f529-4ea4-b322-7df109e86d66","timestamp":1739500000000,"body":{"sense_id":"01234567-89ab-4cde-8f01-23456789abcd","source":"body","payload":{"kind":"present_message_result","act_id":"0194f1f3-cc2f-7aa7-8d4c-486f9f2f7c0a","endpoint_id":"macos-app.01","capability_id":"present.message"}}}"#
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
            act_id: "0194f1f3-cc2f-7aa7-8d4c-486f9f2f7c0a".to_string(),
            based_on: vec!["41f25f33-99f5-4250-99c3-020f8a92e199".to_string()],
            body_endpoint_name: "macos-app.01".to_string(),
            capability_id: "present.message".to_string(),
            capability_instance_id: "chat.instance".to_string(),
            normalized_payload: serde_json::json!({"ok": true}),
            requested_resources: RequestedResources {
                survival_micro: 120,
                time_ms: 100,
                io_units: 1,
                token_units: 64,
            },
        };

        let encoded = encode_body_egress_act_message(&act).expect("act should encode");
        let message: NdjsonEnvelope<OutboundActBody> =
            serde_json::from_str(encoded.trim_end()).expect("message should decode");
        assert_eq!(message.method, "act");
        assert!(uuid::Uuid::parse_str(&message.id).is_ok());
        assert!(message.timestamp > 0);
        assert_eq!(
            message.body.act.act_id,
            "0194f1f3-cc2f-7aa7-8d4c-486f9f2f7c0a"
        );
        assert_eq!(message.body.act.capability_id, "present.message");
        assert_eq!(message.body.act.requested_resources.survival_micro, 120);
        assert_eq!(message.body.act.requested_resources.time_ms, 100);
        assert_eq!(message.body.act.requested_resources.io_units, 1);
        assert_eq!(message.body.act.requested_resources.token_units, 64);
        assert!(encoded.contains("\"method\":\"act\""));
        assert!(encoded.contains("\"body\":{"));
        assert!(encoded.contains("\"act_id\":\"0194f1f3-cc2f-7aa7-8d4c-486f9f2f7c0a\""));
        assert!(encoded.contains("\"capability_id\":\"present.message\""));
    }

    #[test]
    fn accepts_auth_message_with_capabilities() {
        let descriptor: EndpointCapabilityDescriptor = serde_json::from_value(serde_json::json!({
            "route": {
                "endpoint_id": "macos-app.01",
                "capability_id": "present.message"
            },
            "payload_schema": {"type":"object"},
            "max_payload_bytes": 1024,
            "default_cost": {
                "survival_micro": 0,
                "time_ms": 0,
                "io_units": 0,
                "token_units": 0
            },
            "metadata": {}
        }))
        .expect("descriptor should decode");

        let wire = serde_json::json!({
            "method": "auth",
            "id": "2f8daebf-f529-4ea4-b322-7df109e86d66",
            "timestamp": 1739500000000_u64,
            "body": {
                "endpoint_name": "macos-app",
                "capabilities": [descriptor]
            }
        });

        let parsed = parse_body_afferent_message(&wire.to_string()).expect("auth should parse");
        assert!(matches!(parsed, InboundBodyMessage::Auth { .. }));
    }

    #[test]
    fn accepts_auth_message_without_capabilities() {
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
                capabilities,
            } => {
                assert_eq!(endpoint_name, "cli");
                assert!(capabilities.is_empty());
            }
            _ => panic!("expected auth message"),
        }
    }

    #[test]
    fn rejects_non_ws6_methods() {
        for method in ["new_capabilities", "drop_capabilities"] {
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
