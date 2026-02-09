use crate::mind::{
    error::{MindError, policy_violation},
    state::MindState,
    types::{
        Goal, GoalId, GoalRecord, PreemptionDecision, PreemptionDisposition, SafePoint,
        merged_goal_id,
    },
};

pub struct PreemptionContext<'a> {
    pub state: &'a MindState,
    pub active_goal: Option<&'a GoalRecord>,
    pub incoming_goal: &'a Goal,
    pub safe_point: SafePoint,
}

pub trait SafePointPolicy: Send + Sync {
    fn inspect(
        &self,
        state: &MindState,
        active_goal_id: Option<&GoalId>,
    ) -> Result<SafePoint, MindError>;
}

pub trait PreemptionDecider: Send + Sync {
    fn decide(&self, ctx: PreemptionContext<'_>) -> Result<PreemptionDecision, MindError>;
}

#[derive(Debug, Clone)]
pub struct DeterministicSafePointPolicy;

impl SafePointPolicy for DeterministicSafePointPolicy {
    fn inspect(
        &self,
        state: &MindState,
        active_goal_id: Option<&GoalId>,
    ) -> Result<SafePoint, MindError> {
        match active_goal_id {
            Some(goal_id) => {
                let preemptable = state.pending_intents.is_empty();
                Ok(SafePoint {
                    preemptable,
                    checkpoint_token: if preemptable {
                        Some(format!("cp:{}:{}", goal_id, state.cycle_id))
                    } else {
                        None
                    },
                    rationale: if preemptable {
                        "no pending intents; safe to preempt".to_string()
                    } else {
                        "pending intents exist; continue current goal".to_string()
                    },
                })
            }
            None => Ok(SafePoint {
                preemptable: true,
                checkpoint_token: None,
                rationale: "no active goal".to_string(),
            }),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DeterministicPreemptionDecider {
    pub reliability_failure_window: usize,
}

impl Default for DeterministicPreemptionDecider {
    fn default() -> Self {
        Self {
            reliability_failure_window: 3,
        }
    }
}

impl PreemptionDecider for DeterministicPreemptionDecider {
    fn decide(&self, ctx: PreemptionContext<'_>) -> Result<PreemptionDecision, MindError> {
        if !ctx.safe_point.preemptable && ctx.safe_point.checkpoint_token.is_some() {
            return Err(policy_violation(
                "checkpoint token is invalid when safe point is not preemptable",
            ));
        }

        let Some(active_goal) = ctx.active_goal else {
            return Ok(PreemptionDecision {
                disposition: PreemptionDisposition::Continue,
                rationale: "no active goal; activate incoming goal".to_string(),
                safe_point: ctx.safe_point,
                merge_goal_id: None,
            });
        };

        if !ctx.safe_point.preemptable {
            return Ok(PreemptionDecision {
                disposition: PreemptionDisposition::Continue,
                rationale: "active goal is not preemptable at this safe point".to_string(),
                safe_point: ctx.safe_point,
                merge_goal_id: None,
            });
        }

        if merge_compatible(&active_goal.goal, ctx.incoming_goal) {
            return Ok(PreemptionDecision {
                disposition: PreemptionDisposition::Merge,
                rationale: "active and incoming goals are merge-compatible".to_string(),
                safe_point: ctx.safe_point,
                merge_goal_id: Some(merged_goal_id(&active_goal.goal.id, &ctx.incoming_goal.id)),
            });
        }

        if ctx.incoming_goal.priority > active_goal.goal.priority {
            return Ok(PreemptionDecision {
                disposition: PreemptionDisposition::Pause,
                rationale: "incoming goal has higher priority".to_string(),
                safe_point: ctx.safe_point,
                merge_goal_id: None,
            });
        }

        let reliability_failures = ctx.state.recent_criterion_failures(
            Some(&active_goal.goal.id),
            crate::mind::types::EvaluationCriterion::SubsystemReliability,
            self.reliability_failure_window,
        );

        if reliability_failures >= self.reliability_failure_window {
            return Ok(PreemptionDecision {
                disposition: PreemptionDisposition::Cancel,
                rationale: "active goal repeatedly failed reliability checks".to_string(),
                safe_point: ctx.safe_point,
                merge_goal_id: None,
            });
        }

        Ok(PreemptionDecision {
            disposition: PreemptionDisposition::Continue,
            rationale: "keep active goal and backlog incoming goal".to_string(),
            safe_point: ctx.safe_point,
            merge_goal_id: None,
        })
    }
}

pub fn merge_compatible(active: &Goal, incoming: &Goal) -> bool {
    if active.id == incoming.id {
        return false;
    }

    if active.level.abs_diff(incoming.level) > 1 {
        return false;
    }

    if active.parent_goal_id != incoming.parent_goal_id {
        return false;
    }

    true
}

trait GoalLevelDistance {
    fn abs_diff(self, other: Self) -> u8;
}

impl GoalLevelDistance for crate::mind::types::GoalLevel {
    fn abs_diff(self, other: Self) -> u8 {
        let lhs: u8 = match self {
            crate::mind::types::GoalLevel::High => 0,
            crate::mind::types::GoalLevel::Mid => 1,
            crate::mind::types::GoalLevel::Low => 2,
        };
        let rhs: u8 = match other {
            crate::mind::types::GoalLevel::High => 0,
            crate::mind::types::GoalLevel::Mid => 1,
            crate::mind::types::GoalLevel::Low => 2,
        };
        lhs.abs_diff(rhs)
    }
}
