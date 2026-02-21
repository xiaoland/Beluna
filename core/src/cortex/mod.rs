#![allow(dead_code)]

mod clamp;
mod error;
mod helpers_input;
mod helpers_output;
mod ir;
mod runtime;
pub mod testing;
mod types;

pub use error::{CortexError, CortexErrorKind};
pub use runtime::{Cortex, CortexTelemetryEvent, CortexTelemetryHook};
pub use types::{CortexOutput, ReactionLimits};
