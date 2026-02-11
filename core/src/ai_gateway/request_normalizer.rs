use std::collections::{BTreeMap, HashSet};

use uuid::Uuid;

use crate::ai_gateway::{
    error::{GatewayError, invalid_request},
    types::{
        BelunaContentPart, BelunaInferenceRequest, BelunaMessage, BelunaRole, BelunaToolDefinition,
        CanonicalContentPart, CanonicalLimits, CanonicalMessage, CanonicalOutputMode,
        CanonicalRequest, CanonicalRole, CanonicalToolChoice, CanonicalToolDefinition, OutputMode,
        ToolChoice,
    },
};

#[derive(Default)]
pub struct RequestNormalizer;

impl RequestNormalizer {
    pub fn normalize(
        &self,
        request: BelunaInferenceRequest,
    ) -> Result<CanonicalRequest, GatewayError> {
        if request.messages.is_empty() {
            return Err(invalid_request("messages must not be empty"));
        }

        for message in &request.messages {
            Self::validate_message_linkage(message)?;
        }

        for tool in &request.tools {
            Self::validate_tool_schema_keywords(tool)?;
        }

        let request_id = request
            .request_id
            .unwrap_or_else(|| Uuid::now_v7().to_string());

        Ok(CanonicalRequest {
            request_id,
            backend_hint: request.backend_id,
            model_override: request.model,
            messages: request
                .messages
                .into_iter()
                .map(Self::map_message)
                .collect::<Result<Vec<_>, _>>()?,
            tools: request
                .tools
                .into_iter()
                .map(Self::map_tool)
                .collect::<Vec<_>>(),
            tool_choice: Self::map_tool_choice(request.tool_choice),
            output_mode: Self::map_output_mode(request.output_mode),
            limits: CanonicalLimits {
                max_output_tokens: request.limits.max_output_tokens,
                max_request_time_ms: request.limits.max_request_time_ms,
            },
            metadata: request.metadata,
            cost_attribution_id: request.cost_attribution_id,
            stream: request.stream,
        })
    }

    fn validate_message_linkage(message: &BelunaMessage) -> Result<(), GatewayError> {
        match message.role {
            BelunaRole::Tool => {
                if message.tool_call_id.is_none() {
                    return Err(invalid_request(
                        "tool role message must include tool_call_id",
                    ));
                }
                if message.tool_name.is_none() {
                    return Err(invalid_request("tool role message must include tool_name"));
                }
                if message
                    .parts
                    .iter()
                    .any(|part| matches!(part, BelunaContentPart::ImageUrl { .. }))
                {
                    return Err(invalid_request(
                        "tool role message parts may only contain text/json",
                    ));
                }
            }
            BelunaRole::System | BelunaRole::User | BelunaRole::Assistant => {
                if message.tool_call_id.is_some() {
                    return Err(invalid_request(
                        "non-tool message must not include tool_call_id",
                    ));
                }
                if message.tool_name.is_some() {
                    return Err(invalid_request(
                        "non-tool message must not include tool_name",
                    ));
                }
            }
        }

        Ok(())
    }

    fn validate_tool_schema_keywords(tool: &BelunaToolDefinition) -> Result<(), GatewayError> {
        let object = tool.input_schema.as_object().ok_or_else(|| {
            invalid_request(format!(
                "tool '{}' input_schema must be a JSON object",
                tool.name
            ))
        })?;

        let allowed: HashSet<&'static str> = [
            "$defs",
            "additionalProperties",
            "allOf",
            "anyOf",
            "default",
            "description",
            "enum",
            "format",
            "items",
            "maxItems",
            "maxLength",
            "maximum",
            "minItems",
            "minLength",
            "minimum",
            "nullable",
            "oneOf",
            "pattern",
            "properties",
            "required",
            "title",
            "type",
        ]
        .into_iter()
        .collect();

        for key in object.keys() {
            if !allowed.contains(key.as_str()) {
                return Err(invalid_request(format!(
                    "tool '{}' input_schema contains unsupported keyword '{}'",
                    tool.name, key
                )));
            }
        }

        Ok(())
    }

    fn map_message(message: BelunaMessage) -> Result<CanonicalMessage, GatewayError> {
        let role = match message.role {
            BelunaRole::System => CanonicalRole::System,
            BelunaRole::User => CanonicalRole::User,
            BelunaRole::Assistant => CanonicalRole::Assistant,
            BelunaRole::Tool => CanonicalRole::Tool,
        };

        let mut parts = Vec::with_capacity(message.parts.len());
        for part in message.parts {
            let mapped = match part {
                BelunaContentPart::Text { text } => CanonicalContentPart::Text { text },
                BelunaContentPart::ImageUrl { url, mime_type } => {
                    CanonicalContentPart::ImageUrl { url, mime_type }
                }
                BelunaContentPart::Json { value } => CanonicalContentPart::Json { value },
            };
            parts.push(mapped);
        }

        Ok(CanonicalMessage {
            role,
            parts,
            tool_call_id: message.tool_call_id,
            tool_name: message.tool_name,
        })
    }

    fn map_tool(tool: BelunaToolDefinition) -> CanonicalToolDefinition {
        CanonicalToolDefinition {
            name: tool.name,
            description: tool.description,
            input_schema: tool.input_schema,
        }
    }

    fn map_tool_choice(choice: ToolChoice) -> CanonicalToolChoice {
        match choice {
            ToolChoice::Auto => CanonicalToolChoice::Auto,
            ToolChoice::None => CanonicalToolChoice::None,
            ToolChoice::Required => CanonicalToolChoice::Required,
            ToolChoice::Specific { name } => CanonicalToolChoice::Specific { name },
        }
    }

    fn map_output_mode(mode: OutputMode) -> CanonicalOutputMode {
        match mode {
            OutputMode::Text => CanonicalOutputMode::Text,
            OutputMode::JsonObject => CanonicalOutputMode::JsonObject,
        }
    }
}

pub fn redact_metadata(metadata: &BTreeMap<String, String>) -> BTreeMap<String, String> {
    metadata
        .iter()
        .map(|(k, v)| {
            let redacted = if k.to_ascii_lowercase().contains("token")
                || k.to_ascii_lowercase().contains("key")
            {
                "<redacted>".to_string()
            } else {
                v.clone()
            };
            (k.clone(), redacted)
        })
        .collect()
}
