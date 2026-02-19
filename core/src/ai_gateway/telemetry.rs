use std::sync::OnceLock;

use crate::ai_gateway::{
    error::GatewayErrorKind,
    types::{BackendId, RequestId},
    types_chat::UsageStats,
};

#[derive(Debug, Clone)]
pub enum GatewayTelemetryEvent {
    RequestStarted {
        request_id: RequestId,
        backend_id: BackendId,
        model: String,
        cost_attribution_id: Option<String>,
    },
    AttemptStarted {
        request_id: RequestId,
        attempt: u32,
        cost_attribution_id: Option<String>,
    },
    AttemptFailed {
        request_id: RequestId,
        attempt: u32,
        kind: GatewayErrorKind,
        retryable: bool,
        cost_attribution_id: Option<String>,
    },
    StreamFirstEvent {
        request_id: RequestId,
    },
    RequestCompleted {
        request_id: RequestId,
        attempts: u32,
        usage: Option<UsageStats>,
        cost_attribution_id: Option<String>,
    },
    RequestFailed {
        request_id: RequestId,
        attempts: u32,
        error_kind: GatewayErrorKind,
        cost_attribution_id: Option<String>,
    },
    RequestCancelled {
        request_id: RequestId,
        cost_attribution_id: Option<String>,
    },
}

pub trait TelemetrySink: Send + Sync {
    fn on_event(&self, event: GatewayTelemetryEvent);
}

pub fn ai_gateway_debug_enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| {
        std::env::var("BELUNA_DEBUG_AI_GATEWAY")
            .ok()
            .is_some_and(|raw| env_flag_enabled(&raw))
    })
}

pub fn debug_log(message: impl AsRef<str>) {
    if ai_gateway_debug_enabled() {
        eprintln!("[ai_gateway] {}", message.as_ref());
    }
}

fn env_flag_enabled(raw: &str) -> bool {
    matches!(
        raw.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on" | "debug"
    )
}

#[derive(Default)]
pub struct NoopTelemetrySink;

impl TelemetrySink for NoopTelemetrySink {
    fn on_event(&self, _event: GatewayTelemetryEvent) {}
}

#[derive(Default)]
pub struct StderrTelemetrySink;

impl TelemetrySink for StderrTelemetrySink {
    fn on_event(&self, event: GatewayTelemetryEvent) {
        if !ai_gateway_debug_enabled() {
            return;
        }

        match event {
            GatewayTelemetryEvent::RequestStarted {
                request_id,
                backend_id,
                model,
                cost_attribution_id,
            } => {
                eprintln!(
                    "[ai_gateway] request_started request_id={} backend_id={} model={} cost_attribution_id={}",
                    request_id,
                    backend_id,
                    model,
                    cost_attribution_id.as_deref().unwrap_or("-"),
                );
            }
            GatewayTelemetryEvent::AttemptStarted {
                request_id,
                attempt,
                cost_attribution_id,
            } => {
                eprintln!(
                    "[ai_gateway] attempt_started request_id={} attempt={} cost_attribution_id={}",
                    request_id,
                    attempt,
                    cost_attribution_id.as_deref().unwrap_or("-"),
                );
            }
            GatewayTelemetryEvent::AttemptFailed {
                request_id,
                attempt,
                kind,
                retryable,
                cost_attribution_id,
            } => {
                eprintln!(
                    "[ai_gateway] attempt_failed request_id={} attempt={} kind={:?} retryable={} cost_attribution_id={}",
                    request_id,
                    attempt,
                    kind,
                    retryable,
                    cost_attribution_id.as_deref().unwrap_or("-"),
                );
            }
            GatewayTelemetryEvent::StreamFirstEvent { request_id } => {
                eprintln!("[ai_gateway] stream_first_event request_id={}", request_id);
            }
            GatewayTelemetryEvent::RequestCompleted {
                request_id,
                attempts,
                usage,
                cost_attribution_id,
            } => {
                let (input_tokens, output_tokens, total_tokens) = usage
                    .as_ref()
                    .map(|u| (u.input_tokens, u.output_tokens, u.total_tokens))
                    .unwrap_or((None, None, None));
                eprintln!(
                    "[ai_gateway] request_completed request_id={} attempts={} input_tokens={:?} output_tokens={:?} total_tokens={:?} cost_attribution_id={}",
                    request_id,
                    attempts,
                    input_tokens,
                    output_tokens,
                    total_tokens,
                    cost_attribution_id.as_deref().unwrap_or("-"),
                );
            }
            GatewayTelemetryEvent::RequestFailed {
                request_id,
                attempts,
                error_kind,
                cost_attribution_id,
            } => {
                eprintln!(
                    "[ai_gateway] request_failed request_id={} attempts={} error_kind={:?} cost_attribution_id={}",
                    request_id,
                    attempts,
                    error_kind,
                    cost_attribution_id.as_deref().unwrap_or("-"),
                );
            }
            GatewayTelemetryEvent::RequestCancelled {
                request_id,
                cost_attribution_id,
            } => {
                eprintln!(
                    "[ai_gateway] request_cancelled request_id={} cost_attribution_id={}",
                    request_id,
                    cost_attribution_id.as_deref().unwrap_or("-"),
                );
            }
        }
    }
}
