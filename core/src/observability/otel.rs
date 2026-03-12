use std::time::Duration;

use anyhow::{Context, Result, anyhow};
use opentelemetry::{KeyValue, global};
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::{LogExporter, MetricExporter, WithExportConfig};
use opentelemetry_sdk::{
    Resource,
    error::OTelSdkError,
    logs::{BatchConfigBuilder, BatchLogProcessor, SdkLoggerProvider},
    metrics::{PeriodicReader, SdkMeterProvider},
};
use tracing_subscriber::{Layer, Registry};

use crate::config::ObservabilityConfig;

pub struct OpenTelemetryRuntime {
    endpoint: Option<String>,
    meter_provider: Option<SdkMeterProvider>,
    logger_provider: Option<SdkLoggerProvider>,
}

impl OpenTelemetryRuntime {
    pub fn init(config: &ObservabilityConfig) -> Result<Self> {
        if !config.otlp.enabled {
            return Ok(Self {
                endpoint: None,
                meter_provider: None,
                logger_provider: None,
            });
        }

        let endpoint = config.otlp.endpoint.trim().to_string();
        let timeout = Duration::from_millis(config.otlp.export_timeout_ms.max(1));
        let metrics_interval = Duration::from_millis(config.otlp.metrics_export_interval_ms.max(1));
        let logs_interval = Duration::from_millis(config.otlp.logs_export_interval_ms.max(1));

        let resource = Resource::builder()
            .with_attributes(vec![KeyValue::new("service.name", "beluna.core")])
            .build();

        let metric_exporter = MetricExporter::builder()
            .with_http()
            .with_endpoint(endpoint.clone())
            .with_timeout(timeout)
            .build()
            .context("failed to construct OTLP metric exporter")?;
        let metric_reader = PeriodicReader::builder(metric_exporter)
            .with_interval(metrics_interval)
            .build();
        let meter_provider = SdkMeterProvider::builder()
            .with_resource(resource.clone())
            .with_reader(metric_reader)
            .build();
        global::set_meter_provider(meter_provider.clone());

        let log_exporter = LogExporter::builder()
            .with_http()
            .with_endpoint(endpoint.clone())
            .with_timeout(timeout)
            .build()
            .context("failed to construct OTLP log exporter")?;
        let batch_config = BatchConfigBuilder::default()
            .with_scheduled_delay(logs_interval)
            .build();
        let log_processor = BatchLogProcessor::builder(log_exporter)
            .with_batch_config(batch_config)
            .build();
        let logger_provider = SdkLoggerProvider::builder()
            .with_resource(resource)
            .with_log_processor(log_processor)
            .build();

        Ok(Self {
            endpoint: Some(endpoint),
            meter_provider: Some(meter_provider),
            logger_provider: Some(logger_provider),
        })
    }

    pub fn endpoint(&self) -> Option<&str> {
        self.endpoint.as_deref()
    }

    pub fn log_layer(&self) -> Option<Box<dyn Layer<Registry> + Send + Sync>> {
        self.logger_provider
            .as_ref()
            .map(|provider| {
                Box::new(OpenTelemetryTracingBridge::new(provider))
                    as Box<dyn Layer<Registry> + Send + Sync>
            })
    }

    pub fn shutdown(self) -> Result<()> {
        let mut shutdown_errors = Vec::new();

        if let Some(meter_provider) = self.meter_provider {
            match meter_provider.shutdown() {
                Ok(()) | Err(OTelSdkError::AlreadyShutdown) => {}
                Err(err) => shutdown_errors.push(format!("failed to shutdown OTLP metrics provider: {err}")),
            }
        }
        if let Some(logger_provider) = self.logger_provider {
            match logger_provider.shutdown() {
                Ok(()) | Err(OTelSdkError::AlreadyShutdown) => {}
                Err(err) => shutdown_errors.push(format!("failed to shutdown OTLP log provider: {err}")),
            }
        }

        if shutdown_errors.is_empty() {
            return Ok(());
        }
        Err(anyhow!(shutdown_errors.join("; ")))
    }
}
