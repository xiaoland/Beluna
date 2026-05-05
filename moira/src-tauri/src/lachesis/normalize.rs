use opentelemetry_proto::tonic::collector::logs::v1::ExportLogsServiceRequest;
use serde_json::Value;

use crate::lachesis::{
    receiver::{
        extract_i64_field, extract_string, extract_string_field, format_record_time,
        infer_subsystem, random_raw_event_id, resource_attributes_json, scope_json,
    },
    store::NormalizedEvent,
};

pub(crate) fn normalize_export(
    request: ExportLogsServiceRequest,
    received_at: &str,
) -> Vec<NormalizedEvent> {
    let mut events = Vec::new();

    for resource_logs in request.resource_logs {
        let resource_json = resource_attributes_json(resource_logs.resource.as_ref());
        for scope_logs in resource_logs.scope_logs {
            let scope_name = scope_logs
                .scope
                .as_ref()
                .map(|scope| scope.name.clone())
                .filter(|value| !value.is_empty());
            let scope_json = scope_json(&scope_logs);
            for log_record in scope_logs.log_records {
                let attributes =
                    crate::lachesis::receiver::attributes_to_map(&log_record.attributes);
                let body = log_record
                    .body
                    .as_ref()
                    .map(crate::lachesis::receiver::any_value_to_json)
                    .unwrap_or(Value::Null);
                let observed_at = format_record_time(
                    log_record.time_unix_nano,
                    log_record.observed_time_unix_nano,
                    received_at,
                );
                let message_text =
                    extract_string(&body).or_else(|| extract_string_field(&attributes, "message"));
                let target = extract_string_field(&attributes, "target");
                let family = extract_string_field(&attributes, "family");
                let event_name =
                    Some(log_record.event_name.clone()).filter(|value| !value.is_empty());
                let trace_id_hex = bytes_to_hex(&log_record.trace_id);
                let span_id_hex = bytes_to_hex(&log_record.span_id);
                let trace_flags = (log_record.flags > 0).then_some(log_record.flags);
                let record_kind = classify_record(
                    scope_name.as_deref(),
                    event_name.as_deref(),
                    family.as_deref(),
                    attributes.get("payload"),
                );
                let subsystem = extract_string_field(&attributes, "subsystem").or_else(|| {
                    infer_subsystem(target.as_deref(), family.as_deref(), scope_name.as_deref())
                });
                let run_id = extract_string_field(&attributes, "run_id")
                    .or_else(|| extract_string_field_from_value(&body, "run_id"));
                let tick = extract_i64_field(&attributes, "tick")
                    .or_else(|| extract_i64_field(&attributes, "cycle_id"))
                    .or_else(|| extract_i64_field_from_value(&body, "tick"))
                    .or_else(|| extract_i64_field_from_value(&body, "cycle_id"));

                events.push(NormalizedEvent {
                    raw_event_id: random_raw_event_id(),
                    received_at: received_at.to_string(),
                    observed_at,
                    severity_text: log_record.severity_text,
                    severity_number: log_record.severity_number,
                    record_kind: record_kind.to_string(),
                    scope_name: scope_name.clone(),
                    event_name,
                    trace_id_hex,
                    span_id_hex,
                    trace_flags,
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

fn classify_record(
    scope_name: Option<&str>,
    event_name: Option<&str>,
    family: Option<&str>,
    payload: Option<&Value>,
) -> &'static str {
    if scope_name == Some("observability.contract") || (family.is_some() && payload.is_some()) {
        return "legacy_contract";
    }

    if scope_name
        .map(|scope| scope.starts_with("beluna.core."))
        .unwrap_or(false)
        && event_name.is_some()
    {
        return "native_owner";
    }

    "ordinary_log"
}

fn bytes_to_hex(bytes: &[u8]) -> Option<String> {
    if bytes.is_empty() {
        return None;
    }

    Some(bytes.iter().map(|byte| format!("{byte:02x}")).collect())
}

fn extract_string_field_from_value(value: &Value, key: &str) -> Option<String> {
    value.get(key).and_then(extract_string)
}

fn extract_i64_field_from_value(value: &Value, key: &str) -> Option<i64> {
    let value = value.get(key)?;
    if let Some(value) = value.as_i64() {
        return Some(value);
    }
    if let Some(value) = value.as_u64() {
        return i64::try_from(value).ok();
    }
    value.as_str().and_then(|item| item.parse::<i64>().ok())
}
