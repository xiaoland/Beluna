use std::{
    collections::BTreeMap,
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
    io::{AsyncBufReadExt, AsyncWrite, AsyncWriteExt, BufReader},
    net::UnixStream,
};

const PRESENT_PLAIN_TEXT_CAPABILITY_ID: &str = "present.plain_text";

#[derive(Debug, Clone, PartialEq, Eq)]
struct CliOptions {
    socket_path: PathBuf,
    endpoint_id: String,
}

fn cli_options_from_args() -> Result<CliOptions> {
    parse_cli_options(env::args().skip(1))
}

fn parse_cli_options<I>(mut args: I) -> Result<CliOptions>
where
    I: Iterator<Item = String>,
{
    let mut socket_path = None;
    let mut endpoint_id = "body.cli".to_string();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--socket-path" => {
                let value = args
                    .next()
                    .ok_or_else(|| anyhow!("missing value for --socket-path"))?;
                socket_path = Some(PathBuf::from(value));
            }
            "--endpoint-id" => {
                endpoint_id = args
                    .next()
                    .ok_or_else(|| anyhow!("missing value for --endpoint-id"))?;
            }
            other => {
                return Err(anyhow!(
                    "unknown argument: {other}. usage: beluna-cli --socket-path <path> [--endpoint-id <id>]"
                ));
            }
        }
    }

    if endpoint_id.trim().is_empty() {
        return Err(anyhow!("endpoint id cannot be empty"));
    }

    let socket_path = socket_path.ok_or_else(|| {
        anyhow!("missing required argument --socket-path. usage: beluna-cli --socket-path <path> [--endpoint-id <id>]")
    })?;

    Ok(CliOptions {
        socket_path,
        endpoint_id,
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
struct CostVector {
    survival_micro: i64,
    time_ms: u64,
    io_units: u64,
    token_units: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
struct RouteKey {
    endpoint_id: String,
    capability_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct EndpointCapabilityDescriptor {
    route: RouteKey,
    payload_schema: serde_json::Value,
    max_payload_bytes: usize,
    #[serde(default)]
    default_cost: CostVector,
    #[serde(default)]
    metadata: BTreeMap<String, String>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum BodyIngressMessage {
    BodyEndpointRegister {
        endpoint_id: String,
        descriptor: EndpointCapabilityDescriptor,
    },
    Sense {
        sense_id: String,
        source: String,
        payload: serde_json::Value,
    },
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum BodyEgressMessage {
    Act { act: ActEnvelope },
}

#[derive(Debug, Deserialize)]
struct ActEnvelope {
    act_id: String,
    endpoint_id: String,
    capability_id: String,
    normalized_payload: serde_json::Value,
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

    let (read_half, mut write_half) = stream.into_split();
    let mut socket_lines = BufReader::new(read_half).lines();

    let registration = BodyIngressMessage::BodyEndpointRegister {
        endpoint_id: options.endpoint_id.clone(),
        descriptor: plain_text_descriptor(&options.endpoint_id),
    };
    send_message(&mut write_half, &registration).await?;

    eprintln!(
        "beluna-cli connected: socket={} endpoint={} capability={}",
        options.socket_path.display(),
        options.endpoint_id,
        PRESENT_PLAIN_TEXT_CAPABILITY_ID
    );

    let sense_counter = Arc::new(AtomicU64::new(1));
    let stdin_task = tokio::spawn({
        let endpoint_id = options.endpoint_id.clone();
        let counter = Arc::clone(&sense_counter);
        let mut stdin_lines = BufReader::new(tokio::io::stdin()).lines();

        async move {
            while let Some(line) = stdin_lines.next_line().await? {
                let message = line.trim();
                if message.is_empty() {
                    continue;
                }

                let sense = BodyIngressMessage::Sense {
                    sense_id: format!(
                        "sense:cli:{}:{}",
                        now_epoch_millis(),
                        counter.fetch_add(1, Ordering::Relaxed)
                    ),
                    source: endpoint_id.clone(),
                    payload: serde_json::json!({
                        "kind": "user_message",
                        "text": message
                    }),
                };
                send_message(&mut write_half, &sense).await?;
            }

            Ok::<(), anyhow::Error>(())
        }
    });

    while let Some(line) = socket_lines.next_line().await? {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let message: BodyEgressMessage =
            serde_json::from_str(trimmed).context("failed to decode NDJSON act message")?;
        let BodyEgressMessage::Act { act } = message;

        if act.endpoint_id != options.endpoint_id {
            continue;
        }

        if act.capability_id != PRESENT_PLAIN_TEXT_CAPABILITY_ID {
            eprintln!(
                "[warn] unsupported capability '{}' for act {}",
                act.capability_id, act.act_id
            );
            continue;
        }

        let Some(text) = act
            .normalized_payload
            .get("text")
            .and_then(|value| value.as_str())
        else {
            return Err(anyhow!(
                "act {} missing normalized_payload.text string",
                act.act_id
            ));
        };

        println!("{text}");
    }

    stdin_task
        .await
        .context("stdin sender task join failed")??;
    Ok(())
}

async fn send_message<W>(writer: &mut W, message: &BodyIngressMessage) -> Result<()>
where
    W: AsyncWrite + Unpin,
{
    let encoded = serde_json::to_string(message)?;
    writer.write_all(encoded.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await?;
    Ok(())
}

fn plain_text_descriptor(endpoint_id: &str) -> EndpointCapabilityDescriptor {
    EndpointCapabilityDescriptor {
        route: RouteKey {
            endpoint_id: endpoint_id.to_string(),
            capability_id: PRESENT_PLAIN_TEXT_CAPABILITY_ID.to_string(),
        },
        payload_schema: serde_json::json!({
            "type": "object",
            "required": ["text"],
            "properties": {
                "text": { "type": "string" }
            }
        }),
        max_payload_bytes: 64 * 1024,
        default_cost: CostVector::default(),
        metadata: BTreeMap::new(),
    }
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
        assert_eq!(options.endpoint_id, "body.cli");
    }

    #[test]
    fn parses_endpoint_override() {
        let args = vec![
            "--socket-path".to_string(),
            "/tmp/b.sock".to_string(),
            "--endpoint-id".to_string(),
            "body.cli.local".to_string(),
        ]
        .into_iter();
        let options = parse_cli_options(args).expect("options should parse");
        assert_eq!(options.socket_path, PathBuf::from("/tmp/b.sock"));
        assert_eq!(options.endpoint_id, "body.cli.local");
    }

    #[test]
    fn rejects_missing_socket_path() {
        let args = Vec::<String>::new().into_iter();
        assert!(parse_cli_options(args).is_err());
    }
}
