use std::{
    env,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::UnixStream,
    sync::{Mutex, mpsc},
};

const PRESENT_PLAIN_TEXT_NEURAL_SIGNAL_DESCRIPTOR_ID: &str = "present.plain_text";
const USER_MESSAGE_NEURAL_SIGNAL_DESCRIPTOR_ID: &str = "user.message";

#[derive(Debug, Clone, PartialEq, Eq)]
struct CliOptions {
    socket_path: PathBuf,
    endpoint_name: String,
}

fn cli_options_from_args() -> Result<CliOptions> {
    parse_cli_options(env::args().skip(1))
}

fn parse_cli_options<I>(mut args: I) -> Result<CliOptions>
where
    I: Iterator<Item = String>,
{
    let mut socket_path = None;
    let mut endpoint_name = "body.cli".to_string();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--socket-path" => {
                let value = args
                    .next()
                    .ok_or_else(|| anyhow!("missing value for --socket-path"))?;
                socket_path = Some(PathBuf::from(value));
            }
            "--endpoint-name" | "--endpoint-id" => {
                endpoint_name = args
                    .next()
                    .ok_or_else(|| anyhow!("missing value for {arg}"))?;
            }
            other => {
                return Err(anyhow!(
                    "unknown argument: {other}. usage: beluna-cli --socket-path <path> [--endpoint-name <name>]"
                ));
            }
        }
    }

    if endpoint_name.trim().is_empty() {
        return Err(anyhow!("endpoint name cannot be empty"));
    }

    let socket_path = socket_path.ok_or_else(|| {
        anyhow!("missing required argument --socket-path. usage: beluna-cli --socket-path <path> [--endpoint-name <name>]")
    })?;

    Ok(CliOptions {
        socket_path,
        endpoint_name,
    })
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum NeuralSignalType {
    Sense,
    Act,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
struct NeuralSignalDescriptor {
    #[serde(rename = "type")]
    r#type: NeuralSignalType,
    endpoint_id: String,
    neural_signal_descriptor_id: String,
    payload_schema: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
struct NdjsonEnvelope<T>
where
    T: Serialize,
{
    method: String,
    id: String,
    timestamp: u64,
    body: T,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
struct AuthBody {
    endpoint_name: String,
    capabilities: Vec<NeuralSignalDescriptor>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
struct SenseBody {
    sense_id: String,
    neural_signal_descriptor_id: String,
    payload: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
struct ActAckBody {
    act_id: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
struct InboundEnvelope {
    method: String,
    id: String,
    timestamp: u64,
    body: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
struct InboundActBody {
    act: Act,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
struct Act {
    act_id: String,
    endpoint_id: String,
    neural_signal_descriptor_id: String,
    payload: serde_json::Value,
}

#[tokio::main]
async fn main() -> Result<()> {
    let options = cli_options_from_args()?;
    let stream = UnixStream::connect(&options.socket_path)
        .await
        .with_context(|| {
            format!(
                "failed to connect to Beluna socket {}",
                options.socket_path.display()
            )
        })?;

    let (read_half, write_half) = stream.into_split();
    let writer = Arc::new(Mutex::new(write_half));
    let mut socket_lines = BufReader::new(read_half).lines();

    send_ndjson(
        Arc::clone(&writer),
        "auth",
        AuthBody {
            endpoint_name: options.endpoint_name.clone(),
            capabilities: vec![
                plain_text_descriptor(&options.endpoint_name),
                user_message_descriptor(&options.endpoint_name),
            ],
        },
    )
    .await?;

    eprintln!(
        "beluna-cli connected: socket={} endpoint_name={} descriptor_id={}",
        options.socket_path.display(),
        options.endpoint_name,
        PRESENT_PLAIN_TEXT_NEURAL_SIGNAL_DESCRIPTOR_ID
    );

    let (shutdown_tx, mut shutdown_rx) = mpsc::unbounded_channel::<()>();
    let sense_counter = Arc::new(AtomicU64::new(1));
    let stdin_task = tokio::spawn({
        let writer = Arc::clone(&writer);
        let counter = Arc::clone(&sense_counter);
        let mut stdin_lines = BufReader::new(tokio::io::stdin()).lines();

        async move {
            loop {
                tokio::select! {
                    _ = shutdown_rx.recv() => {
                        break;
                    }
                    maybe_line = stdin_lines.next_line() => {
                        let Some(line) = maybe_line? else {
                            break;
                        };
                        let message = line.trim();
                        if message.is_empty() {
                            continue;
                        }

                        send_ndjson(
                            Arc::clone(&writer),
                            "sense",
                            SenseBody {
                                sense_id: uuid::Uuid::new_v4().to_string(),
                                neural_signal_descriptor_id: USER_MESSAGE_NEURAL_SIGNAL_DESCRIPTOR_ID.to_string(),
                                payload: serde_json::json!({
                                    "kind": "user_message",
                                    "text": message,
                                    "seq_no": counter.fetch_add(1, Ordering::Relaxed),
                                    "timestamp_ms": now_epoch_millis(),
                                }),
                            },
                        )
                        .await?;
                    }
                }
            }

            Ok::<(), anyhow::Error>(())
        }
    });

    while let Some(line) = socket_lines.next_line().await? {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let envelope: InboundEnvelope =
            serde_json::from_str(trimmed).context("failed to decode NDJSON envelope")?;
        if envelope.method != "act" {
            continue;
        }

        let act_body: InboundActBody = serde_json::from_value(envelope.body)
            .context("failed to decode NDJSON act body payload")?;
        let act = act_body.act;

        send_ndjson(
            Arc::clone(&writer),
            "act_ack",
            ActAckBody {
                act_id: act.act_id.clone(),
            },
        )
        .await?;

        if act.neural_signal_descriptor_id != PRESENT_PLAIN_TEXT_NEURAL_SIGNAL_DESCRIPTOR_ID {
            eprintln!(
                "[warn] unsupported neural_signal_descriptor_id '{}' for act {}",
                act.neural_signal_descriptor_id, act.act_id
            );
            continue;
        }

        let Some(text) = extract_text_from_payload(&act.payload) else {
            return Err(anyhow!("act {} missing payload text string", act.act_id));
        };

        println!("{text}");
    }

    let _ = shutdown_tx.send(());
    stdin_task
        .await
        .context("stdin sender task join failed")??;
    Ok(())
}

async fn send_ndjson<T>(
    writer: Arc<Mutex<tokio::net::unix::OwnedWriteHalf>>,
    method: &str,
    body: T,
) -> Result<()>
where
    T: Serialize,
{
    let envelope = NdjsonEnvelope {
        method: method.to_string(),
        id: uuid::Uuid::new_v4().to_string(),
        timestamp: now_epoch_millis() as u64,
        body,
    };
    let encoded = serde_json::to_string(&envelope)?;

    let mut writer = writer.lock().await;
    writer.write_all(encoded.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await?;
    Ok(())
}

fn plain_text_descriptor(endpoint_name: &str) -> NeuralSignalDescriptor {
    NeuralSignalDescriptor {
        r#type: NeuralSignalType::Act,
        endpoint_id: endpoint_name.to_string(),
        neural_signal_descriptor_id: PRESENT_PLAIN_TEXT_NEURAL_SIGNAL_DESCRIPTOR_ID.to_string(),
        payload_schema: serde_json::json!({
            "type": "object",
            "required": ["text"],
            "properties": {
                "text": { "type": "string" }
            }
        }),
    }
}

fn user_message_descriptor(endpoint_name: &str) -> NeuralSignalDescriptor {
    NeuralSignalDescriptor {
        r#type: NeuralSignalType::Sense,
        endpoint_id: endpoint_name.to_string(),
        neural_signal_descriptor_id: USER_MESSAGE_NEURAL_SIGNAL_DESCRIPTOR_ID.to_string(),
        payload_schema: serde_json::json!({
            "type": "object",
            "required": ["kind", "text"],
            "properties": {
                "kind": { "type": "string" },
                "text": { "type": "string" },
                "seq_no": { "type": "integer" },
                "timestamp_ms": { "type": "integer" }
            }
        }),
    }
}

fn extract_text_from_payload(payload: &serde_json::Value) -> Option<&str> {
    if let Some(text) = payload.get("text").and_then(|value| value.as_str()) {
        return Some(text);
    }
    if let Some(text) = payload.get("output_text").and_then(|value| value.as_str()) {
        return Some(text);
    }
    payload
        .get("response")
        .and_then(|response| response.get("output_text"))
        .and_then(|value| value.as_str())
}

fn now_epoch_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::parse_cli_options;
    use std::path::PathBuf;

    #[test]
    fn parses_required_socket_path() {
        let args = vec!["--socket-path".to_string(), "./beluna.sock".to_string()].into_iter();
        let options = parse_cli_options(args).expect("options should parse");
        assert_eq!(options.socket_path, PathBuf::from("./beluna.sock"));
        assert_eq!(options.endpoint_name, "body.cli");
    }

    #[test]
    fn parses_endpoint_name_override() {
        let args = vec![
            "--socket-path".to_string(),
            "/tmp/b.sock".to_string(),
            "--endpoint-name".to_string(),
            "body.cli.local".to_string(),
        ]
        .into_iter();
        let options = parse_cli_options(args).expect("options should parse");
        assert_eq!(options.socket_path, PathBuf::from("/tmp/b.sock"));
        assert_eq!(options.endpoint_name, "body.cli.local");
    }

    #[test]
    fn parses_legacy_endpoint_id_flag_for_backward_compatibility() {
        let args = vec![
            "--socket-path".to_string(),
            "/tmp/b.sock".to_string(),
            "--endpoint-id".to_string(),
            "body.cli.legacy".to_string(),
        ]
        .into_iter();
        let options = parse_cli_options(args).expect("options should parse");
        assert_eq!(options.socket_path, PathBuf::from("/tmp/b.sock"));
        assert_eq!(options.endpoint_name, "body.cli.legacy");
    }

    #[test]
    fn rejects_missing_socket_path() {
        let args = Vec::<String>::new().into_iter();
        assert!(parse_cli_options(args).is_err());
    }
}
