use std::{
    env,
    fs,
    net::SocketAddr,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

use opentelemetry_proto::tonic::collector::logs::v1::{
    ExportLogsServiceRequest, ExportLogsServiceResponse,
    logs_service_server::{LogsService, LogsServiceServer},
};
use serde_json::{Value, json};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use tokio::{net::TcpListener, sync::Notify};
use tokio_stream::wrappers::TcpListenerStream;
use tonic::{Request, Response, Status, transport::Server};

#[derive(Debug, Clone)]
struct CaptureConfig {
    bind: SocketAddr,
    out_dir: PathBuf,
    max_batches: usize,
    timeout_ms: u64,
}

#[derive(Debug)]
struct CaptureState {
    out_dir: PathBuf,
    max_batches: usize,
    count: AtomicUsize,
    done: Notify,
}

#[derive(Debug, Clone)]
struct CaptureService {
    state: Arc<CaptureState>,
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let config = parse_args()?;
    fs::create_dir_all(&config.out_dir)
        .map_err(|err| format!("failed to create output dir: {err}"))?;

    let listener = TcpListener::bind(config.bind)
        .await
        .map_err(|err| format!("failed to bind {}: {err}", config.bind))?;
    let state = Arc::new(CaptureState {
        out_dir: config.out_dir,
        max_batches: config.max_batches,
        count: AtomicUsize::new(0),
        done: Notify::new(),
    });
    let service = CaptureService {
        state: state.clone(),
    };

    println!("listening {}", config.bind);

    let server = Server::builder()
        .add_service(LogsServiceServer::new(service))
        .serve_with_incoming_shutdown(TcpListenerStream::new(listener), async move {
            tokio::select! {
                _ = state.done.notified() => {}
                _ = tokio::time::sleep(Duration::from_millis(config.timeout_ms)) => {}
                _ = tokio::signal::ctrl_c() => {}
            }
        });

    server
        .await
        .map_err(|err| format!("capture server failed: {err}"))?;

    Ok(())
}

#[tonic::async_trait]
impl LogsService for CaptureService {
    async fn export(
        &self,
        request: Request<ExportLogsServiceRequest>,
    ) -> Result<Response<ExportLogsServiceResponse>, Status> {
        let batch_index = self.state.count.fetch_add(1, Ordering::SeqCst) + 1;
        let received_at = timestamp_now();
        let request = request.into_inner();
        let raw_path = self
            .state
            .out_dir
            .join(format!("otlp-export-batch-{batch_index:03}.json"));
        let summary_path = self
            .state
            .out_dir
            .join(format!("otlp-export-batch-{batch_index:03}-summary.json"));

        write_json(&raw_path, &request).map_err(Status::internal)?;
        write_json(&summary_path, &summarize_request(&request, &received_at))
            .map_err(Status::internal)?;

        if batch_index >= self.state.max_batches {
            self.state.done.notify_waiters();
        }

        Ok(Response::new(ExportLogsServiceResponse {
            partial_success: None,
        }))
    }
}

fn summarize_request(request: &ExportLogsServiceRequest, received_at: &str) -> Value {
    let mut log_records = Vec::new();

    for resource_logs in &request.resource_logs {
        let resource = resource_logs
            .resource
            .as_ref()
            .map(|resource| attributes_json(&resource.attributes))
            .unwrap_or_else(|| json!({}));
        for scope_logs in &resource_logs.scope_logs {
            let scope = scope_logs
                .scope
                .as_ref()
                .map(|scope| {
                    json!({
                        "name": scope.name,
                        "version": scope.version,
                        "attributes": attributes_json(&scope.attributes),
                    })
                })
                .unwrap_or_else(|| json!({}));
            for log_record in &scope_logs.log_records {
                log_records.push(json!({
                    "received_at": received_at,
                    "resource": resource,
                    "scope": scope,
                    "time_unix_nano": log_record.time_unix_nano.to_string(),
                    "observed_time_unix_nano": log_record.observed_time_unix_nano.to_string(),
                    "severity_text": log_record.severity_text,
                    "severity_number": log_record.severity_number,
                    "event_name": log_record.event_name,
                    "body": log_record.body,
                    "attributes": attributes_json(&log_record.attributes),
                    "trace_id_hex": hex_string(&log_record.trace_id),
                    "span_id_hex": hex_string(&log_record.span_id),
                    "flags": log_record.flags,
                }));
            }
        }
    }

    json!({
        "received_at": received_at,
        "resource_logs_count": request.resource_logs.len(),
        "log_records_count": log_records.len(),
        "log_records": log_records,
    })
}

fn attributes_json(attributes: &[opentelemetry_proto::tonic::common::v1::KeyValue]) -> Value {
    let mut object = serde_json::Map::new();
    for attr in attributes {
        object.insert(attr.key.clone(), json!(attr.value));
    }
    Value::Object(object)
}

fn write_json(path: &PathBuf, value: &impl serde::Serialize) -> Result<(), String> {
    let rendered = serde_json::to_string_pretty(value)
        .map_err(|err| format!("failed to render {}: {err}", path.display()))?;
    fs::write(path, format!("{rendered}\n"))
        .map_err(|err| format!("failed to write {}: {err}", path.display()))
}

fn timestamp_now() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}

fn hex_string(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn parse_args() -> Result<CaptureConfig, String> {
    let mut bind = "127.0.0.1:54317"
        .parse::<SocketAddr>()
        .expect("default bind is valid");
    let mut out_dir = PathBuf::from("tasks/o11y-otel-log-model-reset-20260504/fixtures/current");
    let mut max_batches = 1_usize;
    let mut timeout_ms = 15_000_u64;

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--bind" => {
                let value = args.next().ok_or("missing value for --bind")?;
                bind = value
                    .parse::<SocketAddr>()
                    .map_err(|err| format!("invalid --bind `{value}`: {err}"))?;
            }
            "--out-dir" => {
                out_dir = PathBuf::from(args.next().ok_or("missing value for --out-dir")?);
            }
            "--max-batches" => {
                let value = args.next().ok_or("missing value for --max-batches")?;
                max_batches = value
                    .parse::<usize>()
                    .map_err(|err| format!("invalid --max-batches `{value}`: {err}"))?
                    .max(1);
            }
            "--timeout-ms" => {
                let value = args.next().ok_or("missing value for --timeout-ms")?;
                timeout_ms = value
                    .parse::<u64>()
                    .map_err(|err| format!("invalid --timeout-ms `{value}`: {err}"))?
                    .max(1);
            }
            other => return Err(format!("unknown argument `{other}`")),
        }
    }

    Ok(CaptureConfig {
        bind,
        out_dir,
        max_batches,
        timeout_ms,
    })
}
