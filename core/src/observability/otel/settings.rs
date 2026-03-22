use std::time::Duration;

use anyhow::{Result, anyhow};

use crate::config::{OtlpConfig, OtlpSignalProtocol};

#[derive(Debug, Clone)]
pub struct MetricsSettings {
    pub protocol: OtlpSignalProtocol,
    pub endpoint: String,
    pub timeout_ms: u64,
    pub timeout: Duration,
    pub export_interval_ms: u64,
    pub export_interval: Duration,
}

#[derive(Debug, Clone)]
pub struct LogsSettings {
    pub protocol: OtlpSignalProtocol,
    pub endpoint: String,
    pub timeout_ms: u64,
    pub timeout: Duration,
    pub export_interval_ms: u64,
    pub export_interval: Duration,
}

#[derive(Debug, Clone)]
pub struct TracesSettings {
    pub protocol: OtlpSignalProtocol,
    pub endpoint: String,
    pub timeout_ms: u64,
    pub timeout: Duration,
    pub sampling_ratio: f64,
}

pub fn resolve_metrics(otlp: &OtlpConfig) -> Result<Option<MetricsSettings>> {
    if !otlp.signals.metrics.enabled {
        return Ok(None);
    }

    let endpoint = required_endpoint(
        otlp.signals.metrics.endpoint.as_ref(),
        "observability.otlp.signals.metrics.endpoint",
    )?;
    let timeout_ms = otlp
        .signals
        .metrics
        .timeout_ms
        .unwrap_or(otlp.defaults.timeout_ms);
    let export_interval_ms = otlp.signals.metrics.export_interval_ms;

    Ok(Some(MetricsSettings {
        protocol: otlp.signals.metrics.protocol,
        endpoint,
        timeout_ms,
        timeout: Duration::from_millis(timeout_ms),
        export_interval_ms,
        export_interval: Duration::from_millis(export_interval_ms),
    }))
}

pub fn resolve_logs(otlp: &OtlpConfig) -> Result<Option<LogsSettings>> {
    if !otlp.signals.logs.enabled {
        return Ok(None);
    }

    let endpoint = required_endpoint(
        otlp.signals.logs.endpoint.as_ref(),
        "observability.otlp.signals.logs.endpoint",
    )?;
    let timeout_ms = otlp
        .signals
        .logs
        .timeout_ms
        .unwrap_or(otlp.defaults.timeout_ms);
    let export_interval_ms = otlp.signals.logs.export_interval_ms;

    Ok(Some(LogsSettings {
        protocol: otlp.signals.logs.protocol,
        endpoint,
        timeout_ms,
        timeout: Duration::from_millis(timeout_ms),
        export_interval_ms,
        export_interval: Duration::from_millis(export_interval_ms),
    }))
}

pub fn resolve_traces(otlp: &OtlpConfig) -> Result<Option<TracesSettings>> {
    if !otlp.signals.traces.enabled {
        return Ok(None);
    }

    let endpoint = required_endpoint(
        otlp.signals.traces.endpoint.as_ref(),
        "observability.otlp.signals.traces.endpoint",
    )?;
    let timeout_ms = otlp
        .signals
        .traces
        .timeout_ms
        .unwrap_or(otlp.defaults.timeout_ms);

    Ok(Some(TracesSettings {
        protocol: otlp.signals.traces.protocol,
        endpoint,
        timeout_ms,
        timeout: Duration::from_millis(timeout_ms),
        sampling_ratio: otlp.signals.traces.sampling_ratio,
    }))
}

fn required_endpoint(endpoint: Option<&String>, config_key: &str) -> Result<String> {
    endpoint
        .cloned()
        .ok_or_else(|| anyhow!("config invariant violated: {config_key} must be present"))
}
