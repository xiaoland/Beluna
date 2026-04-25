use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, anyhow};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::ai_gateway::types::AIGatewayConfig;

mod body;
mod continuity;
mod cortex;
mod logging;
mod observability;
mod runtime_loop;
mod schema;
mod spine;
mod validation;

pub use body::{BodyRuntimeConfig, StdShellRuntimeConfig, StdWebRuntimeConfig};
pub use continuity::ContinuityRuntimeConfig;
pub use cortex::{CortexRoutesConfig, CortexRuntimeConfig};
pub use logging::LoggingConfig;
pub use observability::{
    ObservabilityConfig, OtlpConfig, OtlpDefaultsConfig, OtlpLogsConfig, OtlpMetricsConfig,
    OtlpSignalProtocol, OtlpSignalsConfig, OtlpTracesConfig,
};
pub use runtime_loop::{CoreLoopConfig, TickMissedBehavior};
pub use schema::{generate_schema_json_pretty, generate_schema_value, write_schema_to_path};
pub use spine::{
    InlineAdapterConfig, SpineAdapterConfig, SpineRuntimeConfig, UnixSocketNdjsonAdapterConfig,
};

#[derive(Debug, Clone, Serialize, Deserialize, Validate, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(rename = "$schema", default, skip_serializing_if = "Option::is_none")]
    #[validate(custom(function = "validation::validate_non_blank"))]
    pub schema: Option<String>,
    #[validate(nested)]
    pub ai_gateway: AIGatewayConfig,
    #[serde(default)]
    #[validate(nested)]
    pub logging: LoggingConfig,
    #[serde(default)]
    #[validate(nested)]
    pub observability: ObservabilityConfig,
    #[serde(default)]
    #[validate(nested)]
    pub cortex: CortexRuntimeConfig,
    #[serde(default)]
    #[validate(nested)]
    pub continuity: ContinuityRuntimeConfig,
    #[serde(default)]
    #[validate(nested)]
    pub spine: SpineRuntimeConfig,
    #[serde(default)]
    #[validate(nested)]
    pub r#loop: CoreLoopConfig,
    #[serde(default)]
    #[validate(nested)]
    pub body: BodyRuntimeConfig,
}

impl Config {
    pub fn load(config_path: &Path) -> Result<Self> {
        let config_content = fs::read_to_string(config_path)
            .with_context(|| format!("failed to read {}", config_path.display()))?;
        let mut config: Config = json5::from_str(&config_content)
            .with_context(|| format!("failed to parse {}", config_path.display()))?;

        let config_base = resolve_config_base(config_path)?;
        config.normalize_paths(&config_base);

        config
            .validate()
            .map_err(|err| anyhow!("config validation failed: {err}"))?;

        Ok(config)
    }

    fn normalize_paths(&mut self, config_base: &Path) {
        normalize_path_against_base(&mut self.logging.dir, config_base);

        for adapter in &mut self.spine.adapters {
            if let SpineAdapterConfig::UnixSocketNdjson { config } = adapter {
                normalize_path_against_base(&mut config.socket_path, config_base);
            }
        }

        normalize_path_against_base(&mut self.continuity.state_path, config_base);
    }
}

fn resolve_config_base(config_path: &Path) -> Result<PathBuf> {
    let parent = config_path.parent().unwrap_or_else(|| Path::new("."));
    if parent.is_absolute() {
        return Ok(parent.to_path_buf());
    }

    Ok(std::env::current_dir()
        .context("failed to read current working directory")?
        .join(parent))
}

fn normalize_path_against_base(path: &mut PathBuf, config_base: &Path) {
    if !path.is_absolute() {
        *path = config_base.join(&*path);
    }
}
