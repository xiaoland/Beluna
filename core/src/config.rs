use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{ai_gateway::types::AIGatewayConfig, cortex::ReactionLimits};
use anyhow::{Context, Result, anyhow};
use jsonschema::{JSONSchema, ValidationError};
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct Config {
    pub socket_path: PathBuf,
    #[allow(dead_code)]
    pub ai_gateway: AIGatewayConfig,
    pub cortex: CortexRuntimeConfig,
}

#[derive(Debug, Deserialize)]
struct RawConfig {
    #[serde(default = "default_socket_path")]
    socket_path: PathBuf,
    ai_gateway: AIGatewayConfig,
    #[serde(default)]
    cortex: CortexRuntimeConfig,
}

fn default_socket_path() -> PathBuf {
    PathBuf::from("beluna.sock")
}

fn default_cortex_inbox_capacity() -> usize {
    32
}

fn default_cortex_outbox_capacity() -> usize {
    32
}

#[derive(Debug, Clone, Deserialize)]
pub struct CortexRuntimeConfig {
    #[serde(default = "default_cortex_inbox_capacity")]
    pub inbox_capacity: usize,
    #[serde(default = "default_cortex_outbox_capacity")]
    pub outbox_capacity: usize,
    #[serde(default)]
    pub default_limits: ReactionLimits,
    #[serde(default)]
    pub primary_backend_id: Option<String>,
    #[serde(default)]
    pub sub_backend_id: Option<String>,
}

impl Default for CortexRuntimeConfig {
    fn default() -> Self {
        Self {
            inbox_capacity: default_cortex_inbox_capacity(),
            outbox_capacity: default_cortex_outbox_capacity(),
            default_limits: ReactionLimits::default(),
            primary_backend_id: None,
            sub_backend_id: None,
        }
    }
}

impl Config {
    pub fn load(config_path: &Path) -> Result<Self> {
        let base_dir = config_path.parent().unwrap_or_else(|| Path::new("."));

        // Load schema
        let schema_path = base_dir.join("beluna.schema.json");
        let schema_content = fs::read_to_string(&schema_path)
            .with_context(|| format!("unable to read schema {}", schema_path.display()))?;
        let schema: Value = serde_json::from_str(&schema_content)
            .with_context(|| format!("unable to parse schema {}", schema_path.display()))?;

        let raw_config = fs::read_to_string(config_path)
            .with_context(|| format!("unable to read {}", config_path.display()))?;

        // Parse to Value for validation
        let config_value: Value = json5::from_str(&raw_config)
            .with_context(|| format!("unable to parse {}", config_path.display()))?;

        // Validate against schema
        let compiled_schema =
            JSONSchema::compile(&schema).map_err(|e| anyhow!("unable to compile schema: {}", e))?;

        match compiled_schema.validate(&config_value) {
            Ok(()) => {}
            Err(errors_iter) => {
                let validation_errors: Vec<ValidationError> = errors_iter.collect();
                let error_messages: Vec<String> = validation_errors
                    .into_iter()
                    .map(|e| e.to_string())
                    .collect();
                return Err(anyhow::anyhow!(
                    "config validation failed: {}",
                    error_messages.join("; ")
                ));
            }
        }

        // Now parse to RawConfig
        let parsed: RawConfig = serde_json::from_value(config_value)
            .with_context(|| format!("unable to deserialize {}", config_path.display()))?;

        let socket_path = if parsed.socket_path.is_absolute() {
            parsed.socket_path
        } else {
            base_dir.join(parsed.socket_path)
        };

        Ok(Self {
            socket_path,
            ai_gateway: parsed.ai_gateway,
            cortex: parsed.cortex,
        })
    }
}
