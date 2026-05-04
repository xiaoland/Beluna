use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use validator::Validate;

fn default_inline_act_queue_capacity() -> usize {
    32
}

fn default_inline_sense_queue_capacity() -> usize {
    32
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
