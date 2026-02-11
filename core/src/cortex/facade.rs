use std::sync::Arc;

use crate::cortex::{
    commitment_manager::CommitmentManager,
    error::CortexError,
    planner::{DeterministicPlanner, PlannerConfig},
    ports::GoalDecomposerPort,
    state::CortexState,
    types::{CortexCommand, CortexCycleOutput, CortexEvent},
};

pub struct CortexFacade {
    state: CortexState,
    planner: DeterministicPlanner,
}

impl Default for CortexFacade {
    fn default() -> Self {
        Self::with_defaults()
    }
}

impl CortexFacade {
    pub fn new(state: CortexState, planner: DeterministicPlanner) -> Self {
        Self { state, planner }
    }

    pub fn with_defaults() -> Self {
        Self {
            state: CortexState::default(),
            planner: DeterministicPlanner::default(),
        }
    }

    pub fn with_decomposer(
        state: CortexState,
        decomposer: Arc<dyn GoalDecomposerPort>,
        config: PlannerConfig,
    ) -> Self {
        Self {
            state,
            planner: DeterministicPlanner::new(decomposer, config),
        }
    }

    pub fn state(&self) -> &CortexState {
        &self.state
    }

    pub fn step(&mut self, command: CortexCommand) -> Result<CortexCycleOutput, CortexError> {
        let cycle_id = self.state.next_cycle();
        let mut events = Vec::new();

        match command {
            CortexCommand::ProposeGoal(goal) => {
                let goal_id = goal.id.clone();
                CommitmentManager::register_goal(&mut self.state, goal)?;
                events.push(CortexEvent::GoalRegistered { goal_id });
            }
            CortexCommand::CommitGoal {
                goal_id,
                commitment_id,
            } => {
                let commitment_id =
                    CommitmentManager::create_commitment(&mut self.state, &goal_id, commitment_id)?;
                events.push(CortexEvent::CommitmentCreated {
                    commitment_id,
                    goal_id,
                });
            }
            CortexCommand::SetCommitmentStatus {
                commitment_id,
                status,
                superseded_by,
                failure_code,
            } => {
                CommitmentManager::set_commitment_status(
                    &mut self.state,
                    &commitment_id,
                    status,
                    superseded_by,
                    failure_code,
                )?;
                events.push(CortexEvent::CommitmentStatusChanged {
                    commitment_id,
                    status,
                });
            }
            CortexCommand::ObserveAdmissionReport(report) => {
                let outcomes = report.outcomes.len();
                let observed_cycle = report.cycle_id;
                self.state.push_admission_report(report);
                events.push(CortexEvent::AdmissionObserved {
                    cycle_id: observed_cycle,
                    outcomes,
                });
            }
            CortexCommand::PlanNow => {}
        }

        let scheduling = CommitmentManager::recompute_scheduling(&self.state);
        let attempts = self.planner.plan(&self.state, &scheduling)?;

        self.state.last_scheduling = scheduling.clone();
        self.state.push_attempts(&attempts);
        self.state.assert_invariants()?;

        Ok(CortexCycleOutput {
            cycle_id,
            events,
            scheduling,
            attempts,
        })
    }
}
