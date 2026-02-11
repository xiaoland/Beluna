#![allow(dead_code)]

pub mod error;
pub mod noop;
pub mod ports;
pub mod types;

pub use error::{SpineError, SpineErrorKind};
pub use noop::DeterministicNoopSpine;
pub use ports::SpineExecutorPort;
pub use types::{
    ActionId, AdmittedAction, AdmittedActionBatch, CostAttributionId, CostVector,
    OrderedSpineEvent, ReserveEntryId, SpineEvent, SpineExecutionMode, SpineExecutionReport,
};
