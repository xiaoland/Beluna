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
};
use tokio_util::sync::CancellationToken;

use crate::{
    ingress::SenseIngress,
    runtime_types::{Act, CapabilityDropPatch, CapabilityPatch, Sense, SenseDatum},
    spine::{runtime::Spine, types::EndpointCapabilityDescriptor},
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

fn parse_body_ingress_message(line: &str) -> Result<InboundBodyMessage, serde_json::Error> {
    let wire: NdjsonEnvelope<serde_json::Value> = serde_json::from_str(line)?;
    if uuid::Uuid::parse_str(&wire.id).is_err() {
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
            InboundBodyMessage::Sense(SenseDatum {
                sense_id: body.sense_id,
                source: body.source,
                payload,
            })
        }
        "unplug" => InboundBodyMessage::Unplug,
        "act" => {
            return Err(invalid_correlated_sense_error(
                "direction violation: endpoint cannot send method 'act'",
            ));
        }
        _ => {
            return Err(invalid_correlated_sense_error(
                "unsupported method, expected one of: auth|sense|unplug",
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

    if act_id
        .as_str()
        .map(|value| value.trim().is_empty())
        .unwrap_or(true)
    {
        return Err(invalid_correlated_sense_error(
            "act_id must be a non-empty string",
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
        ingress: SenseIngress,
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
                            let ingress = ingress.clone();
                            let spine = Arc::clone(&spine);
                            let adapter_id = self.adapter_id;
                            tokio::spawn(async move {
                                if let Err(err) =
                                    handle_body_endpoint(stream, ingress, spine, adapter_id)
                                        .await
                                {
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
    spine: Arc<Spine>,
    adapter_id: u64,
) -> Result<()> {
    let (read_half, mut write_half) = stream.into_split();

    let (outbound_tx, mut outbound_rx) = mpsc::unbounded_channel::<Act>();
    let channel_id = spine.on_adapter_channel_open(adapter_id, outbound_tx);

    let writer_task = tokio::spawn(async move {
        while let Some(act) = outbound_rx.recv().await {
            let encoded = encode_body_egress_act_message(&act)?;
            write_half.write_all(encoded.as_bytes()).await?;
            write_half.flush().await?;
        }

        Ok::<(), anyhow::Error>(())
    });

    let mut lines = BufReader::new(read_half).lines();
    let mut auth_endpoint_id: Option<uuid::Uuid> = None;

    while let Some(line) = lines.next_line().await? {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        match parse_body_ingress_message(line) {
            Ok(message) => match message {
                InboundBodyMessage::Auth {
                    endpoint_name,
                    capabilities,
                } => {
                    if auth_endpoint_id.is_some() {
                        eprintln!("auth ignored: endpoint already authenticated on channel");
                        continue;
                    }
                    if endpoint_name.trim().is_empty() {
                        eprintln!("auth rejected: endpoint_name cannot be empty");
                        continue;
                    }

                    let handle = match spine.new_body_endpoint(channel_id, &endpoint_name) {
                        Ok(handle) => handle,
                        Err(err) => {
                            eprintln!("body endpoint registration failed during auth: {err:#}");
                            continue;
                        }
                    };
                    auth_endpoint_id = Some(handle.body_endpoint_id);

                    let mut registered_entries = Vec::new();
                    for descriptor in capabilities {
                        match spine
                            .register_body_endpoint_capability(handle.body_endpoint_id, descriptor)
                        {
                            Ok(descriptor) => registered_entries.push(descriptor),
                            Err(err) => {
                                eprintln!(
                                    "body endpoint capability registration failed during auth: {err:#}"
                                );
                            }
                        }
                    }

                    if !registered_entries.is_empty()
                        && let Err(err) = ingress
                            .send(Sense::NewCapabilities(CapabilityPatch {
                                entries: registered_entries,
                            }))
                            .await
                    {
                        eprintln!("dropping capability patch after auth: {err}");
                    }
                }
                InboundBodyMessage::Unplug => {
                    if let Some(body_endpoint_id) = auth_endpoint_id {
                        let routes = spine.remove_body_endpoint(body_endpoint_id);
                        if !routes.is_empty()
                            && let Err(err) = ingress
                                .send(Sense::DropCapabilities(CapabilityDropPatch { routes }))
                                .await
                        {
                            eprintln!("dropping capability drop after unplug: {err}");
                        }
                    }
                    break;
                }
                InboundBodyMessage::Sense(sense) => {
                    if auth_endpoint_id.is_none() {
                        eprintln!("sense rejected: endpoint must auth first");
                        continue;
                    }
                    if let Err(err) = ingress.send(Sense::Domain(sense)).await {
                        eprintln!("dropping sense due to closed ingress: {err}");
                    }
                }
            },
            Err(err) => {
                eprintln!("invalid ingress message: {err}");
            }
        }
    }

    let routes = spine.on_adapter_channel_closed(channel_id);
    if !routes.is_empty() {
        let _ = ingress
            .send(Sense::DropCapabilities(CapabilityDropPatch { routes }))
            .await;
    }

    writer_task.await??;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{
        runtime_types::{Act, RequestedResources},
        spine::{
            adapters::unix_socket::{
                InboundBodyMessage, NdjsonEnvelope, OutboundActBody,
                encode_body_egress_act_message, parse_body_ingress_message,
            },
            types::EndpointCapabilityDescriptor,
        },
    };

    #[test]
    fn accepts_sense_message() {
        let parsed = parse_body_ingress_message(
            r#"{"method":"sense","id":"2f8daebf-f529-4ea4-b322-7df109e86d66","timestamp":1739500000000,"body":{"sense_id":"s1","source":"sensor","payload":{"v":1}}}"#,
        )
        .expect("sense message should parse");
        assert!(matches!(parsed, InboundBodyMessage::Sense(_)));
    }

    #[test]
    fn accepts_correlated_sense_message_with_required_echo_fields() {
        let parsed = parse_body_ingress_message(
            r#"{"method":"sense","id":"2f8daebf-f529-4ea4-b322-7df109e86d66","timestamp":1739500000000,"body":{"sense_id":"s2","source":"body","payload":{"kind":"present_message_result","act_id":"act:1","capability_instance_id":"chat.1","endpoint_id":"macos-app.01","capability_id":"present.message"}}}"#,
        )
        .expect("correlated sense should parse");
        assert!(matches!(parsed, InboundBodyMessage::Sense(_)));
    }

    #[test]
    fn accepts_legacy_correlated_sense_message_with_neural_signal_alias() {
        let parsed = parse_body_ingress_message(
            r#"{"method":"sense","id":"2f8daebf-f529-4ea4-b322-7df109e86d66","timestamp":1739500000000,"body":{"sense_id":"s2","source":"body","payload":{"kind":"present_message_result","neural_signal_id":"act:legacy","capability_instance_id":"chat.1","endpoint_id":"macos-app.01","capability_id":"present.message"}}}"#,
        )
        .expect("legacy correlated sense should parse");

        match parsed {
            InboundBodyMessage::Sense(sense) => {
                assert_eq!(
                    sense.payload.get("act_id"),
                    Some(&serde_json::json!("act:legacy"))
                );
            }
            _ => panic!("expected sense message"),
        }
    }

    #[test]
    fn rejects_correlated_sense_message_missing_echo_fields() {
        assert!(
            parse_body_ingress_message(
                r#"{"method":"sense","id":"2f8daebf-f529-4ea4-b322-7df109e86d66","timestamp":1739500000000,"body":{"sense_id":"s3","source":"body","payload":{"kind":"present_message_result","act_id":"act:1","endpoint_id":"macos-app.01","capability_id":"present.message"}}}"#
            )
            .is_err()
        );
    }

    #[test]
    fn rejects_direction_violation_for_inbound_act_method() {
        assert!(
            parse_body_ingress_message(
                r#"{"method":"act","id":"2f8daebf-f529-4ea4-b322-7df109e86d66","timestamp":1739500000000,"body":{}}"#
            )
            .is_err()
        );
    }

    #[test]
    fn act_encoding_contains_act_and_target_fields() {
        let act = Act {
            act_id: "act:1".to_string(),
            based_on: vec!["s1".to_string()],
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
        assert_eq!(message.body.act.act_id, "act:1");
        assert_eq!(message.body.act.capability_id, "present.message");
        assert_eq!(message.body.act.requested_resources.survival_micro, 120);
        assert_eq!(message.body.act.requested_resources.time_ms, 100);
        assert_eq!(message.body.act.requested_resources.io_units, 1);
        assert_eq!(message.body.act.requested_resources.token_units, 64);
        assert!(encoded.contains("\"method\":\"act\""));
        assert!(encoded.contains("\"body\":{"));
        assert!(encoded.contains("\"act_id\":\"act:1\""));
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

        let parsed = parse_body_ingress_message(&wire.to_string()).expect("auth should parse");
        assert!(matches!(parsed, InboundBodyMessage::Auth { .. }));
    }
}
