use std::{collections::BTreeMap, net::SocketAddr, sync::Arc};

use opentelemetry_proto::tonic::{
    collector::logs::v1::{
        ExportLogsServiceRequest, ExportLogsServiceResponse,
        logs_service_server::{LogsService, LogsServiceServer},
    },
    common::v1::{AnyValue, KeyValue, any_value::Value as OtlpValue},
};
use serde_json::{Map, Value, json};
use tauri::AppHandle;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::{Request, Response, Status, transport::Server};
use uuid::Uuid;

use crate::lachesis::{
    model::ReceiverStatus,
    normalize::normalize_export,
    pulse::{emit_lachesis_updated, empty_ingest_pulse},
    store::LachesisStore,
};

#[derive(Debug, Clone)]
pub struct ReceiverSnapshot {
    pub receiver_state: String,
    pub last_batch_at: Option<String>,
    pub last_error: Option<String>,
}

impl ReceiverSnapshot {
    fn new() -> Self {
        Self {
            receiver_state: "awakening".to_string(),
            last_batch_at: None,
            last_error: None,
        }
    }
}

pub struct ReceiverState {
    endpoint: String,
    inner: RwLock<ReceiverSnapshot>,
}

impl ReceiverState {
    pub fn new(endpoint: String) -> Arc<Self> {
        Arc::new(Self {
            endpoint,
            inner: RwLock::new(ReceiverSnapshot::new()),
        })
    }

    pub async fn mark_faulted(&self, error: impl Into<String>) {
        let mut guard = self.inner.write().await;
        guard.receiver_state = "faulted".to_string();
        guard.last_error = Some(error.into());
    }

    pub async fn mark_listening(&self) {
        let mut guard = self.inner.write().await;
        guard.receiver_state = "listening".to_string();
        guard.last_error = None;
    }

    pub async fn mark_batch(&self, last_batch_at: String) {
        let mut guard = self.inner.write().await;
        guard.last_batch_at = Some(last_batch_at);
        if guard.receiver_state == "awakening" || guard.receiver_state == "listening" {
            guard.receiver_state = "awake".to_string();
        }
    }

    pub async fn snapshot(&self, store: &LachesisStore) -> Result<ReceiverStatus, String> {
        let guard = self.inner.read().await.clone();
        let counts = store.counts().await?;

        Ok(ReceiverStatus {
            endpoint: self.endpoint.clone(),
            wake_state: guard.receiver_state,
            db_path: store.db_path(),
            last_batch_at: guard.last_batch_at,
            last_error: guard.last_error,
            raw_event_count: counts.raw_event_count,
            wake_count: counts.run_count,
            tick_count: counts.tick_count,
        })
    }
}

pub async fn start_otlp_logs_receiver(
    endpoint: SocketAddr,
    receiver: Arc<ReceiverState>,
    store: Arc<LachesisStore>,
    app_handle: AppHandle,
) {
    let service = LachesisLogsService::new(receiver.clone(), store, app_handle.clone());
    let listener = match TcpListener::bind(endpoint).await {
        Ok(listener) => {
            receiver.mark_listening().await;
            listener
        }
        Err(err) => {
            let message = format!("OTLP logs receiver failed to bind: {err}");
            receiver.mark_faulted(message).await;
            emit_lachesis_updated(&app_handle, empty_ingest_pulse());
            return;
        }
    };

    match Server::builder()
        .add_service(logs_service(service))
        .serve_with_incoming(TcpListenerStream::new(listener))
        .await
    {
        Ok(()) => {}
        Err(err) => {
            let message = format!("OTLP logs receiver faulted: {err}");
            receiver.mark_faulted(message).await;
            emit_lachesis_updated(&app_handle, empty_ingest_pulse());
        }
    }
}

struct LachesisLogsService {
    receiver: Arc<ReceiverState>,
    store: Arc<LachesisStore>,
    app_handle: AppHandle,
}

impl LachesisLogsService {
    fn new(receiver: Arc<ReceiverState>, store: Arc<LachesisStore>, app_handle: AppHandle) -> Self {
        Self {
            receiver,
            store,
            app_handle,
        }
    }
}

fn logs_service(service: LachesisLogsService) -> LogsServiceServer<LachesisLogsService> {
    LogsServiceServer::new(service)
}

