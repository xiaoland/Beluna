use crate::ai_gateway::{
    error::{GatewayError, unsupported_capability},
    types::BackendCapabilities,
    types_chat::{
        CanonicalContentPart, CanonicalOutputMode, CanonicalRequest, CanonicalToolChoice,
    },
};

#[derive(Default)]
pub struct CapabilityGuard;

impl CapabilityGuard {
    pub fn assert_supported(
        &self,
        request: &CanonicalRequest,
        capabilities: &BackendCapabilities,
    ) -> Result<(), GatewayError> {
        if request.stream && !capabilities.streaming {
            return Err(unsupported_capability(
                "backend does not support streaming requests",
            ));
        }

        let requests_tool_calls = !request.tools.is_empty()
            || matches!(
                request.tool_choice,
                CanonicalToolChoice::Required | CanonicalToolChoice::Specific { .. }
            );
        if requests_tool_calls && !capabilities.tool_calls {
            return Err(unsupported_capability(
                "backend does not support tool calling",
            ));
        }

        if matches!(request.output_mode, CanonicalOutputMode::JsonObject) && !capabilities.json_mode
        {
            return Err(unsupported_capability(
                "backend does not support json output mode",
            ));
        }
        if matches!(request.output_mode, CanonicalOutputMode::JsonSchema { .. })
            && !capabilities.json_schema_mode
        {
            return Err(unsupported_capability(
                "backend does not support json schema output mode",
            ));
        }

        let needs_vision = request.messages.iter().any(|message| {
            message
                .parts
                .iter()
                .any(|part| matches!(part, CanonicalContentPart::ImageUrl { .. }))
        });
        if needs_vision && !capabilities.vision {
            return Err(unsupported_capability(
                "backend does not support vision inputs",
            ));
        }

        Ok(())
    }
}
