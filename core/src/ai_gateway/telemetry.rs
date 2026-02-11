use crate::ai_gateway::{
    error::GatewayErrorKind,
    types::{BackendId, RequestId, UsageStats},
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

#[derive(Default)]
pub struct NoopTelemetrySink;

impl TelemetrySink for NoopTelemetrySink {
    fn on_event(&self, _event: GatewayTelemetryEvent) {}
}