#[tonic::async_trait]
impl LogsService for LachesisLogsService {
    async fn export(
        &self,
        request: Request<ExportLogsServiceRequest>,
    ) -> Result<Response<ExportLogsServiceResponse>, Status> {
        let received_at = timestamp_now();
        let events = normalize_export(request.into_inner(), &received_at);
        let outcome = self
            .store
            .ingest_events(events)
            .await
            .map_err(Status::internal)?;

        if !outcome.last_batch_at.is_empty() {
            self.receiver
                .mark_batch(outcome.last_batch_at.clone())
                .await;
        }

        emit_lachesis_updated(
            &self.app_handle,
            crate::lachesis::model::IngestPulse {
                touched_run_ids: outcome.touched_run_ids,
                last_batch_at: outcome.last_batch_at,
            },
        );

        Ok(Response::new(ExportLogsServiceResponse {
            partial_success: None,
        }))
    }
}

pub(crate) fn attributes_to_map(attributes: &[KeyValue]) -> Map<String, Value> {
    let mut map = Map::new();
    for attribute in attributes {
        let value = attribute
            .value
            .as_ref()
            .map(any_value_to_json)
            .unwrap_or(Value::Null);
        map.insert(attribute.key.clone(), value);
    }
    map
}

pub(crate) fn resource_attributes_json(
    resource: Option<&opentelemetry_proto::tonic::resource::v1::Resource>,
) -> Value {
    resource
        .map(|resource| Value::Object(attributes_to_map(&resource.attributes)))
        .unwrap_or_else(|| json!({}))
}

pub(crate) fn scope_json(scope_logs: &opentelemetry_proto::tonic::logs::v1::ScopeLogs) -> Value {
    let mut value = BTreeMap::new();
    if let Some(scope) = &scope_logs.scope {
        value.insert("name".to_string(), json!(scope.name));
        value.insert("version".to_string(), json!(scope.version));
        value.insert(
            "attributes".to_string(),
            Value::Object(attributes_to_map(&scope.attributes)),
        );
    }
    Value::Object(value.into_iter().collect())
}

pub(crate) fn any_value_to_json(value: &AnyValue) -> Value {
    match &value.value {
        Some(OtlpValue::StringValue(item)) => json!(item),
        Some(OtlpValue::BoolValue(item)) => json!(item),
        Some(OtlpValue::IntValue(item)) => json!(item),
        Some(OtlpValue::DoubleValue(item)) => json!(item),
        Some(OtlpValue::BytesValue(item)) => json!(item),
        Some(OtlpValue::ArrayValue(item)) => {
            Value::Array(item.values.iter().map(any_value_to_json).collect())
        }
        Some(OtlpValue::KvlistValue(item)) => Value::Object(attributes_to_map(&item.values)),
        None => Value::Null,
    }
}

pub(crate) fn format_record_time(
    time_unix_nano: u64,
    observed_time_unix_nano: u64,
    fallback: &str,
) -> String {
    let candidate = if time_unix_nano > 0 {
        time_unix_nano
    } else {
        observed_time_unix_nano
    };
    if candidate == 0 {
        return fallback.to_string();
    }
    OffsetDateTime::from_unix_timestamp_nanos(candidate as i128)
        .ok()
        .and_then(|value| value.format(&Rfc3339).ok())
        .unwrap_or_else(|| fallback.to_string())
}

fn timestamp_now() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}

pub(crate) fn extract_string(value: &Value) -> Option<String> {
    value.as_str().map(str::to_string)
}

pub(crate) fn extract_string_field(attributes: &Map<String, Value>, key: &str) -> Option<String> {
    attributes.get(key).and_then(extract_string)
}

pub(crate) fn extract_i64_field(attributes: &Map<String, Value>, key: &str) -> Option<i64> {
    let value = attributes.get(key)?;
    if let Some(value) = value.as_i64() {
        return Some(value);
    }
    if let Some(value) = value.as_u64() {
        return i64::try_from(value).ok();
    }
    value.as_str().and_then(|item| item.parse::<i64>().ok())
}

pub(crate) fn infer_subsystem(target: Option<&str>, family: Option<&str>) -> Option<String> {
    family
        .and_then(|item| item.split('.').next())
        .or_else(|| target.and_then(|item| item.split('.').next()))
        .map(str::to_string)
}

pub(crate) fn random_raw_event_id() -> String {
    Uuid::now_v7().to_string()
}
