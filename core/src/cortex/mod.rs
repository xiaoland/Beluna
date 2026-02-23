#![allow(dead_code)]

mod clamp;
pub mod cognition;
mod cognition_patch;
mod error;
mod helpers;
mod ir;
mod prompts;
mod runtime;
pub mod testing;
mod types;

pub use cognition::{
    CognitionState, GoalNode, GoalTree, GoalTreePatchOp, L1Memory, ROOT_PARTITION,
    new_default_cognition_state, root_partition_runtime,
};
pub use error::{CortexError, CortexErrorKind};
pub use runtime::{Cortex, CortexTelemetryEvent, CortexTelemetryHook};
pub use types::{CortexOutput, ReactionLimits};
