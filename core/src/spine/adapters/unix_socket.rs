use std::{
    fs,
    io::ErrorKind,
    os::unix::fs::FileTypeExt,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{UnixListener, UnixStream},
    sync::mpsc,
};
use tokio_util::sync::CancellationToken;

use crate::{
    ingress::SenseIngress,
    runtime_types::{
        Act, CapabilityDropPatch, CapabilityPatch, RequestedResources, Sense, SenseDatum,
    },
    spine::{
        registry::InMemoryEndpointRegistry,
        types::{EndpointCapabilityDescriptor, RouteKey},
    },
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum BodyEgressMessage {
    Act {
        act: Act,
        action: LegacyAdmittedAction,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct LegacyAdmittedAction {
    neural_signal_id: String,
    capability_instance_id: String,
    endpoint_id: String,
    capability_id: String,
    normalized_payload: serde_json::Value,
    reserved_cost: LegacyCostVector,
}

impl From<&Act> for LegacyAdmittedAction {
    fn from(act: &Act) -> Self {
        Self {
            neural_signal_id: act.act_id.clone(),
            capability_instance_id: act.capability_instance_id.clone(),
            endpoint_id: act.endpoint_id.clone(),
            capability_id: act.capability_id.clone(),
            normalized_payload: act.normalized_payload.clone(),
            reserved_cost: LegacyCostVector::from(&act.requested_resources),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct LegacyCostVector {
    survival_micro: i64,
    time_ms: u64,
    io_units: u64,
    token_units: u64,
}

impl From<&RequestedResources> for LegacyCostVector {
    fn from(resources: &RequestedResources) -> Self {
        Self {
            survival_micro: resources.survival_micro,
            time_ms: resources.time_ms,
            io_units: resources.io_units,
            token_units: resources.token_units,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum InboundBodyMessage {
    Sense(SenseDatum),
    NewCapabilities(CapabilityPatch),
    DropCapabilities(CapabilityDropPatch),
    BodyEndpointRegister {
        endpoint_id: String,
        descriptor: EndpointCapabilityDescriptor,
    },
    BodyEndpointUnregister {
        route: RouteKey,
    },
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
enum WireMessage {
    Sense {
        sense_id: String,
        source: String,
        payload: serde_json::Value,
    },
    NewCapabilities {
        entries: Vec<EndpointCapabilityDescriptor>,
    },
    DropCapabilities {
        routes: Vec<RouteKey>,
    },
    BodyEndpointRegister {
        endpoint_id: String,
        descriptor: EndpointCapabilityDescriptor,
    },
    BodyEndpointUnregister {
        endpoint_id: String,
        capability_id: String,
    },
}

fn parse_body_ingress_message(line: &str) -> Result<InboundBodyMessage, serde_json::Error> {
    let wire: WireMessage = serde_json::from_str(line)?;
    let message = match wire {
        WireMessage::Sense {
            sense_id,
            source,
            mut payload,
        } => {
            normalize_correlated_sense_payload(&mut payload);
            validate_correlated_sense_payload(&payload)?;
            InboundBodyMessage::Sense(SenseDatum {
                sense_id,
                source,
                payload,
            })
        }
        WireMessage::NewCapabilities { entries } => {
            InboundBodyMessage::NewCapabilities(CapabilityPatch { entries })
        }
        WireMessage::DropCapabilities { routes } => {
            InboundBodyMessage::DropCapabilities(CapabilityDropPatch { routes })
        }
        WireMessage::BodyEndpointRegister {
            endpoint_id,
            descriptor,
        } => InboundBodyMessage::BodyEndpointRegister {
            endpoint_id,
            descriptor,
        },
        WireMessage::BodyEndpointUnregister {
            endpoint_id,
            capability_id,
        } => InboundBodyMessage::BodyEndpointUnregister {
            route: RouteKey {
                endpoint_id,
                capability_id,
            },
        },
    };
    Ok(message)
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
    let encoded = serde_json::to_string(&BodyEgressMessage::Act {
        act: act.clone(),
        action: LegacyAdmittedAction::from(act),
    })?;
    Ok(format!("{encoded}\n"))
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
        registry: Arc<InMemoryEndpointRegistry>,
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
                            let adapter_id = self.adapter_id;
                            tokio::spawn(async move {
                                if let Err(err) =
                                    handle_body_endpoint(stream, ingress, registry, adapter_id)
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
    registry: Arc<InMemoryEndpointRegistry>,
    adapter_id: u64,
) -> Result<()> {
    let channel_id = registry.allocate_adapter_channel_id(adapter_id);
    let (read_half, mut write_half) = stream.into_split();

    let (outbound_tx, mut outbound_rx) = mpsc::unbounded_channel::<Act>();
    registry.attach_adapter_channel(channel_id, outbound_tx);

    let writer_task = tokio::spawn(async move {
        while let Some(act) = outbound_rx.recv().await {
            let encoded = encode_body_egress_act_message(&act)?;
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
                    if endpoint_id.trim().is_empty() {
                        eprintln!("body endpoint register rejected: endpoint_id cannot be empty");
                        continue;
                    }
                    if let Err(err) =
                        registry.register_adapter_route(channel_id, descriptor.clone())
                    {
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
                    if registry
                        .unregister_adapter_route(channel_id, &route)
                        .is_some()
                        && let Err(err) = ingress
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

    let routes = registry.detach_adapter_channel(channel_id);
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
                BodyEgressMessage, InboundBodyMessage, encode_body_egress_act_message,
                parse_body_ingress_message,
            },
            types::EndpointCapabilityDescriptor,
        },
    };

    #[test]
    fn accepts_sense_message() {
        let parsed = parse_body_ingress_message(
            r#"{"type":"sense","sense_id":"s1","source":"sensor","payload":{"v":1}}"#,
        )
        .expect("sense message should parse");
        assert!(matches!(parsed, InboundBodyMessage::Sense(_)));
    }

    #[test]
    fn accepts_correlated_sense_message_with_required_echo_fields() {
        let parsed = parse_body_ingress_message(
            r#"{"type":"sense","sense_id":"s2","source":"body","payload":{"kind":"present_message_result","act_id":"act:1","capability_instance_id":"chat.1","endpoint_id":"macos-app.01","capability_id":"present.message"}}"#,
        )
        .expect("correlated sense should parse");
        assert!(matches!(parsed, InboundBodyMessage::Sense(_)));
    }

    #[test]
    fn accepts_legacy_correlated_sense_message_with_neural_signal_alias() {
        let parsed = parse_body_ingress_message(
            r#"{"type":"sense","sense_id":"s2","source":"body","payload":{"kind":"present_message_result","neural_signal_id":"act:legacy","capability_instance_id":"chat.1","endpoint_id":"macos-app.01","capability_id":"present.message"}}"#,
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
                r#"{"type":"sense","sense_id":"s3","source":"body","payload":{"kind":"present_message_result","act_id":"act:1","endpoint_id":"macos-app.01","capability_id":"present.message"}}"#
            )
            .is_err()
        );
    }

    #[test]
    fn rejects_removed_admission_feedback_message() {
        assert!(
            parse_body_ingress_message(
                r#"{"type":"admission_feedback","attempt_id":"att:1","code":"admitted"}"#
            )
            .is_err()
        );
    }

    #[test]
    fn act_encoding_contains_act_and_target_fields() {
        let act = Act {
            act_id: "act:1".to_string(),
            based_on: vec!["s1".to_string()],
            endpoint_id: "macos-app.01".to_string(),
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
        let message: BodyEgressMessage =
            serde_json::from_str(encoded.trim_end()).expect("message should decode");
        match message {
            BodyEgressMessage::Act { act, action } => {
                assert_eq!(act.act_id, "act:1");
                assert_eq!(action.neural_signal_id, "act:1");
                assert_eq!(action.capability_id, "present.message");
                assert_eq!(action.reserved_cost.survival_micro, 120);
                assert_eq!(action.reserved_cost.time_ms, 100);
                assert_eq!(action.reserved_cost.io_units, 1);
                assert_eq!(action.reserved_cost.token_units, 64);
            }
        }
        assert!(encoded.contains("\"type\":\"act\""));
        assert!(encoded.contains("\"act\":{"));
        assert!(encoded.contains("\"action\":{"));
        assert!(encoded.contains("\"act_id\":\"act:1\""));
        assert!(encoded.contains("\"neural_signal_id\":\"act:1\""));
        assert!(encoded.contains("\"capability_id\":\"present.message\""));
    }

    #[test]
    fn accepts_registration_message() {
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
            "type": "body_endpoint_register",
            "endpoint_id": "session-1",
            "descriptor": descriptor
        });

        let parsed = parse_body_ingress_message(&wire.to_string()).expect("register should parse");
        assert!(matches!(
            parsed,
            InboundBodyMessage::BodyEndpointRegister { .. }
        ));
    }
}
