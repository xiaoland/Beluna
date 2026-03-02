#![allow(dead_code)]

mod clamp;
pub mod cognition;
mod cognition_patch;
mod error;
mod helpers;
mod ir;
mod primary;
mod prompts;
pub mod runtime;
pub mod testing;
mod types;

pub use cognition::{
    CognitionState, GoalForest, GoalForestPatchOp, GoalNode, new_default_cognition_state,
};
pub use error::{CortexError, CortexErrorKind};
pub use primary::{Cortex, CortexTelemetryEvent, CortexTelemetryHook};
pub use runtime::{CortexDeps, CortexRuntime, PhysicalStateReadPort};
pub use types::{CortexControlDirective, CortexOutput, EmittedAct, ReactionLimits};
