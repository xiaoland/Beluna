use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::cortex::ReactionLimits;

use super::validation::validate_non_blank;

fn default_cortex_inbox_capacity() -> usize {
    32
}

fn default_cortex_outbox_capacity() -> usize {
    32
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CortexHelperRoutesConfig {
    #[serde(default)]
    #[validate(custom(function = "validate_non_blank"))]
    pub default: Option<String>,
    #[serde(default)]
    #[validate(custom(function = "validate_non_blank"))]
    pub primary: Option<String>,
    #[serde(default)]
    #[validate(custom(function = "validate_non_blank"))]
    pub sense_helper: Option<String>,
    #[serde(default)]
    #[validate(custom(function = "validate_non_blank"))]
    pub acts_helper: Option<String>,
    #[serde(default)]
    #[validate(custom(function = "validate_non_blank"))]
    pub goal_forest_helper: Option<String>,
}

impl Default for CortexHelperRoutesConfig {
    fn default() -> Self {
        Self {
            default: None,
            primary: None,
            sense_helper: None,
            acts_helper: None,
            goal_forest_helper: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CortexRuntimeConfig {
    #[serde(default = "default_cortex_inbox_capacity")]
    #[validate(range(min = 1))]
    pub inbox_capacity: usize,
    #[serde(default = "default_cortex_outbox_capacity")]
    #[validate(range(min = 1))]
    pub outbox_capacity: usize,
    #[serde(default)]
    #[validate(nested)]
    pub default_limits: ReactionLimits,
    #[serde(default)]
    #[validate(nested)]
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
