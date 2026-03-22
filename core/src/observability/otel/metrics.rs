use anyhow::{Context, Result};
use opentelemetry_otlp::{MetricExporter, WithExportConfig};
use opentelemetry_sdk::{
    Resource,
    metrics::{PeriodicReader, SdkMeterProvider},
};

use crate::config::OtlpSignalProtocol;

use super::settings::MetricsSettings;

pub fn build_meter_provider(resource: &Resource, settings: &MetricsSettings) -> Result<SdkMeterProvider> {
    let metric_exporter = match settings.protocol {
        OtlpSignalProtocol::Http => MetricExporter::builder()
            .with_http()
            .with_endpoint(settings.endpoint.clone())
            .with_timeout(settings.timeout)
            .build(),
        OtlpSignalProtocol::Grpc => MetricExporter::builder()
            .with_tonic()
            .with_endpoint(settings.endpoint.clone())
            .with_timeout(settings.timeout)
            .build(),
    }
    .context("failed to construct OTLP metric exporter")?;

    let metric_reader = PeriodicReader::builder(metric_exporter)
        .with_interval(settings.export_interval)
        .build();

    Ok(SdkMeterProvider::builder()
        .with_resource(resource.clone())
        .with_reader(metric_reader)
        .build())
}
