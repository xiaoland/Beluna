use std::{collections::BTreeMap, sync::Arc};

use opentelemetry_proto::tonic::{
    collector::logs::v1::{
        ExportLogsServiceRequest, ExportLogsServiceResponse,
        logs_service_server::{LogsService, LogsServiceServer},
    },
    common::v1::{AnyValue, KeyValue, any_value::Value as OtlpValue},
};
use serde_json::{Map, Value, json};
use tauri::{AppHandle, Emitter};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::{
    model::IngestPulse,
    state::ReceiverState,
    store::{MoiraStore, NormalizedEvent},
};

pub struct MoiraLogsService {
    receiver: Arc<ReceiverState>,
    store: Arc<MoiraStore>,
    app_handle: AppHandle,
}

impl MoiraLogsService {
    pub fn new(
        receiver: Arc<ReceiverState>,
        store: Arc<MoiraStore>,
        app_handle: AppHandle,
    ) -> Self {
        Self {
            receiver,
            store,
            app_handle,
        }
    }
}

pub fn logs_service(service: MoiraLogsService) -> LogsServiceServer<MoiraLogsService> {
    LogsServiceServer::new(service)
}

#[tonic::async_trait]
impl LogsService for MoiraLogsService {
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
            self.receiver.mark_batch(outcome.last_batch_at.clone()).await;
        }
        let _ = self.app_handle.emit(
            "lachesis-updated",
            IngestPulse {
                touched_run_ids: outcome.touched_run_ids,
                last_batch_at: outcome.last_batch_at,
            },
        );

        Ok(Response::new(ExportLogsServiceResponse {
            partial_success: None,
        }))
    }
}

fn normalize_export(request: ExportLogsServiceRequest, received_at: &str) -> Vec<NormalizedEvent> {
    let mut events = Vec::new();

    for resource_logs in request.resource_logs {
        let resource_json = resource_attributes_json(resource_logs.resource.as_ref());
        for scope_logs in resource_logs.scope_logs {
            let scope_json = scope_json(&scope_logs);
            for log_record in scope_logs.log_records {
                let attributes = attributes_to_map(&log_record.attributes);
                let body = log_record
                    .body
                    .as_ref()
                    .map(any_value_to_json)
                    .unwrap_or(Value::Null);
                let observed_at = format_record_time(
                    log_record.time_unix_nano,
                    log_record.observed_time_unix_nano,
                    received_at,
                );
                let message_text = extract_string(&body)
                    .or_else(|| extract_string_field(&attributes, "message"));
                let target = extract_string_field(&attributes, "target");
                let family = extract_string_field(&attributes, "family");
                let subsystem = extract_string_field(&attributes, "subsystem")
                    .or_else(|| infer_subsystem(target.as_deref(), family.as_deref()));
                let run_id = extract_string_field(&attributes, "run_id");
                let tick = extract_i64_field(&attributes, "tick")
                    .or_else(|| extract_i64_field(&attributes, "cycle_id"));

                events.push(NormalizedEvent {
                    raw_event_id: Uuid::now_v7().to_string(),
                    received_at: received_at.to_string(),
                    observed_at,
                    severity_text: log_record.severity_text,
                    severity_number: log_record.severity_number,
                    target,
                    family,
                    subsystem,
                    run_id,
                    tick,
                    message_text,
                    body_json: body.to_string(),
                    attributes_json: Value::Object(attributes).to_string(),
                    resource_json: resource_json.to_string(),
                    scope_json: scope_json.to_string(),
                });
            }
        }
    }

    events
}

fn attributes_to_map(attributes: &[KeyValue]) -> Map<String, Value> {
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

fn resource_attributes_json(
    resource: Option<&opentelemetry_proto::tonic::resource::v1::Resource>,
) -> Value {
    resource
        .map(|resource| Value::Object(attributes_to_map(&resource.attributes)))
        .unwrap_or_else(|| json!({}))
}

fn scope_json(scope_logs: &opentelemetry_proto::tonic::logs::v1::ScopeLogs) -> Value {
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

fn any_value_to_json(value: &AnyValue) -> Value {
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

fn format_record_time(time_unix_nano: u64, observed_time_unix_nano: u64, fallback: &str) -> String {
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

fn extract_string(value: &Value) -> Option<String> {
    value.as_str().map(str::to_string)
}

fn extract_string_field(attributes: &Map<String, Value>, key: &str) -> Option<String> {
    attributes.get(key).and_then(extract_string)
}

fn extract_i64_field(attributes: &Map<String, Value>, key: &str) -> Option<i64> {
    let value = attributes.get(key)?;
    if let Some(value) = value.as_i64() {
        return Some(value);
    }
    if let Some(value) = value.as_u64() {
        return i64::try_from(value).ok();
    }
    value
        .as_str()
        .and_then(|item| item.parse::<i64>().ok())
}

fn infer_subsystem(target: Option<&str>, family: Option<&str>) -> Option<String> {
    family
        .and_then(|item| item.split('.').next())
        .or_else(|| target.and_then(|item| item.split('.').next()))
        .map(str::to_string)
}
