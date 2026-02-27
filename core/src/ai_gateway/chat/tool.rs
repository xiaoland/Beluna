use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::ai_gateway::error::{GatewayError, invalid_request};

/// A tool definition available to AI backends during chat turns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatToolDefinition {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub input_schema: serde_json::Value,
}

/// Specifies how a tool set should be modified at Thread/Turn level.
#[derive(Debug, Clone)]
pub enum ToolOverride {
    /// Add or replace a tool definition (last writer wins by name).
    Set(ChatToolDefinition),
    /// Remove a tool by name.
    Remove(String),
}

/// Validate a tool definition. Returns an error if the definition is malformed.
pub fn validate_tool(tool: &ChatToolDefinition) -> Result<(), GatewayError> {
    if tool.name.trim().is_empty() {
        return Err(invalid_request("tool name must not be empty"));
    }

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

/// Resolve effective tool set from base tools + overrides.
pub fn resolve_tools(
    base: &[ChatToolDefinition],
    overrides: &[ToolOverride],
) -> Vec<ChatToolDefinition> {
    let mut tools: Vec<ChatToolDefinition> = base.to_vec();

    for ov in overrides {
        match ov {
            ToolOverride::Set(def) => {
                if let Some(existing) = tools.iter_mut().find(|t| t.name == def.name) {
                    *existing = def.clone();
                } else {
                    tools.push(def.clone());
                }
            }
            ToolOverride::Remove(name) => {
                tools.retain(|t| t.name != *name);
            }
        }
    }

    tools
}
