use std::path::PathBuf;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use validator::Validate;

use super::validation::{validate_non_blank, validate_non_empty_path};

fn default_enabled_true() -> bool {
    true
}

fn default_logging_dir() -> PathBuf {
    PathBuf::from("./logs/core")
}

fn default_logging_filter() -> String {
    "info".to_string()
}

fn default_logging_retention_days() -> usize {
    14
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LoggingConfig {
    #[serde(default = "default_logging_dir")]
    #[validate(custom(function = "validate_non_empty_path"))]
    pub dir: PathBuf,
    #[serde(default = "default_logging_filter")]
    #[validate(custom(function = "validate_non_blank"))]
    pub filter: String,
    #[serde(default = "default_logging_retention_days")]
    #[validate(range(min = 1))]
    pub retention_days: usize,
    #[serde(default = "default_enabled_true")]
    pub stderr_warn_enabled: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            dir: default_logging_dir(),
            filter: default_logging_filter(),
            retention_days: default_logging_retention_days(),
            stderr_warn_enabled: default_enabled_true(),
        }
    }
}
