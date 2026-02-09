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
    },
    AttemptStarted {
        request_id: RequestId,
        attempt: u32,
    },
    AttemptFailed {
        request_id: RequestId,
        attempt: u32,
        kind: GatewayErrorKind,
        retryable: bool,
    },
    StreamFirstEvent {
        request_id: RequestId,
    },
    RequestCompleted {
        request_id: RequestId,
        attempts: u32,
        usage: Option<UsageStats>,
    },
    RequestFailed {
        request_id: RequestId,
        attempts: u32,
        error_kind: GatewayErrorKind,
    },
    RequestCancelled {
        request_id: RequestId,
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
