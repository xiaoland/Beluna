mod common;

use moira_runtime::lachesis::model::TickDetail;
use opentelemetry_proto::tonic::{
    collector::logs::v1::{ExportLogsServiceRequest, logs_service_client::LogsServiceClient},
    common::v1::{
        AnyValue, InstrumentationScope, KeyValue, KeyValueList, any_value::Value as AnyValueKind,
    },
    logs::v1::{LogRecord, ResourceLogs, ScopeLogs, SeverityNumber},
    resource::v1::Resource,
};

use crate::common::{RuntimeSandbox, wait_for_receiver_ready, wait_for_runtime_status};

#[tokio::test]
async fn runtime_ingests_otlp_logs_and_projects_run_tick_detail() {
    let sandbox = RuntimeSandbox::new();
    let runtime = sandbox.open_runtime().await;
    wait_for_receiver_ready(runtime.as_ref()).await;

    let mut client = LogsServiceClient::connect(format!("http://{}", sandbox.receiver_bind()))
        .await
        .expect("OTLP client should connect");
    client
        .export(native_tick_request())
        .await
        .expect("OTLP export should succeed");

    let status = wait_for_runtime_status(runtime.as_ref(), |status| {
        status.receiver.raw_event_count >= 2 && status.receiver.tick_count >= 1
    })
    .await;
    assert_eq!(status.receiver.raw_event_count, 2);
    assert_eq!(status.receiver.wake_count, 1);
    assert_eq!(status.receiver.tick_count, 1);

    let runs = runtime
        .lachesis()
        .list_runs()
        .await
        .expect("runs should query");
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].run_id, "run-it");
    assert_eq!(runs[0].event_count, 2);
    assert_eq!(runs[0].latest_tick, Some(7));

    let ticks = runtime
        .lachesis()
        .list_ticks("run-it")
        .await
        .expect("ticks should query");
    assert_eq!(ticks.len(), 1);
    assert_eq!(ticks[0].tick, 7);
    assert_eq!(ticks[0].event_count, 2);
    assert!(ticks[0].cortex_handled);

    let detail = runtime
        .lachesis()
        .tick_detail("run-it", 7)
        .await
        .expect("tick detail should query");
    assert_tick_detail(detail);
}

fn native_tick_request() -> ExportLogsServiceRequest {
    let trace_id = (1_u8..=16).collect::<Vec<_>>();

    ExportLogsServiceRequest {
        resource_logs: vec![ResourceLogs {
            resource: Some(Resource {
                attributes: vec![kv_string("service.name", "beluna.core")],
                dropped_attributes_count: 0,
                entity_refs: Vec::new(),
            }),
            scope_logs: vec![
                ScopeLogs {
                    scope: Some(scope("beluna.core.stem.tick")),
                    log_records: vec![LogRecord {
                        time_unix_nano: 1_767_225_600_000_000_000,
                        observed_time_unix_nano: 1_767_225_600_000_000_000,
                        severity_number: SeverityNumber::Info as i32,
                        severity_text: "INFO".to_string(),
                        body: Some(kv_body(vec![
                            kv_string("summary", "Stem granted tick 7."),
                            kv_string("run_id", "run-it"),
                            kv_i64("tick", 7),
                        ])),
                        attributes: vec![kv_string("run_id", "run-it"), kv_i64("tick", 7)],
                        dropped_attributes_count: 0,
                        flags: 1,
                        trace_id: trace_id.clone(),
                        span_id: vec![1, 2, 3, 4, 5, 6, 7, 8],
                        event_name: "granted".to_string(),
                    }],
                    schema_url: String::new(),
                },
                ScopeLogs {
                    scope: Some(scope("beluna.core.cortex.primary")),
                    log_records: vec![LogRecord {
                        time_unix_nano: 1_767_225_601_000_000_000,
                        observed_time_unix_nano: 1_767_225_601_000_000_000,
                        severity_number: SeverityNumber::Info as i32,
                        severity_text: "INFO".to_string(),
                        body: Some(kv_body(vec![kv_string(
                            "summary",
                            "Cortex primary phase started.",
                        )])),
                        attributes: Vec::new(),
                        dropped_attributes_count: 0,
                        flags: 1,
                        trace_id,
                        span_id: vec![8, 7, 6, 5, 4, 3, 2, 1],
                        event_name: "started".to_string(),
                    }],
                    schema_url: String::new(),
                },
            ],
            schema_url: String::new(),
        }],
    }
}

fn assert_tick_detail(detail: TickDetail) {
    assert_eq!(detail.summary.run_id, "run-it");
    assert_eq!(detail.summary.tick, 7);
    assert_eq!(detail.raw.len(), 2);
    assert_eq!(detail.stem.len(), 1);
    assert_eq!(detail.cortex.len(), 1);
    assert_eq!(
        detail.raw[0].scope_name.as_deref(),
        Some("beluna.core.stem.tick")
    );
    assert_eq!(
        detail.raw[1].scope_name.as_deref(),
        Some("beluna.core.cortex.primary")
    );
}

fn scope(name: &str) -> InstrumentationScope {
    InstrumentationScope {
        name: name.to_string(),
        version: String::new(),
        attributes: Vec::new(),
        dropped_attributes_count: 0,
    }
}

fn kv_body(values: Vec<KeyValue>) -> AnyValue {
    AnyValue {
        value: Some(AnyValueKind::KvlistValue(KeyValueList { values })),
    }
}

fn kv_string(key: &str, value: &str) -> KeyValue {
    KeyValue {
        key: key.to_string(),
        value: Some(AnyValue {
            value: Some(AnyValueKind::StringValue(value.to_string())),
        }),
    }
}

fn kv_i64(key: &str, value: i64) -> KeyValue {
    KeyValue {
        key: key.to_string(),
        value: Some(AnyValue {
            value: Some(AnyValueKind::IntValue(value)),
        }),
    }
}
