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
        runtime::Spine,
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
            endpoint_id: act.body_endpoint_name.clone(),
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
    Auth {
        endpoint_name: String,
        capabilities: Vec<EndpointCapabilityDescriptor>,
    },
    Sense(SenseDatum),
    NewCapabilities(CapabilityPatch),
    DropCapabilities(CapabilityDropPatch),
    Unplug,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
enum WireMessage {
    Auth {
        endpoint_name: String,
        #[serde(default)]
        capabilities: Vec<EndpointCapabilityDescriptor>,
    },
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
    Unplug,
}

fn parse_body_ingress_message(line: &str) -> Result<InboundBodyMessage, serde_json::Error> {
    let wire: WireMessage = serde_json::from_str(line)?;
    let message = match wire {
        WireMessage::Auth {
            endpoint_name,
            capabilities,
        } => InboundBodyMessage::Auth {
            endpoint_name,
            capabilities,
        },
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
        WireMessage::Unplug => InboundBodyMessage::Unplug,
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
                InboundBodyMessage::NewCapabilities(patch) => {
                    let Some(body_endpoint_id) = auth_endpoint_id else {
                        eprintln!("new_capabilities rejected: endpoint must auth first");
                        continue;
                    };

                    let mut registered_entries = Vec::new();
                    for descriptor in patch.entries {
                        match spine.register_body_endpoint_capability(body_endpoint_id, descriptor)
                        {
                            Ok(descriptor) => registered_entries.push(descriptor),
                            Err(err) => {
                                eprintln!("capability registration failed: {err:#}");
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
                        eprintln!("dropping new_capabilities due to closed ingress: {err}");
                    }
                }
                InboundBodyMessage::DropCapabilities(drop_patch) => {
                    let Some(body_endpoint_id) = auth_endpoint_id else {
                        eprintln!("drop_capabilities rejected: endpoint must auth first");
                        continue;
                    };

                    let mut dropped = Vec::new();
                    for route in drop_patch.routes {
                        if let Some(route) = spine.unregister_body_endpoint_capability(
                            body_endpoint_id,
                            &route.capability_id,
                        ) {
                            dropped.push(route);
                        }
                    }

                    if !dropped.is_empty()
                        && let Err(err) = ingress
                            .send(Sense::DropCapabilities(CapabilityDropPatch {
                                routes: dropped,
                            }))
                            .await
                    {
                        eprintln!("dropping drop_capabilities due to closed ingress: {err}");
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
            "type": "auth",
            "endpoint_name": "macos-app",
            "capabilities": [descriptor]
        });

        let parsed = parse_body_ingress_message(&wire.to_string()).expect("auth should parse");
        assert!(matches!(parsed, InboundBodyMessage::Auth { .. }));
    }
}
