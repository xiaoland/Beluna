mod logs;
mod metrics;
mod settings;
mod traces;

use anyhow::{Result, anyhow};
use opentelemetry::{KeyValue, global};
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_sdk::{
    Resource, error::OTelSdkError, logs::SdkLoggerProvider, metrics::SdkMeterProvider,
    trace::SdkTracerProvider,
};
use tracing_subscriber::{Layer, Registry};

use crate::config::ObservabilityConfig;

const OTEL_SERVICE_NAME: &str = "beluna.core";

#[derive(Debug, Clone)]
pub struct SignalRuntimeState {
    pub signal: &'static str,
    pub requested: bool,
    pub enabled: bool,
    pub protocol: Option<String>,
    pub endpoint: Option<String>,
    pub timeout_ms: Option<u64>,
    pub detail: Option<String>,
}

pub struct OpenTelemetryRuntime {
    meter_provider: Option<SdkMeterProvider>,
    logger_provider: Option<SdkLoggerProvider>,
    tracer_provider: Option<SdkTracerProvider>,
    signal_states: Vec<SignalRuntimeState>,
}

impl OpenTelemetryRuntime {
    pub fn init(config: &ObservabilityConfig) -> Result<Self> {
        let mut runtime = Self {
            meter_provider: None,
            logger_provider: None,
            tracer_provider: None,
            signal_states: Vec::with_capacity(3),
        };

        let resource = Resource::builder()
            .with_attributes(vec![KeyValue::new("service.name", OTEL_SERVICE_NAME)])
            .build();

        runtime.init_metrics(config, &resource);
        runtime.init_logs(config, &resource);
        runtime.init_traces(config, &resource);

        Ok(runtime)
    }

    pub fn log_layer(&self) -> Option<Box<dyn Layer<Registry> + Send + Sync>> {
        self.logger_provider.as_ref().map(|provider| {
            Box::new(OpenTelemetryTracingBridge::new(provider))
                as Box<dyn Layer<Registry> + Send + Sync>
        })
    }

    pub fn trace_layer(&self) -> Option<Box<dyn Layer<Registry> + Send + Sync>> {
        use opentelemetry::trace::TracerProvider as _;

        self.tracer_provider.as_ref().map(|provider| {
            let tracer = provider.tracer(OTEL_SERVICE_NAME);
            Box::new(tracing_opentelemetry::layer().with_tracer(tracer))
                as Box<dyn Layer<Registry> + Send + Sync>
        })
    }

    pub fn signal_states(&self) -> &[SignalRuntimeState] {
        &self.signal_states
    }

    pub fn shutdown(self) -> Result<()> {
        let mut shutdown_errors = Vec::new();

        if let Some(meter_provider) = self.meter_provider {
            match meter_provider.shutdown() {
                Ok(()) | Err(OTelSdkError::AlreadyShutdown) => {}
                Err(err) => {
                    shutdown_errors.push(format!("failed to shutdown OTLP metrics provider: {err}"))
                }
            }
        }

        if let Some(logger_provider) = self.logger_provider {
            match logger_provider.shutdown() {
                Ok(()) | Err(OTelSdkError::AlreadyShutdown) => {}
                Err(err) => {
                    shutdown_errors.push(format!("failed to shutdown OTLP log provider: {err}"))
                }
            }
        }

        if let Some(tracer_provider) = self.tracer_provider {
            match tracer_provider.shutdown() {
                Ok(()) | Err(OTelSdkError::AlreadyShutdown) => {}
                Err(err) => {
                    shutdown_errors.push(format!("failed to shutdown OTLP trace provider: {err}"))
                }
            }
        }

        if shutdown_errors.is_empty() {
            return Ok(());
        }

        Err(anyhow!(shutdown_errors.join("; ")))
    }

    fn init_metrics(&mut self, config: &ObservabilityConfig, resource: &Resource) {
        let resolved = settings::resolve_metrics(&config.otlp);
        let state = match resolved {
            Ok(Some(settings)) => {
                let protocol = settings.protocol.as_str().to_string();
                let endpoint = settings.endpoint.clone();
                let timeout_ms = settings.timeout_ms;
                match metrics::build_meter_provider(resource, &settings) {
                    Ok(provider) => {
                        global::set_meter_provider(provider.clone());
                        self.meter_provider = Some(provider);
                        SignalRuntimeState {
                            signal: "metrics",
                            requested: true,
                            enabled: true,
                            protocol: Some(protocol),
                            endpoint: Some(endpoint),
                            timeout_ms: Some(timeout_ms),
                            detail: Some(format!(
                                "export_interval_ms={} initialized",
                                settings.export_interval_ms
                            )),
                        }
                    }
                    Err(err) => SignalRuntimeState {
                        signal: "metrics",
                        requested: true,
                        enabled: false,
                        protocol: Some(protocol),
                        endpoint: Some(endpoint),
                        timeout_ms: Some(timeout_ms),
                        detail: Some(format!("disabled_after_init_error: {err}")),
                    },
                }
            }
            Ok(None) => SignalRuntimeState {
                signal: "metrics",
                requested: false,
                enabled: false,
                protocol: Some(config.otlp.signals.metrics.protocol.as_str().to_string()),
                endpoint: None,
                timeout_ms: None,
                detail: Some("disabled_by_config".to_string()),
            },
            Err(err) => SignalRuntimeState {
                signal: "metrics",
                requested: true,
                enabled: false,
                protocol: Some(config.otlp.signals.metrics.protocol.as_str().to_string()),
                endpoint: None,
                timeout_ms: None,
                detail: Some(format!("disabled_after_config_error: {err}")),
            },
        };

        self.signal_states.push(state);
    }

