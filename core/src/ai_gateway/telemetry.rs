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

pub fn emit_gateway_event(event: GatewayTelemetryEvent) {
    match event {
        GatewayTelemetryEvent::RequestStarted {
            request_id,
            backend_id,
            model,
            cost_attribution_id,
        } => {
            tracing::info!(
                target: "ai_gateway",
                event = "request_started",
                request_id = %request_id,
                backend_id = %backend_id,
                model = %model,
                cost_attribution_id = cost_attribution_id.as_deref().unwrap_or("-"),
                "request_started"
            );
        }
        GatewayTelemetryEvent::AttemptStarted {
            request_id,
            attempt,
            cost_attribution_id,
        } => {
            tracing::info!(
                target: "ai_gateway",
                event = "attempt_started",
                request_id = %request_id,
                attempt = attempt,
                cost_attribution_id = cost_attribution_id.as_deref().unwrap_or("-"),
                "attempt_started"
            );
        }
        GatewayTelemetryEvent::AttemptFailed {
            request_id,
            attempt,
            kind,
            retryable,
            cost_attribution_id,
        } => {
            tracing::warn!(
                target: "ai_gateway",
                event = "attempt_failed",
                request_id = %request_id,
                attempt = attempt,
                kind = ?kind,
                retryable = retryable,
                cost_attribution_id = cost_attribution_id.as_deref().unwrap_or("-"),
                "attempt_failed"
            );
        }
        GatewayTelemetryEvent::StreamFirstEvent { request_id } => {
            tracing::info!(
                target: "ai_gateway",
                event = "stream_first_event",
                request_id = %request_id,
                "stream_first_event"
            );
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
            tracing::info!(
                target: "ai_gateway",
                event = "request_completed",
                request_id = %request_id,
                attempts = attempts,
                input_tokens = ?input_tokens,
                output_tokens = ?output_tokens,
                total_tokens = ?total_tokens,
                cost_attribution_id = cost_attribution_id.as_deref().unwrap_or("-"),
                "request_completed"
            );
        }
        GatewayTelemetryEvent::RequestFailed {
            request_id,
            attempts,
            error_kind,
            cost_attribution_id,
        } => {
            tracing::warn!(
                target: "ai_gateway",
                event = "request_failed",
                request_id = %request_id,
                attempts = attempts,
                error_kind = ?error_kind,
                cost_attribution_id = cost_attribution_id.as_deref().unwrap_or("-"),
                "request_failed"
            );
        }
        GatewayTelemetryEvent::RequestCancelled {
            request_id,
            cost_attribution_id,
        } => {
            tracing::info!(
                target: "ai_gateway",
                event = "request_cancelled",
                request_id = %request_id,
                cost_attribution_id = cost_attribution_id.as_deref().unwrap_or("-"),
                "request_cancelled"
            );
        }
    }
}
