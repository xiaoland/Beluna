use crate::mind::{
    error::MindError,
    state::MindState,
    types::{ActionIntent, EvaluationReport, GoalId, MemoryDirective},
};

pub trait DelegationCoordinatorPort: Send + Sync {
    fn plan(
        &self,
        state: &MindState,
        goal_id: Option<&GoalId>,
    ) -> Result<Vec<ActionIntent>, MindError>;
}

pub trait MemoryPolicyPort: Send + Sync {
    fn decide(
        &self,
        state: &MindState,
        report: &EvaluationReport,
    ) -> Result<MemoryDirective, MindError>;
}