    fn init_logs(&mut self, config: &ObservabilityConfig, resource: &Resource) {
        let resolved = settings::resolve_logs(&config.otlp);
        let state = match resolved {
            Ok(Some(settings)) => {
                let protocol = settings.protocol.as_str().to_string();
                let endpoint = settings.endpoint.clone();
                let timeout_ms = settings.timeout_ms;
                match logs::build_logger_provider(resource, &settings) {
                    Ok(provider) => {
                        self.logger_provider = Some(provider);
                        SignalRuntimeState {
                            signal: "logs",
                            requested: true,
                            enabled: true,
                            protocol: Some(protocol),
                            endpoint: Some(endpoint),
                            timeout_ms: Some(timeout_ms),
                            detail: Some(format!(
                                "export_interval_ms={} initialized",
                                settings.export_interval_ms
                            )),
                        }
                    }
                    Err(err) => SignalRuntimeState {
                        signal: "logs",
                        requested: true,
                        enabled: false,
                        protocol: Some(protocol),
                        endpoint: Some(endpoint),
                        timeout_ms: Some(timeout_ms),
                        detail: Some(format!("disabled_after_init_error: {err}")),
                    },
                }
            }
            Ok(None) => SignalRuntimeState {
                signal: "logs",
                requested: false,
                enabled: false,
                protocol: Some(config.otlp.signals.logs.protocol.as_str().to_string()),
                endpoint: None,
                timeout_ms: None,
                detail: Some("disabled_by_config".to_string()),
            },
            Err(err) => SignalRuntimeState {
                signal: "logs",
                requested: true,
                enabled: false,
                protocol: Some(config.otlp.signals.logs.protocol.as_str().to_string()),
                endpoint: None,
                timeout_ms: None,
                detail: Some(format!("disabled_after_config_error: {err}")),
            },
        };

        self.signal_states.push(state);
    }

    fn init_traces(&mut self, config: &ObservabilityConfig, resource: &Resource) {
        let resolved = settings::resolve_traces(&config.otlp);
        let state = match resolved {
            Ok(Some(settings)) => {
                let protocol = settings.protocol.as_str().to_string();
                let endpoint = settings.endpoint.clone();
                let timeout_ms = settings.timeout_ms;
                let sampling_ratio = settings.sampling_ratio;
                match traces::build_tracer_provider(resource, &settings) {
                    Ok(provider) => {
                        global::set_tracer_provider(provider.clone());
                        self.tracer_provider = Some(provider);
                        SignalRuntimeState {
                            signal: "traces",
                            requested: true,
                            enabled: true,
                            protocol: Some(protocol),
                            endpoint: Some(endpoint),
                            timeout_ms: Some(timeout_ms),
                            detail: Some(format!("sampling_ratio={sampling_ratio} initialized")),
                        }
                    }
                    Err(err) => SignalRuntimeState {
                        signal: "traces",
                        requested: true,
                        enabled: false,
                        protocol: Some(protocol),
                        endpoint: Some(endpoint),
                        timeout_ms: Some(timeout_ms),
                        detail: Some(format!("disabled_after_init_error: {err}")),
                    },
                }
            }
            Ok(None) => SignalRuntimeState {
                signal: "traces",
                requested: false,
                enabled: false,
                protocol: Some(config.otlp.signals.traces.protocol.as_str().to_string()),
                endpoint: None,
                timeout_ms: None,
                detail: Some("disabled_by_config".to_string()),
            },
            Err(err) => SignalRuntimeState {
                signal: "traces",
                requested: true,
                enabled: false,
                protocol: Some(config.otlp.signals.traces.protocol.as_str().to_string()),
                endpoint: None,
                timeout_ms: None,
                detail: Some(format!("disabled_after_config_error: {err}")),
            },
        };

        self.signal_states.push(state);
    }
}
