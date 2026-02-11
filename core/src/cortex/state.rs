use std::collections::{BTreeMap, VecDeque};

use crate::{
    admission::types::{AdmissionReport, IntentAttempt},
    cortex::{
        error::{CortexError, invariant_violation},
        types::{
            CommitmentId, CommitmentRecord, CommitmentStatus, Goal, GoalId, SchedulingContext,
        },
    },
};

const MAX_ADMISSION_REPORTS: usize = 64;
const MAX_ATTEMPT_JOURNAL: usize = 256;

#[derive(Debug, Clone, Default)]
pub struct CortexState {
    pub cycle_id: u64,
    pub goals: BTreeMap<GoalId, Goal>,
    pub commitments: BTreeMap<CommitmentId, CommitmentRecord>,
    pub last_scheduling: Vec<SchedulingContext>,
    pub recent_admission_reports: VecDeque<AdmissionReport>,
    pub attempt_journal: VecDeque<IntentAttempt>,
}

impl CortexState {
    pub fn next_cycle(&mut self) -> u64 {
        self.cycle_id = self.cycle_id.saturating_add(1);
        self.cycle_id
    }

    pub fn push_admission_report(&mut self, report: AdmissionReport) {
        self.recent_admission_reports.push_back(report);
        while self.recent_admission_reports.len() > MAX_ADMISSION_REPORTS {
            self.recent_admission_reports.pop_front();
        }
    }

    pub fn push_attempts(&mut self, attempts: &[IntentAttempt]) {
        for attempt in attempts {
            self.attempt_journal.push_back(attempt.clone());
            while self.attempt_journal.len() > MAX_ATTEMPT_JOURNAL {
                self.attempt_journal.pop_front();
            }
        }
    }

    pub fn active_commitments(&self) -> Vec<&CommitmentRecord> {
        self.commitments
            .values()
            .filter(|record| matches!(record.status, CommitmentStatus::Active))
            .collect()
    }

    pub fn assert_invariants(&self) -> Result<(), CortexError> {
        for (goal_id, goal) in &self.goals {
            if let Some(parent_goal_id) = goal.parent_goal_id.as_ref() {
                if !self.goals.contains_key(parent_goal_id) {
                    return Err(invariant_violation(format!(
                        "goal '{}' references unknown parent '{}'",
                        goal_id, parent_goal_id
                    )));
                }
            }
        }

        for (commitment_id, commitment) in &self.commitments {
            if !self.goals.contains_key(&commitment.goal_id) {
                return Err(invariant_violation(format!(
                    "commitment '{}' references unknown goal '{}'",
                    commitment_id, commitment.goal_id
                )));
            }

            if matches!(commitment.status, CommitmentStatus::Failed)
                && commitment.failure_code.is_none()
            {
                return Err(invariant_violation(format!(
                    "failed commitment '{}' missing failure_code",
                    commitment_id
                )));
            }

            if let Some(superseded_by) = commitment.superseded_by.as_ref() {
                if !self.commitments.contains_key(superseded_by) {
                    return Err(invariant_violation(format!(
                        "commitment '{}' superseded_by '{}' does not exist",
                        commitment_id, superseded_by
                    )));
                }
            }
        }

        Ok(())
    }
}
