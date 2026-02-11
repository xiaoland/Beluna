#![allow(dead_code)]

pub mod commitment_manager;
pub mod error;
pub mod facade;
pub mod noop;
pub mod planner;
pub mod ports;
pub mod state;
pub mod types;

pub use commitment_manager::CommitmentManager;
pub use error::{CortexError, CortexErrorKind};
pub use facade::CortexFacade;
pub use noop::NoopGoalDecomposer;
pub use planner::{
    DeterministicPlanner, PlannerConfig, derive_attempt_id, derive_cost_attribution_id,
};
pub use ports::GoalDecomposerPort;
pub use state::CortexState;
pub use types::{
    CommitmentId, CommitmentRecord, CommitmentStatus, CortexCommand, CortexCycleOutput,
    CortexEvent, Goal, GoalClass, GoalId, GoalMetadataEntry, GoalMetadataValue, GoalScope,
    MetadataProvenance, MetadataSource, SchedulingContext,
};
