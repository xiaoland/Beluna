use std::path::PathBuf;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

use super::validation::validate_non_empty_path;

fn default_socket_path() -> PathBuf {
    PathBuf::from("beluna.sock")
}

fn default_inline_act_queue_capacity() -> usize {
    32
}

fn default_inline_sense_queue_capacity() -> usize {
    32
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

#[derive(Debug, Clone, Serialize, Deserialize, Validate, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SpineRuntimeConfig {
    #[serde(default = "default_spine_adapters")]
    #[validate(custom(function = "validate_adapters"))]
    pub adapters: Vec<SpineAdapterConfig>,
}

impl Default for SpineRuntimeConfig {
    fn default() -> Self {
        Self {
            adapters: default_spine_adapters(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum SpineAdapterConfig {
    Inline {
        config: InlineAdapterConfig,
    },
    UnixSocketNdjson {
        config: UnixSocketNdjsonAdapterConfig,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct InlineAdapterConfig {
    #[serde(default = "default_inline_act_queue_capacity")]
    #[validate(range(min = 1))]
    pub act_queue_capacity: usize,
    #[serde(default = "default_inline_sense_queue_capacity")]
    #[validate(range(min = 1))]
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

#[derive(Debug, Clone, Serialize, Deserialize, Validate, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct UnixSocketNdjsonAdapterConfig {
    #[validate(custom(function = "validate_non_empty_path"))]
    pub socket_path: PathBuf,
}

fn validate_adapters(adapters: &[SpineAdapterConfig]) -> Result<(), ValidationError> {
    if adapters.is_empty() {
        return Err(ValidationError::new("adapters_min_len"));
    }

    for adapter in adapters {
        match adapter {
            SpineAdapterConfig::Inline { config } => {
                config
                    .validate()
                    .map_err(|_| ValidationError::new("inline_adapter_invalid"))?;
            }
            SpineAdapterConfig::UnixSocketNdjson { config } => {
                config
                    .validate()
                    .map_err(|_| ValidationError::new("unix_socket_adapter_invalid"))?;
            }
        }
    }

    Ok(())
}
