use std::path::PathBuf;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use validator::Validate;

use super::validation::validate_non_empty_path;

fn default_continuity_state_path() -> PathBuf {
    PathBuf::from("./state/continuity.json")
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContinuityRuntimeConfig {
    #[serde(default = "default_continuity_state_path")]
    #[validate(custom(function = "validate_non_empty_path"))]
    pub state_path: PathBuf,
}

impl Default for ContinuityRuntimeConfig {
    fn default() -> Self {
        Self {
            state_path: default_continuity_state_path(),
        }
    }
}
