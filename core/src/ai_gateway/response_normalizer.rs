use crate::ai_gateway::{
    error::GatewayError,
    types::RequestId,
    types_chat::{BackendRawEvent, ChatEvent},
};

#[derive(Default, Clone, Copy)]
pub struct ResponseNormalizer;

impl ResponseNormalizer {
    pub fn map_raw(
        &self,
        request_id: &RequestId,
        raw: BackendRawEvent,
    ) -> Result<ChatEvent, GatewayError> {
        let event = match raw {
            BackendRawEvent::OutputTextDelta { delta } => ChatEvent::TextDelta {
                request_id: request_id.clone(),
                delta,
            },
            BackendRawEvent::ToolCallDelta {
                call_id,
                name,
                arguments_delta,
            } => ChatEvent::ToolCallDelta {
                request_id: request_id.clone(),
                call_id,
                name,
                arguments_delta,
            },
            BackendRawEvent::ToolCallReady { call } => ChatEvent::ToolCallReady {
                request_id: request_id.clone(),
                call,
            },
            BackendRawEvent::Usage { usage } => ChatEvent::Usage {
                request_id: request_id.clone(),
                usage,
            },
            BackendRawEvent::Completed { finish_reason } => ChatEvent::Completed {
                request_id: request_id.clone(),
                finish_reason,
            },
            BackendRawEvent::Failed { error } => ChatEvent::Failed {
                request_id: request_id.clone(),
                error,
            },
        };

        Ok(event)
    }

    pub fn is_output_event(event: &ChatEvent) -> bool {
        matches!(
            event,
            ChatEvent::TextDelta { .. }
                | ChatEvent::ToolCallDelta { .. }
                | ChatEvent::ToolCallReady { .. }
        )
    }

    pub fn is_tool_event(event: &ChatEvent) -> bool {
        matches!(
            event,
            ChatEvent::ToolCallDelta { .. } | ChatEvent::ToolCallReady { .. }
        )
    }

    pub fn is_terminal_event(event: &ChatEvent) -> bool {
        matches!(
            event,
            ChatEvent::Completed { .. } | ChatEvent::Failed { .. }
        )
    }
}
