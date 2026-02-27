use crate::ai_gateway::{
    error::{GatewayError, unsupported_capability},
    types::BackendCapabilities,
};

use super::types::{ContentPart, OutputMode, TurnPayload};

#[derive(Default)]
pub struct CapabilityGuard;

impl CapabilityGuard {
    pub(crate) fn assert_supported(
        &self,
        payload: &TurnPayload,
        capabilities: &BackendCapabilities,
    ) -> Result<(), GatewayError> {
        let requests_tool_calls = !payload.tools.is_empty();
        if requests_tool_calls && !capabilities.tool_calls {
            return Err(unsupported_capability(
                "backend does not support tool calling",
            ));
        }

        if matches!(payload.output_mode, OutputMode::JsonObject) && !capabilities.json_mode {
            return Err(unsupported_capability(
                "backend does not support json output mode",
            ));
        }
        if matches!(payload.output_mode, OutputMode::JsonSchema { .. })
            && !capabilities.json_schema_mode
        {
            return Err(unsupported_capability(
                "backend does not support json schema output mode",
            ));
        }

        let needs_vision = payload.messages.iter().any(|message| {
            message
                .parts
                .iter()
                .any(|part| matches!(part, ContentPart::ImageUrl { .. }))
        });
        if needs_vision && !capabilities.vision {
            return Err(unsupported_capability(
                "backend does not support vision inputs",
            ));
        }

        Ok(())
    }
}
