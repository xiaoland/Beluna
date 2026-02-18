use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, anyhow};
use jsonschema::{JSONSchema, ValidationError};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    ai_gateway::types::AIGatewayConfig,
    body::payloads::{ShellLimits, WebLimits},
    cortex::ReactionLimits,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub ai_gateway: AIGatewayConfig,
    #[serde(default)]
    pub cortex: CortexRuntimeConfig,
    #[serde(default)]
    pub spine: SpineRuntimeConfig,
    #[serde(default)]
    pub r#loop: CoreLoopConfig,
    #[serde(default)]
    pub body: BodyRuntimeConfig,
}

fn default_socket_path() -> PathBuf {
    PathBuf::from("beluna.sock")
}

fn default_inline_act_queue_capacity() -> usize {
    32
}

fn default_inline_sense_queue_capacity() -> usize {
    32
}

fn default_cortex_inbox_capacity() -> usize {
    32
}

fn default_cortex_outbox_capacity() -> usize {
    32
}

fn default_enabled_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpineRuntimeConfig {
    #[serde(default = "default_spine_adapters")]
    pub adapters: Vec<SpineAdapterConfig>,
}

impl Default for SpineRuntimeConfig {
    fn default() -> Self {
        Self {
            adapters: default_spine_adapters(),
        }
    }
}

fn default_spine_adapters() -> Vec<SpineAdapterConfig> {
    vec![
        SpineAdapterConfig::Inline {
            config: InlineAdapterConfig::default(),
        },
        SpineAdapterConfig::UnixSocketNdjson {
            config: UnixSocketNdjsonAdapterConfig {
                socket_path: default_socket_path(),
            },
        },
    ]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum SpineAdapterConfig {
    Inline {
        config: InlineAdapterConfig,
    },
    UnixSocketNdjson {
        config: UnixSocketNdjsonAdapterConfig,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InlineAdapterConfig {
    #[serde(default = "default_inline_act_queue_capacity")]
    pub act_queue_capacity: usize,
    #[serde(default = "default_inline_sense_queue_capacity")]
    pub sense_queue_capacity: usize,
}

impl Default for InlineAdapterConfig {
    fn default() -> Self {
        Self {
            act_queue_capacity: default_inline_act_queue_capacity(),
            sense_queue_capacity: default_inline_sense_queue_capacity(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnixSocketNdjsonAdapterConfig {
    pub socket_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreLoopConfig {
    #[serde(default = "default_sense_queue_capacity")]
    pub sense_queue_capacity: usize,
}

impl Default for CoreLoopConfig {
    fn default() -> Self {
        Self {
            sense_queue_capacity: default_sense_queue_capacity(),
        }
    }
}

fn default_sense_queue_capacity() -> usize {
    32
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BodyRuntimeConfig {
    #[serde(default)]
    pub std_shell: StdShellRuntimeConfig,
    #[serde(default)]
    pub std_web: StdWebRuntimeConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StdShellRuntimeConfig {
    #[serde(default = "default_enabled_true")]
    pub enabled: bool,
    #[serde(default)]
    pub limits: ShellLimits,
}

impl Default for StdShellRuntimeConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            limits: ShellLimits::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StdWebRuntimeConfig {
    #[serde(default = "default_enabled_true")]
    pub enabled: bool,
    #[serde(default)]
    pub limits: WebLimits,
}

impl Default for StdWebRuntimeConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            limits: WebLimits::default(),
        }
    }
}

impl Config {
    pub fn load(config_path: &Path) -> Result<Self> {
        let config_content = fs::read_to_string(config_path)
            .with_context(|| format!("failed to read {}", config_path.display()))?;
        let config_value: Value = json5::from_str(&config_content)
            .with_context(|| format!("failed to parse {}", config_path.display()))?;

        let config_base = config_path.parent().unwrap_or_else(|| Path::new("."));
        let schema_path = resolve_schema_path(config_base, &config_value)?;
        validate_against_schema(&config_value, &schema_path)?;

        let mut config: Config =
            serde_json::from_value(config_value).context("failed to deserialize core config")?;

        for adapter in &mut config.spine.adapters {
            match adapter {
                SpineAdapterConfig::Inline { .. } => {}
                SpineAdapterConfig::UnixSocketNdjson { config } => {
                    if !config.socket_path.is_absolute() {
                        config.socket_path = config_base.join(&config.socket_path);
                    }
                }
            }
        }

        Ok(config)
    }
}

fn resolve_schema_path(config_base: &Path, config_value: &Value) -> Result<PathBuf> {
    if let Some(path_text) = config_value.get("$schema").and_then(|value| value.as_str()) {
        let configured = PathBuf::from(path_text);
        if configured.is_absolute() {
            return Ok(configured);
        }
        return Ok(config_base.join(&configured));
    }

    let root_default = config_base.join("core/beluna.schema.json");
    if root_default.exists() {
        return Ok(root_default);
    }

    let local_default = config_base.join("beluna.schema.json");
    if local_default.exists() {
        return Ok(local_default);
    }

    Err(anyhow!(
        "unable to resolve schema path: expected $schema in config, core/beluna.schema.json, or beluna.schema.json"
    ))
}

fn validate_against_schema(config_value: &Value, schema_path: &Path) -> Result<()> {
    let schema_content = fs::read_to_string(schema_path)
        .with_context(|| format!("failed to read schema {}", schema_path.display()))?;
    let schema: Value = serde_json::from_str(&schema_content)
        .with_context(|| format!("failed to parse schema {}", schema_path.display()))?;

    let compiled =
        JSONSchema::compile(&schema).map_err(|e| anyhow!("failed to compile schema: {e}"))?;

    match compiled.validate(config_value) {
        Ok(()) => Ok(()),
        Err(errors_iter) => {
            let validation_errors: Vec<ValidationError> = errors_iter.collect();
            let messages: Vec<String> = validation_errors
                .into_iter()
                .map(|error| error.to_string())
                .collect();
            Err(anyhow!("config validation failed: {}", messages.join("; ")))
        }
    }
}
