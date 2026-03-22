use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

use super::validation::{validate_non_blank, validate_sampling_ratio};

fn default_observability_export_timeout_ms() -> u64 {
    5_000
}

fn default_enabled_false() -> bool {
    false
}

fn default_observability_otlp_protocol() -> OtlpSignalProtocol {
    OtlpSignalProtocol::Grpc
}

fn default_observability_metrics_export_interval_ms() -> u64 {
    5_000
}

fn default_observability_logs_export_interval_ms() -> u64 {
    2_000
}

fn default_observability_traces_sampling_ratio() -> f64 {
    1.0
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ObservabilityConfig {
    #[serde(default)]
    #[validate(nested)]
    pub otlp: OtlpConfig,
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            otlp: OtlpConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OtlpConfig {
    #[serde(default)]
    #[validate(nested)]
    pub defaults: OtlpDefaultsConfig,
    #[serde(default)]
    #[validate(nested)]
    pub signals: OtlpSignalsConfig,
}

impl Default for OtlpConfig {
    fn default() -> Self {
        Self {
            defaults: OtlpDefaultsConfig::default(),
            signals: OtlpSignalsConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OtlpDefaultsConfig {
    #[serde(default = "default_observability_export_timeout_ms")]
    #[validate(range(min = 1))]
    pub timeout_ms: u64,
}

impl Default for OtlpDefaultsConfig {
    fn default() -> Self {
        Self {
            timeout_ms: default_observability_export_timeout_ms(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum OtlpSignalProtocol {
    Http,
    Grpc,
}

impl OtlpSignalProtocol {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Http => "http",
            Self::Grpc => "grpc",
        }
    }
}

impl Default for OtlpSignalProtocol {
    fn default() -> Self {
        default_observability_otlp_protocol()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OtlpSignalsConfig {
    #[serde(default)]
    #[validate(nested)]
    pub metrics: OtlpMetricsConfig,
    #[serde(default)]
    #[validate(nested)]
    pub traces: OtlpTracesConfig,
    #[serde(default)]
    #[validate(nested)]
    pub logs: OtlpLogsConfig,
}

impl Default for OtlpSignalsConfig {
    fn default() -> Self {
        Self {
            metrics: OtlpMetricsConfig::default(),
            traces: OtlpTracesConfig::default(),
            logs: OtlpLogsConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, JsonSchema)]
#[serde(deny_unknown_fields)]
#[validate(schema(function = "validate_metrics_endpoint_when_enabled"))]
pub struct OtlpMetricsConfig {
    #[serde(default = "default_enabled_false")]
    pub enabled: bool,
    #[serde(default = "default_observability_otlp_protocol")]
    pub protocol: OtlpSignalProtocol,
    #[serde(default)]
    #[validate(custom(function = "validate_non_blank"))]
    pub endpoint: Option<String>,
    #[serde(default)]
    #[validate(range(min = 1))]
    pub timeout_ms: Option<u64>,
    #[serde(default = "default_observability_metrics_export_interval_ms")]
    #[validate(range(min = 1))]
    pub export_interval_ms: u64,
}

impl Default for OtlpMetricsConfig {
    fn default() -> Self {
        Self {
            enabled: default_enabled_false(),
            protocol: default_observability_otlp_protocol(),
            endpoint: None,
            timeout_ms: None,
            export_interval_ms: default_observability_metrics_export_interval_ms(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, JsonSchema)]
#[serde(deny_unknown_fields)]
#[validate(schema(function = "validate_traces_endpoint_when_enabled"))]
pub struct OtlpTracesConfig {
    #[serde(default = "default_enabled_false")]
    pub enabled: bool,
    #[serde(default = "default_observability_otlp_protocol")]
    pub protocol: OtlpSignalProtocol,
    #[serde(default)]
    #[validate(custom(function = "validate_non_blank"))]
    pub endpoint: Option<String>,
    #[serde(default)]
    #[validate(range(min = 1))]
    pub timeout_ms: Option<u64>,
    #[serde(default = "default_observability_traces_sampling_ratio")]
    #[validate(custom(function = "validate_sampling_ratio"))]
    pub sampling_ratio: f64,
}

impl Default for OtlpTracesConfig {
    fn default() -> Self {
        Self {
            enabled: default_enabled_false(),
            protocol: default_observability_otlp_protocol(),
            endpoint: None,
            timeout_ms: None,
            sampling_ratio: default_observability_traces_sampling_ratio(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, JsonSchema)]
#[serde(deny_unknown_fields)]
#[validate(schema(function = "validate_logs_endpoint_when_enabled"))]
pub struct OtlpLogsConfig {
    #[serde(default = "default_enabled_false")]
    pub enabled: bool,
    #[serde(default = "default_observability_otlp_protocol")]
    pub protocol: OtlpSignalProtocol,
    #[serde(default)]
    #[validate(custom(function = "validate_non_blank"))]
    pub endpoint: Option<String>,
    #[serde(default)]
    #[validate(range(min = 1))]
    pub timeout_ms: Option<u64>,
    #[serde(default = "default_observability_logs_export_interval_ms")]
    #[validate(range(min = 1))]
    pub export_interval_ms: u64,
}

impl Default for OtlpLogsConfig {
    fn default() -> Self {
        Self {
            enabled: default_enabled_false(),
            protocol: default_observability_otlp_protocol(),
            endpoint: None,
            timeout_ms: None,
            export_interval_ms: default_observability_logs_export_interval_ms(),
        }
    }
}

fn validate_metrics_endpoint_when_enabled(
    signal: &OtlpMetricsConfig,
) -> Result<(), ValidationError> {
    if signal.enabled && signal.endpoint.is_none() {
        return Err(ValidationError::new("endpoint_required_when_enabled"));
    }
    Ok(())
}

fn validate_traces_endpoint_when_enabled(signal: &OtlpTracesConfig) -> Result<(), ValidationError> {
    if signal.enabled && signal.endpoint.is_none() {
        return Err(ValidationError::new("endpoint_required_when_enabled"));
    }
    Ok(())
}

fn validate_logs_endpoint_when_enabled(signal: &OtlpLogsConfig) -> Result<(), ValidationError> {
    if signal.enabled && signal.endpoint.is_none() {
        return Err(ValidationError::new("endpoint_required_when_enabled"));
    }
    Ok(())
}
