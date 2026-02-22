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
    pub logging: LoggingConfig,
    #[serde(default)]
    pub cortex: CortexRuntimeConfig,
    #[serde(default)]
    pub continuity: ContinuityRuntimeConfig,
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

fn default_continuity_state_path() -> PathBuf {
    PathBuf::from("./state/continuity.json")
}

fn default_enabled_true() -> bool {
    true
}

fn default_logging_dir() -> PathBuf {
    PathBuf::from("./logs/core")
}

fn default_logging_filter() -> String {
    "info".to_string()
}

fn default_logging_rotation() -> LoggingRotation {
    LoggingRotation::Daily
}

fn default_logging_retention_days() -> usize {
    14
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum LoggingRotation {
    Daily,
    Hourly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    #[serde(default = "default_logging_dir")]
    pub dir: PathBuf,
    #[serde(default = "default_logging_filter")]
    pub filter: String,
    #[serde(default = "default_logging_rotation")]
    pub rotation: LoggingRotation,
    #[serde(default = "default_logging_retention_days")]
    pub retention_days: usize,
    #[serde(default = "default_enabled_true")]
    pub stderr_warn_enabled: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            dir: default_logging_dir(),
            filter: default_logging_filter(),
            rotation: default_logging_rotation(),
            retention_days: default_logging_retention_days(),
            stderr_warn_enabled: true,
        }
    }
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
pub struct CortexHelperRoutesConfig {
    #[serde(default)]
    pub default: Option<String>,
    #[serde(default)]
    pub primary: Option<String>,
    #[serde(default)]
    pub sense_helper: Option<String>,
    #[serde(default)]
    pub act_descriptor_helper: Option<String>,
    #[serde(default)]
    pub acts_helper: Option<String>,
    #[serde(default)]
    pub goal_tree_helper: Option<String>,
    #[serde(default)]
    pub goal_tree_patch_helper: Option<String>,
    #[serde(default)]
    pub l1_memory_flush_helper: Option<String>,
}

impl Default for CortexHelperRoutesConfig {
    fn default() -> Self {
        Self {
            default: None,
            primary: None,
            sense_helper: None,
            act_descriptor_helper: None,
            acts_helper: None,
            goal_tree_helper: None,
            goal_tree_patch_helper: None,
            l1_memory_flush_helper: None,
        }
    }
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
    pub helper_routes: CortexHelperRoutesConfig,
}

impl Default for CortexRuntimeConfig {
    fn default() -> Self {
        Self {
            inbox_capacity: default_cortex_inbox_capacity(),
            outbox_capacity: default_cortex_outbox_capacity(),
            default_limits: ReactionLimits::default(),
            helper_routes: CortexHelperRoutesConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContinuityRuntimeConfig {
    #[serde(default = "default_continuity_state_path")]
    pub state_path: PathBuf,
}

impl Default for ContinuityRuntimeConfig {
    fn default() -> Self {
        Self {
            state_path: default_continuity_state_path(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TickMissedBehavior {
    Skip,
}

fn default_tick_interval_ms() -> u64 {
    10_000
}

fn default_tick_missed_behavior() -> TickMissedBehavior {
    TickMissedBehavior::Skip
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreLoopConfig {
    #[serde(default = "default_sense_queue_capacity")]
    pub sense_queue_capacity: usize,
    #[serde(default = "default_tick_interval_ms")]
    pub tick_interval_ms: u64,
    #[serde(default = "default_tick_missed_behavior")]
    pub tick_missed_behavior: TickMissedBehavior,
}

impl Default for CoreLoopConfig {
    fn default() -> Self {
        Self {
            sense_queue_capacity: default_sense_queue_capacity(),
            tick_interval_ms: default_tick_interval_ms(),
            tick_missed_behavior: default_tick_missed_behavior(),
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
        if !config.continuity.state_path.is_absolute() {
            config.continuity.state_path = config_base.join(&config.continuity.state_path);
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

#[cfg(test)]
mod tests {
    use std::fs;

    use uuid::Uuid;

    use super::{Config, LoggingConfig, LoggingRotation};

    #[test]
    fn logging_config_defaults_match_contract() {
        let config = LoggingConfig::default();
        assert_eq!(config.dir, std::path::PathBuf::from("./logs/core"));
        assert_eq!(config.filter, "info");
        assert_eq!(config.rotation, LoggingRotation::Daily);
        assert_eq!(config.retention_days, 14);
        assert!(config.stderr_warn_enabled);
    }

    #[test]
    fn logging_rotation_hourly_is_deserialized() {
        #[derive(serde::Deserialize)]
        struct Wrapper {
            logging: LoggingConfig,
        }

        let parsed: Wrapper = serde_json::from_value(serde_json::json!({
            "logging": {
                "rotation": "hourly"
            }
        }))
        .expect("wrapper should deserialize");
        assert_eq!(parsed.logging.rotation, LoggingRotation::Hourly);
    }

    #[test]
    fn config_load_rejects_zero_logging_retention_days() {
        let work_dir = std::env::temp_dir().join(format!("beluna-config-test-{}", Uuid::now_v7()));
        fs::create_dir_all(&work_dir).expect("temp work dir should be created");

        let config_path = work_dir.join("beluna.jsonc");
        let schema_path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("beluna.schema.json");
        let config_text = format!(
            r#"{{
  "$schema": "{}",
  "ai_gateway": {{
    "backends": [
      {{
        "id": "backend-default",
        "dialect": "openai_compatible",
        "credential": {{
          "type": "none"
        }},
        "models": [
          {{
            "id": "m1"
          }}
        ]
      }}
    ],
    "route_aliases": {{
      "default": {{
        "backend_id": "backend-default",
        "model_id": "m1"
      }}
    }}
  }},
  "logging": {{
    "retention_days": 0
  }}
}}"#,
            schema_path.display(),
        );
        fs::write(&config_path, config_text).expect("config should be written");

        let err = Config::load(&config_path).expect_err("retention_days=0 should fail schema");
        assert!(
            err.to_string().contains("minimum"),
            "unexpected error: {err}",
        );

        let _ = fs::remove_file(&config_path);
        let _ = fs::remove_dir(&work_dir);
    }

    #[test]
    fn config_load_rejects_deprecated_cortex_route_fields() {
        let work_dir = std::env::temp_dir().join(format!("beluna-config-test-{}", Uuid::now_v7()));
        fs::create_dir_all(&work_dir).expect("temp work dir should be created");

        let config_path = work_dir.join("beluna.jsonc");
        let schema_path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("beluna.schema.json");
        let config_text = format!(
            r#"{{
  "$schema": "{}",
  "ai_gateway": {{
    "backends": [
      {{
        "id": "backend-default",
        "dialect": "openai_compatible",
        "credential": {{
          "type": "none"
        }},
        "models": [
          {{
            "id": "m1"
          }}
        ]
      }}
    ],
    "route_aliases": {{
      "default": {{
        "backend_id": "backend-default",
        "model_id": "m1"
      }}
    }}
  }},
  "cortex": {{
    "primary_route": "default"
  }}
}}"#,
            schema_path.display(),
        );
        fs::write(&config_path, config_text).expect("config should be written");

        let err = Config::load(&config_path)
            .expect_err("deprecated primary_route should fail schema validation");
        assert!(
            err.to_string().contains("Additional properties"),
            "unexpected error: {err}",
        );

        let _ = fs::remove_file(&config_path);
        let _ = fs::remove_dir(&work_dir);
    }
}
