use crate::mind::{
    error::MindError,
    ports::{DelegationCoordinatorPort, MemoryPolicyPort},
    state::MindState,
    types::{ActionIntent, EvaluationReport, GoalId, MemoryDirective},
};

#[derive(Debug, Clone, Default)]
pub struct NoopDelegationCoordinator;

impl DelegationCoordinatorPort for NoopDelegationCoordinator {
    fn plan(
        &self,
        _state: &MindState,
        _goal_id: Option<&GoalId>,
    ) -> Result<Vec<ActionIntent>, MindError> {
        Ok(Vec::new())
    }
}

#[derive(Debug, Clone, Default)]
pub struct NoopMemoryPolicy;

impl MemoryPolicyPort for NoopMemoryPolicy {
    fn decide(
        &self,
        _state: &MindState,
        _report: &EvaluationReport,
    ) -> Result<MemoryDirective, MindError> {
        Ok(MemoryDirective::KeepTransient)
    }
}
