use crate::cortex::{error::CortexError, types::Goal};

pub trait GoalDecomposerPort: Send + Sync {
    fn suggest_affordance(&self, goal: &Goal) -> Result<Option<(String, String)>, CortexError>;
}
