use std::time::Duration;

use anyhow::{Context, Result};
use opentelemetry::{InstrumentationScope, logs::LogRecord};
use opentelemetry_otlp::{LogExporter, WithExportConfig};
use opentelemetry_sdk::{
    Resource,
    error::OTelSdkResult,
    logs::{BatchConfigBuilder, BatchLogProcessor, LogProcessor, SdkLogRecord, SdkLoggerProvider},
};

use crate::config::OtlpSignalProtocol;

use super::settings::LogsSettings;

pub fn build_logger_provider(
    resource: &Resource,
    settings: &LogsSettings,
) -> Result<SdkLoggerProvider> {
    let log_exporter = match settings.protocol {
        OtlpSignalProtocol::Http => LogExporter::builder()
            .with_http()
            .with_endpoint(settings.endpoint.clone())
            .with_timeout(settings.timeout)
            .build(),
        OtlpSignalProtocol::Grpc => LogExporter::builder()
            .with_tonic()
            .with_endpoint(settings.endpoint.clone())
            .with_timeout(settings.timeout)
            .build(),
    }
    .context("failed to construct OTLP log exporter")?;

    let batch_config = BatchConfigBuilder::default()
        .with_scheduled_delay(settings.export_interval)
        .build();
    let timestamp_backfill_processor = TimestampBackfillProcessor;
    let log_processor = BatchLogProcessor::builder(log_exporter)
        .with_batch_config(batch_config)
        .build();

    Ok(SdkLoggerProvider::builder()
        .with_resource(resource.clone())
        .with_log_processor(timestamp_backfill_processor)
        .with_log_processor(log_processor)
        .build())
}

/// Backfill log event timestamp from observed timestamp for backends that
/// require an explicit event timestamp field on OTLP logs.
#[derive(Debug)]
struct TimestampBackfillProcessor;

impl LogProcessor for TimestampBackfillProcessor {
    fn emit(&self, record: &mut SdkLogRecord, _instrumentation: &InstrumentationScope) {
        if record.timestamp().is_none() {
            if let Some(observed_timestamp) = record.observed_timestamp() {
                record.set_timestamp(observed_timestamp);
            }
        }
    }

    fn force_flush(&self) -> OTelSdkResult {
        Ok(())
    }

    fn shutdown_with_timeout(&self, _timeout: Duration) -> OTelSdkResult {
        Ok(())
    }
}
