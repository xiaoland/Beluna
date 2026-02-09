use crate::ai_gateway::{
    error::GatewayError,
    types::{BackendRawEvent, GatewayEvent, RequestId},
};

#[derive(Default, Clone, Copy)]
pub struct ResponseNormalizer;

impl ResponseNormalizer {
    pub fn map_raw(
        &self,
        request_id: &RequestId,
        raw: BackendRawEvent,
    ) -> Result<GatewayEvent, GatewayError> {
        let event = match raw {
            BackendRawEvent::OutputTextDelta { delta } => GatewayEvent::OutputTextDelta {
                request_id: request_id.clone(),
                delta,
            },
            BackendRawEvent::ToolCallDelta {
                call_id,
                name,
                arguments_delta,
            } => GatewayEvent::ToolCallDelta {
                request_id: request_id.clone(),
                call_id,
                name,
                arguments_delta,
            },
            BackendRawEvent::ToolCallReady { call } => GatewayEvent::ToolCallReady {
                request_id: request_id.clone(),
                call,
            },
            BackendRawEvent::Usage { usage } => GatewayEvent::Usage {
                request_id: request_id.clone(),
                usage,
            },
            BackendRawEvent::Completed { finish_reason } => GatewayEvent::Completed {
                request_id: request_id.clone(),
                finish_reason,
            },
            BackendRawEvent::Failed { error } => GatewayEvent::Failed {
                request_id: request_id.clone(),
                error,
            },
        };

        Ok(event)
    }

    pub fn is_output_event(event: &GatewayEvent) -> bool {
        matches!(
            event,
            GatewayEvent::OutputTextDelta { .. }
                | GatewayEvent::ToolCallDelta { .. }
                | GatewayEvent::ToolCallReady { .. }
        )
    }

    pub fn is_tool_event(event: &GatewayEvent) -> bool {
        matches!(
            event,
            GatewayEvent::ToolCallDelta { .. } | GatewayEvent::ToolCallReady { .. }
        )
    }

    pub fn is_terminal_event(event: &GatewayEvent) -> bool {
        matches!(
            event,
            GatewayEvent::Completed { .. } | GatewayEvent::Failed { .. }
        )
    }
}
