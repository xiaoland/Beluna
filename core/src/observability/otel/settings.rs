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

    let endpoint = resolve_required_endpoint(
        otlp.signals.metrics.endpoint.as_deref(),
        "observability.otlp.signals.metrics.endpoint",
    )?;
    let timeout_ms = resolve_timeout_ms(
        otlp,
        otlp.signals.metrics.timeout_ms,
        "observability.otlp.signals.metrics.timeout_ms",
    )?;
    let export_interval_ms = otlp.signals.metrics.export_interval_ms.max(1);

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

    let endpoint = resolve_required_endpoint(
        otlp.signals.logs.endpoint.as_deref(),
        "observability.otlp.signals.logs.endpoint",
    )?;
    let timeout_ms = resolve_timeout_ms(
        otlp,
        otlp.signals.logs.timeout_ms,
        "observability.otlp.signals.logs.timeout_ms",
    )?;
    let export_interval_ms = otlp.signals.logs.export_interval_ms.max(1);

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

    let endpoint = resolve_required_endpoint(
        otlp.signals.traces.endpoint.as_deref(),
        "observability.otlp.signals.traces.endpoint",
    )?;
    let timeout_ms = resolve_timeout_ms(
        otlp,
        otlp.signals.traces.timeout_ms,
        "observability.otlp.signals.traces.timeout_ms",
    )?;
    validate_sampling_ratio(otlp.signals.traces.sampling_ratio)?;

    Ok(Some(TracesSettings {
        protocol: otlp.signals.traces.protocol,
        endpoint,
        timeout_ms,
        timeout: Duration::from_millis(timeout_ms),
        sampling_ratio: otlp.signals.traces.sampling_ratio,
    }))
}

fn resolve_required_endpoint(endpoint: Option<&str>, endpoint_config_key: &str) -> Result<String> {
    let endpoint = endpoint.ok_or_else(|| {
        anyhow!("{endpoint_config_key} is required when signal export is enabled")
    })?;
    normalize_non_empty(endpoint, endpoint_config_key)
}

fn resolve_timeout_ms(
    otlp: &OtlpConfig,
    timeout_override_ms: Option<u64>,
    timeout_config_key: &str,
) -> Result<u64> {
    if let Some(timeout_ms) = timeout_override_ms {
        if timeout_ms == 0 {
            return Err(anyhow!("{timeout_config_key} must be greater than zero"));
        }
        return Ok(timeout_ms);
    }

    if otlp.defaults.timeout_ms == 0 {
        return Err(anyhow!(
            "observability.otlp.defaults.timeout_ms must be greater than zero"
        ));
    }

    Ok(otlp.defaults.timeout_ms)
}

fn validate_sampling_ratio(sampling_ratio: f64) -> Result<()> {
    if sampling_ratio.is_finite() && (0.0..=1.0).contains(&sampling_ratio) {
        return Ok(());
    }

    Err(anyhow!(
        "observability.otlp.signals.traces.sampling_ratio must be in [0.0, 1.0]"
    ))
}

fn normalize_non_empty(raw: &str, config_key: &str) -> Result<String> {
    let normalized = raw.trim();
    if normalized.is_empty() {
        return Err(anyhow!("{config_key} cannot be empty"));
    }
    Ok(normalized.to_string())
}
