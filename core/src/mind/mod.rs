#![allow(dead_code)]

pub mod conflict;
pub mod error;
pub mod evaluator;
pub mod evolution;
pub mod facade;
pub mod goal_manager;
pub mod noop;
pub mod ports;
pub mod preemption;
pub mod state;
pub mod types;

pub use conflict::{ConflictResolver, DeterministicConflictResolver};
pub use error::{MindError, MindErrorKind};
pub use evaluator::{DeterministicEvaluator, NormativeEvaluator};
pub use evolution::{DeterministicEvolutionDecider, EvolutionDecider};
pub use facade::MindFacade;
pub use goal_manager::GoalManager;
pub use noop::{NoopDelegationCoordinator, NoopMemoryPolicy};
pub use ports::{DelegationCoordinatorPort, MemoryPolicyPort};
pub use preemption::{
    DeterministicPreemptionDecider, DeterministicSafePointPolicy, PreemptionContext,
    PreemptionDecider, SafePointPolicy, merge_compatible,
};
pub use state::MindState;
pub use types::{
    ActionIntent, ChangeProposal, CheckpointToken, ConflictCase, ConflictResolution, CycleId,
    DelegationResult, EvaluationCriterion, EvaluationReport, EvaluationVerdict, EvolutionAction,
    EvolutionDecision, EvolutionTarget, Goal, GoalId, GoalLevel, GoalRecord, GoalStatus, IntentId,
    IntentKind, Judgment, MemoryDirective, MindCommand, MindCycleOutput, MindDecision, MindEvent,
    PreemptionDecision, PreemptionDisposition, SafePoint, SignalObservation,
};
