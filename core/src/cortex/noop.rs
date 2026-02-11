use crate::cortex::{error::CortexError, ports::GoalDecomposerPort, types::Goal};

#[derive(Debug, Clone, Default)]
pub struct NoopGoalDecomposer;

impl GoalDecomposerPort for NoopGoalDecomposer {
    fn suggest_affordance(&self, _goal: &Goal) -> Result<Option<(String, String)>, CortexError> {
        Ok(None)
    }
}
