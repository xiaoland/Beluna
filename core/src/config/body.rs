use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::body::payloads::{ShellLimits, WebLimits};

fn default_enabled_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, JsonSchema, Default)]
#[serde(deny_unknown_fields)]
pub struct BodyRuntimeConfig {
    #[serde(default)]
    #[validate(nested)]
    pub std_shell: StdShellRuntimeConfig,
    #[serde(default)]
    #[validate(nested)]
    pub std_web: StdWebRuntimeConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StdShellRuntimeConfig {
    #[serde(default = "default_enabled_true")]
    pub enabled: bool,
    #[serde(default)]
    #[validate(nested)]
    pub limits: ShellLimits,
}

impl Default for StdShellRuntimeConfig {
    fn default() -> Self {
        Self {
            enabled: default_enabled_true(),
            limits: ShellLimits::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StdWebRuntimeConfig {
    #[serde(default = "default_enabled_true")]
    pub enabled: bool,
    #[serde(default)]
    #[validate(nested)]
    pub limits: WebLimits,
}

impl Default for StdWebRuntimeConfig {
    fn default() -> Self {
        Self {
            enabled: default_enabled_true(),
            limits: WebLimits::default(),
        }
    }
}
