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
                let subsystem = extract_string_field(&attributes, "subsystem")
                    .or_else(|| infer_subsystem(target.as_deref(), family.as_deref()));
                let run_id = extract_string_field(&attributes, "run_id");
                let tick = extract_i64_field(&attributes, "tick")
                    .or_else(|| extract_i64_field(&attributes, "cycle_id"));

                events.push(NormalizedEvent {
                    raw_event_id: random_raw_event_id(),
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
