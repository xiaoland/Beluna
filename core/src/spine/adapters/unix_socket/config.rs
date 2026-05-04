use std::path::{Path, PathBuf};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

fn default_socket_path() -> PathBuf {
    PathBuf::from("beluna.sock")
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct UnixSocketNdjsonAdapterConfig {
    #[validate(custom(function = "validate_non_empty_path"))]
    pub socket_path: PathBuf,
}

impl UnixSocketNdjsonAdapterConfig {
    pub(crate) fn normalize_paths(&mut self, config_base: &Path) {
        if !self.socket_path.is_absolute() {
            self.socket_path = config_base.join(&self.socket_path);
        }
    }
}

impl Default for UnixSocketNdjsonAdapterConfig {
    fn default() -> Self {
        Self {
            socket_path: default_socket_path(),
        }
    }
}

fn validate_non_empty_path(path: &PathBuf) -> Result<(), ValidationError> {
    if path.as_os_str().is_empty() {
        return Err(ValidationError::new("non_empty_path"));
    }
    Ok(())
}
