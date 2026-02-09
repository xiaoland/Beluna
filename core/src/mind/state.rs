use std::collections::{BTreeMap, VecDeque};

use crate::mind::types::{
    ActionIntent, DelegationResult, EvaluationCriterion, EvaluationReport, GoalId, GoalRecord,
    MemoryDirective, PreemptionDecision, SignalObservation,
};

const MAX_PENDING_INTENTS: usize = 128;
const MAX_RECENT_EVALUATIONS: usize = 64;
const MAX_RECENT_DELEGATION_RESULTS: usize = 128;
const MAX_RECENT_SIGNALS: usize = 128;

#[derive(Debug, Clone)]
pub struct MindState {
    pub cycle_id: u64,
    pub goals: BTreeMap<GoalId, GoalRecord>,
    pub active_goal_id: Option<GoalId>,
    pub pending_intents: VecDeque<ActionIntent>,
    pub recent_evaluations: VecDeque<EvaluationReport>,
    pub recent_delegation_results: VecDeque<DelegationResult>,
    pub recent_signals: VecDeque<SignalObservation>,
    pub last_preemption: Option<PreemptionDecision>,
    pub last_memory_directive: Option<MemoryDirective>,
}

impl Default for MindState {
    fn default() -> Self {
        Self {
            cycle_id: 0,
            goals: BTreeMap::new(),
            active_goal_id: None,
            pending_intents: VecDeque::new(),
            recent_evaluations: VecDeque::new(),
            recent_delegation_results: VecDeque::new(),
            recent_signals: VecDeque::new(),
            last_preemption: None,
            last_memory_directive: None,
        }
    }
}

impl MindState {
    pub fn next_cycle(&mut self) -> u64 {
        self.cycle_id = self.cycle_id.saturating_add(1);
        self.cycle_id
    }

    pub fn push_intents(&mut self, intents: Vec<ActionIntent>) {
        for intent in intents {
            self.pending_intents.push_back(intent);
            while self.pending_intents.len() > MAX_PENDING_INTENTS {
                self.pending_intents.pop_front();
            }
        }
    }

    pub fn push_evaluation(&mut self, mut report: EvaluationReport) {
        for judgment in &mut report.judgments {
            judgment.confidence = judgment.confidence.clamp(0.0, 1.0);
        }
        self.recent_evaluations.push_back(report);
        while self.recent_evaluations.len() > MAX_RECENT_EVALUATIONS {
            self.recent_evaluations.pop_front();
        }
    }

    pub fn push_delegation_result(&mut self, mut result: DelegationResult) {
        result.confidence = result.confidence.clamp(0.0, 1.0);
        self.recent_delegation_results.push_back(result);
        while self.recent_delegation_results.len() > MAX_RECENT_DELEGATION_RESULTS {
            self.recent_delegation_results.pop_front();
        }
    }

    pub fn push_signal(&mut self, signal: SignalObservation) {
        self.recent_signals.push_back(signal);
        while self.recent_signals.len() > MAX_RECENT_SIGNALS {
            self.recent_signals.pop_front();
        }
    }

    pub fn recent_criterion_failures(
        &self,
        goal_id: Option<&GoalId>,
        criterion: EvaluationCriterion,
        limit: usize,
    ) -> usize {
        self.recent_evaluations
            .iter()
            .rev()
            .filter(|report| match (goal_id, report.goal_id.as_ref()) {
                (Some(expected), Some(current)) => expected == current,
                (Some(_), None) => false,
                (None, _) => true,
            })
            .take(limit)
            .map(|report| {
                report
                    .judgments
                    .iter()
                    .filter(|j| {
                        j.criterion == criterion
                            && matches!(j.verdict, crate::mind::types::EvaluationVerdict::Fail)
                    })
                    .count()
            })
            .sum()
    }

    pub fn previous_evaluation(&self) -> Option<&EvaluationReport> {
        let mut iter = self.recent_evaluations.iter().rev();
        iter.next()?;
        iter.next()
    }
}
