use anyhow::{Context, Result};
use opentelemetry_otlp::{SpanExporter, WithExportConfig};
use opentelemetry_sdk::{
    Resource,
    trace::{Sampler, SdkTracerProvider},
};

use crate::config::OtlpSignalProtocol;

use super::settings::TracesSettings;

pub fn build_tracer_provider(
    resource: &Resource,
    settings: &TracesSettings,
) -> Result<SdkTracerProvider> {
    let span_exporter = match settings.protocol {
        OtlpSignalProtocol::Http => SpanExporter::builder()
            .with_http()
            .with_endpoint(settings.endpoint.clone())
            .with_timeout(settings.timeout)
            .build(),
        OtlpSignalProtocol::Grpc => SpanExporter::builder()
            .with_tonic()
            .with_endpoint(settings.endpoint.clone())
            .with_timeout(settings.timeout)
            .build(),
    }
    .context("failed to construct OTLP trace exporter")?;

    let sampler = Sampler::ParentBased(Box::new(Sampler::TraceIdRatioBased(
        settings.sampling_ratio,
    )));

    Ok(SdkTracerProvider::builder()
        .with_resource(resource.clone())
        .with_batch_exporter(span_exporter)
        .with_sampler(sampler)
        .build())
}
