use crate::mind::{
    error::{MindError, invalid_request, invariant_violation, policy_violation},
    state::MindState,
    types::{Goal, GoalId, GoalLevel, GoalRecord, GoalStatus},
};

pub struct GoalManager;

impl GoalManager {
    pub fn register_goal(state: &mut MindState, goal: Goal) -> Result<(), MindError> {
        if state.goals.contains_key(&goal.id) {
            return Err(invalid_request(format!("duplicate goal id '{}'", goal.id)));
        }

        match goal.level {
            GoalLevel::High => {}
            GoalLevel::Mid | GoalLevel::Low => {
                let parent = goal.parent_goal_id.as_ref().ok_or_else(|| {
                    invalid_request(format!("goal '{}' requires parent_goal_id", goal.id))
                })?;
                if !state.goals.contains_key(parent) {
                    return Err(invalid_request(format!(
                        "goal '{}' references unknown parent '{}'",
                        goal.id, parent
                    )));
                }
            }
        }

        state.goals.insert(
            goal.id.clone(),
            GoalRecord {
                goal,
                status: GoalStatus::Proposed,
                merged_into: None,
            },
        );
        Ok(())
    }

    pub fn activate_goal(state: &mut MindState, goal_id: &GoalId) -> Result<(), MindError> {
        if let Some(active) = state.active_goal_id.as_ref() {
            if active != goal_id {
                return Err(policy_violation(format!(
                    "cannot activate '{}' while '{}' is active",
                    goal_id, active
                )));
            }
            return Ok(());
        }

        let record = state
            .goals
            .get_mut(goal_id)
            .ok_or_else(|| invalid_request(format!("unknown goal '{}'", goal_id)))?;

        if record.status.is_terminal() {
            return Err(policy_violation(format!(
                "cannot activate terminal goal '{}'",
                goal_id
            )));
        }

        record.status = GoalStatus::Active;
        state.active_goal_id = Some(goal_id.clone());
        Ok(())
    }

    pub fn pause_active_goal(state: &mut MindState, _rationale: &str) -> Result<GoalId, MindError> {
        let active_goal_id = state
            .active_goal_id
            .clone()
            .ok_or_else(|| policy_violation("cannot pause without active goal"))?;

        let record = state
            .goals
            .get_mut(&active_goal_id)
            .ok_or_else(|| invariant_violation("active goal id not found in goal map"))?;
        record.status = GoalStatus::Paused;
        state.active_goal_id = None;
        Ok(active_goal_id)
    }

    pub fn cancel_goal(
        state: &mut MindState,
        goal_id: &GoalId,
        _rationale: &str,
    ) -> Result<(), MindError> {
        let record = state
            .goals
            .get_mut(goal_id)
            .ok_or_else(|| invalid_request(format!("unknown goal '{}'", goal_id)))?;

        if matches!(record.status, GoalStatus::Completed | GoalStatus::Merged) {
            return Err(policy_violation(format!(
                "cannot cancel goal '{}' from status {:?}",
                goal_id, record.status
            )));
        }

        record.status = GoalStatus::Cancelled;
        if state.active_goal_id.as_ref() == Some(goal_id) {
            state.active_goal_id = None;
        }
        Ok(())
    }

    pub fn merge_goals(
        state: &mut MindState,
        active_goal_id: &GoalId,
        incoming_goal_id: &GoalId,
        merged_goal: Goal,
    ) -> Result<(), MindError> {
        if !state.goals.contains_key(active_goal_id) {
            return Err(invalid_request(format!(
                "unknown active goal '{}'",
                active_goal_id
            )));
        }
        if !state.goals.contains_key(incoming_goal_id) {
            return Err(invalid_request(format!(
                "unknown incoming goal '{}'",
                incoming_goal_id
            )));
        }

        Self::register_goal(state, merged_goal.clone())?;

        for source_id in [active_goal_id, incoming_goal_id] {
            let source = state
                .goals
                .get_mut(source_id)
                .ok_or_else(|| invariant_violation("merge source disappeared"))?;
            source.status = GoalStatus::Merged;
            source.merged_into = Some(merged_goal.id.clone());
        }

        let merged = state
            .goals
            .get_mut(&merged_goal.id)
            .ok_or_else(|| invariant_violation("merged goal not found after insert"))?;
        merged.status = GoalStatus::Active;
        state.active_goal_id = Some(merged_goal.id);

        Ok(())
    }

    pub fn assert_invariants(state: &MindState) -> Result<(), MindError> {
        let mut active_count = 0usize;
        for (goal_id, record) in &state.goals {
            if matches!(record.status, GoalStatus::Active) {
                active_count = active_count.saturating_add(1);
            }

            match record.goal.level {
                GoalLevel::High => {}
                GoalLevel::Mid | GoalLevel::Low => {
                    let parent = record.goal.parent_goal_id.as_ref().ok_or_else(|| {
                        invariant_violation(format!("goal '{}' missing parent_goal_id", goal_id))
                    })?;
                    if !state.goals.contains_key(parent) {
                        return Err(invariant_violation(format!(
                            "goal '{}' parent '{}' does not exist",
                            goal_id, parent
                        )));
                    }
                }
            }

            if matches!(record.status, GoalStatus::Merged) && record.merged_into.is_none() {
                return Err(invariant_violation(format!(
                    "goal '{}' is merged but merged_into is none",
                    goal_id
                )));
            }
        }

        if active_count > 1 {
            return Err(invariant_violation(format!(
                "multiple active goals detected: {}",
                active_count
            )));
        }

        match state.active_goal_id.as_ref() {
            Some(active_goal_id) => {
                let record = state.goals.get(active_goal_id).ok_or_else(|| {
                    invariant_violation(format!(
                        "active_goal_id '{}' missing from goal map",
                        active_goal_id
                    ))
                })?;
                if !matches!(record.status, GoalStatus::Active) {
                    return Err(invariant_violation(format!(
                        "active_goal_id '{}' is not marked active",
                        active_goal_id
                    )));
                }
            }
            None => {
                if active_count > 0 {
                    return Err(invariant_violation(
                        "active_goal_id is none but an active goal exists",
                    ));
                }
            }
        }

        Ok(())
    }
}
