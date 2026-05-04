use std::path::Path;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

pub use crate::spine::adapters::{
    inline::InlineAdapterConfig, unix_socket::UnixSocketNdjsonAdapterConfig,
};

fn default_spine_adapters() -> Vec<SpineAdapterConfig> {
    vec![
        SpineAdapterConfig::Inline {
            config: InlineAdapterConfig::default(),
        },
        SpineAdapterConfig::UnixSocketNdjson {
            config: UnixSocketNdjsonAdapterConfig::default(),
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

impl SpineRuntimeConfig {
    pub(super) fn normalize_paths(&mut self, config_base: &Path) {
        for adapter in &mut self.adapters {
            adapter.normalize_paths(config_base);
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

impl SpineAdapterConfig {
    fn normalize_paths(&mut self, config_base: &Path) {
        match self {
            SpineAdapterConfig::Inline { .. } => {}
            SpineAdapterConfig::UnixSocketNdjson { config } => config.normalize_paths(config_base),
        }
    }
}
