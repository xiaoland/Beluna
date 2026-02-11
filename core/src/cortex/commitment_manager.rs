use crate::cortex::{
    error::{CortexError, invalid_request, policy_violation},
    state::CortexState,
    types::{CommitmentId, CommitmentRecord, CommitmentStatus, Goal, GoalId, SchedulingContext},
};

pub struct CommitmentManager;

impl CommitmentManager {
    pub fn register_goal(state: &mut CortexState, goal: Goal) -> Result<(), CortexError> {
        if state.goals.contains_key(&goal.id) {
            return Err(invalid_request(format!("duplicate goal id '{}'", goal.id)));
        }

        if let Some(parent_goal_id) = goal.parent_goal_id.as_ref() {
            if !state.goals.contains_key(parent_goal_id) {
                return Err(invalid_request(format!(
                    "goal '{}' references unknown parent '{}'",
                    goal.id, parent_goal_id
                )));
            }
        }

        state.goals.insert(goal.id.clone(), goal);
        Ok(())
    }

    pub fn create_commitment(
        state: &mut CortexState,
        goal_id: &GoalId,
        commitment_id: Option<CommitmentId>,
    ) -> Result<CommitmentId, CortexError> {
        if !state.goals.contains_key(goal_id) {
            return Err(invalid_request(format!("unknown goal '{}'", goal_id)));
        }

        let commitment_id = commitment_id.unwrap_or_else(|| {
            let base = format!("com:{}:{}", goal_id, state.cycle_id);
            if !state.commitments.contains_key(&base) {
                return base;
            }

            let mut suffix = 1_u32;
            loop {
                let candidate = format!("{}:{}", base, suffix);
                if !state.commitments.contains_key(&candidate) {
                    break candidate;
                }
                suffix = suffix.saturating_add(1);
            }
        });

        if state.commitments.contains_key(&commitment_id) {
            return Err(invalid_request(format!(
                "duplicate commitment id '{}'",
                commitment_id
            )));
        }

        let record = CommitmentRecord {
            commitment_id: commitment_id.clone(),
            goal_id: goal_id.clone(),
            status: CommitmentStatus::Active,
            created_cycle: state.cycle_id,
            last_transition_cycle: state.cycle_id,
            superseded_by: None,
            failure_code: None,
        };

        state.commitments.insert(commitment_id.clone(), record);
        Ok(commitment_id)
    }

    pub fn set_commitment_status(
        state: &mut CortexState,
        commitment_id: &CommitmentId,
        status: CommitmentStatus,
        superseded_by: Option<CommitmentId>,
        failure_code: Option<String>,
    ) -> Result<(), CortexError> {
        let record = state
            .commitments
            .get_mut(commitment_id)
            .ok_or_else(|| invalid_request(format!("unknown commitment '{}'", commitment_id)))?;

        if status.is_terminal() && record.status.is_terminal() && record.status != status {
            return Err(policy_violation(format!(
                "cannot transition terminal commitment '{}' from {:?} to {:?}",
                commitment_id, record.status, status
            )));
        }

        if matches!(status, CommitmentStatus::Failed) && failure_code.is_none() {
            return Err(invalid_request(format!(
                "failed commitment '{}' requires failure_code",
                commitment_id
            )));
        }

        record.status = status;
        record.last_transition_cycle = state.cycle_id;
        record.superseded_by = superseded_by;
        record.failure_code = failure_code;

        Ok(())
    }

    pub fn recompute_scheduling(state: &CortexState) -> Vec<SchedulingContext> {
        let mut active: Vec<&CommitmentRecord> = state
            .commitments
            .values()
            .filter(|record| matches!(record.status, CommitmentStatus::Active))
            .collect();

        active.sort_by(|lhs, rhs| {
            lhs.created_cycle
                .cmp(&rhs.created_cycle)
                .then_with(|| lhs.commitment_id.cmp(&rhs.commitment_id))
        });

        let total = active.len() as u16;
        active
            .into_iter()
            .enumerate()
            .map(|(index, record)| SchedulingContext {
                commitment_id: record.commitment_id.clone(),
                cycle_id: state.cycle_id,
                dynamic_priority: total.saturating_sub(index as u16),
                queue_position: index as u16,
            })
            .collect()
    }
}
