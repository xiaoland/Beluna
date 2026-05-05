use std::sync::OnceLock;

use opentelemetry::{
    InstrumentationScope, Key, TraceFlags,
    logs::{AnyValue, LogRecord as _, Logger as _, LoggerProvider as _, Severity},
};
use opentelemetry_sdk::logs::SdkLoggerProvider;
use serde_json::Value;

use crate::observability::runtime::current_run_id;

use super::{OwnerScope, ids, value::json_to_any};

static LOGGER_PROVIDER: OnceLock<SdkLoggerProvider> = OnceLock::new();

pub(crate) struct OwnerLogEvent {
    pub(crate) scope: OwnerScope,
    pub(crate) event_name: &'static str,
    pub(crate) tick: u64,
    pub(crate) span_key: String,
    pub(crate) severity: OwnerLogSeverity,
    pub(crate) attributes: Vec<OwnerLogAttribute>,
    pub(crate) body: Value,
}

pub(crate) struct OwnerLogAttribute {
    key: &'static str,
    value: OwnerLogAttributeValue,
}

impl OwnerLogAttribute {
    pub(crate) fn string(key: &'static str, value: impl Into<String>) -> Self {
        Self {
            key,
            value: OwnerLogAttributeValue::String(value.into()),
        }
    }
}

enum OwnerLogAttributeValue {
    String(String),
}

impl OwnerLogAttributeValue {
    fn into_any(self) -> AnyValue {
        match self {
            OwnerLogAttributeValue::String(value) => AnyValue::String(value.into()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OwnerLogSeverity {
    Info,
    Error,
}

impl OwnerLogSeverity {
    fn as_otel(self) -> Severity {
        match self {
            OwnerLogSeverity::Info => Severity::Info,
            OwnerLogSeverity::Error => Severity::Error,
        }
    }

    fn as_str(self) -> &'static str {
        self.as_otel().name()
    }
}

pub(crate) fn install_logger_provider(provider: SdkLoggerProvider) -> Result<(), String> {
    LOGGER_PROVIDER
        .set(provider)
        .map_err(|_| "owner log provider already installed".to_string())
}

pub(crate) fn emit(event: OwnerLogEvent) {
    if let Some(provider) = LOGGER_PROVIDER.get() {
        emit_with_provider(provider, event);
    }
}

fn emit_with_provider(provider: &SdkLoggerProvider, event: OwnerLogEvent) {
    let scope_name = event.scope.as_str();
    let logger = provider.logger_with_scope(InstrumentationScope::builder(scope_name).build());
    let mut record = logger.create_log_record();

    record.set_event_name(event.event_name);
    record.set_severity_number(event.severity.as_otel());
    record.set_severity_text(event.severity.as_str());
    record.set_body(json_to_any(event.body));
    record.set_trace_context(
        ids::trace_id(current_run_id(), event.tick),
        ids::span_id(current_run_id(), event.tick, scope_name, &event.span_key),
        Some(TraceFlags::SAMPLED),
    );

    for attribute in event.attributes {
        record.add_attribute(Key::new(attribute.key), attribute.value.into_any());
    }

    logger.emit(record);
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use opentelemetry::{InstrumentationScope, logs::AnyValue};
    use opentelemetry_sdk::{
        Resource,
        error::OTelSdkResult,
        logs::{LogProcessor, SdkLogRecord, SdkLoggerProvider},
    };
    use serde_json::json;

    use super::*;

    #[derive(Debug, Clone, Default)]
    struct CaptureProcessor {
        records: Arc<Mutex<Vec<(SdkLogRecord, InstrumentationScope)>>>,
    }

    impl LogProcessor for CaptureProcessor {
        fn emit(&self, record: &mut SdkLogRecord, scope: &InstrumentationScope) {
            self.records
                .lock()
                .expect("capture processor lock poisoned")
                .push((record.clone(), scope.clone()));
        }

        fn force_flush(&self) -> OTelSdkResult {
            Ok(())
        }

        fn shutdown_with_timeout(&self, _timeout: std::time::Duration) -> OTelSdkResult {
            Ok(())
        }
    }

    #[test]
    fn emits_owner_scoped_structured_record() {
        let processor = CaptureProcessor::default();
        let records = processor.records.clone();
        let provider = SdkLoggerProvider::builder()
            .with_resource(Resource::builder_empty().build())
            .with_log_processor(processor)
            .build();

        emit_with_provider(
            &provider,
            OwnerLogEvent {
                scope: OwnerScope::AiGatewayTransport,
                event_name: "request.completed",
                tick: 7,
                span_key: "request:test-1".to_string(),
                severity: OwnerLogSeverity::Info,
                attributes: vec![OwnerLogAttribute::string("ai.backend.id", "test")],
                body: json!({ "summary": "request completed" }),
            },
        );

        let records = records.lock().expect("capture processor lock poisoned");
        assert_eq!(records.len(), 1);
        let (record, scope) = &records[0];
        assert_eq!(scope.name(), "beluna.core.ai-gateway.transport");
        assert_eq!(record.event_name(), Some("request.completed"));
        assert_eq!(record.severity_number(), Some(Severity::Info));
        assert!(record.trace_context().is_some());
        assert!(matches!(record.body(), Some(AnyValue::Map(_))));
        assert!(
            record
                .attributes_iter()
                .any(|(key, value)| key.as_str() == "ai.backend.id"
                    && *value == AnyValue::String("test".into()))
        );
    }
}
