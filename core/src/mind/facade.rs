use std::collections::{BTreeMap, BTreeSet};

use crate::mind::{
    conflict::{ConflictResolver, DeterministicConflictResolver},
    error::{MindError, internal_error, policy_violation},
    evaluator::{DeterministicEvaluator, NormativeEvaluator},
    evolution::{DeterministicEvolutionDecider, EvolutionDecider},
    goal_manager::GoalManager,
    noop::{NoopDelegationCoordinator, NoopMemoryPolicy},
    ports::{DelegationCoordinatorPort, MemoryPolicyPort},
    preemption::{
        DeterministicPreemptionDecider, DeterministicSafePointPolicy, PreemptionContext,
        PreemptionDecider, SafePointPolicy,
    },
    state::MindState,
    types::{
        ConflictCase, DelegationResult, EvaluationCriterion, Goal, GoalRecord, MindCommand,
        MindCycleOutput, MindDecision, MindEvent, PreemptionDecision, PreemptionDisposition,
        SignalObservation, merged_goal_id,
    },
};

pub struct MindFacade {
    state: MindState,
    safe_point_policy: Box<dyn SafePointPolicy>,
    preemption_decider: Box<dyn PreemptionDecider>,
    delegation_port: Box<dyn DelegationCoordinatorPort>,
    evaluator: Box<dyn NormativeEvaluator>,
    conflict_resolver: Box<dyn ConflictResolver>,
    memory_policy: Box<dyn MemoryPolicyPort>,
    evolution_decider: Box<dyn EvolutionDecider>,
}

impl Default for MindFacade {
    fn default() -> Self {
        Self::with_defaults()
    }
}

impl MindFacade {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        state: MindState,
        safe_point_policy: Box<dyn SafePointPolicy>,
        preemption_decider: Box<dyn PreemptionDecider>,
        delegation_port: Box<dyn DelegationCoordinatorPort>,
        evaluator: Box<dyn NormativeEvaluator>,
        conflict_resolver: Box<dyn ConflictResolver>,
        memory_policy: Box<dyn MemoryPolicyPort>,
        evolution_decider: Box<dyn EvolutionDecider>,
    ) -> Self {
        Self {
            state,
            safe_point_policy,
            preemption_decider,
            delegation_port,
            evaluator,
            conflict_resolver,
            memory_policy,
            evolution_decider,
        }
    }

    pub fn with_defaults() -> Self {
        Self::new(
            MindState::default(),
            Box::new(DeterministicSafePointPolicy),
            Box::new(DeterministicPreemptionDecider::default()),
            Box::new(NoopDelegationCoordinator),
            Box::new(DeterministicEvaluator),
            Box::new(DeterministicConflictResolver),
            Box::new(NoopMemoryPolicy),
            Box::new(DeterministicEvolutionDecider::default()),
        )
    }

    pub fn state(&self) -> &MindState {
        &self.state
    }

    pub fn step(&mut self, command: MindCommand) -> Result<MindCycleOutput, MindError> {
        let cycle_id = self.state.next_cycle();
        let mut output = MindCycleOutput {
            cycle_id,
            events: Vec::new(),
            decisions: Vec::new(),
        };

        let active_before = self.state.active_goal_id.clone();
        let proposed_goal = self.apply_command_base_effects(&command, &mut output.events)?;

        if let (Some(incoming_goal), Some(active_goal_id)) = (proposed_goal.as_ref(), active_before)
        {
            let active_goal = self
                .state
                .goals
                .get(&active_goal_id)
                .cloned()
                .ok_or_else(|| {
                    internal_error("active goal missing from state during preemption")
                })?;

            let safe_point = self
                .safe_point_policy
                .inspect(&self.state, Some(&active_goal_id))?;
            if !safe_point.preemptable && safe_point.checkpoint_token.is_some() {
                return Err(policy_violation(
                    "checkpoint token present while safe point is non-preemptable",
                ));
            }

            let preemption = self.preemption_decider.decide(PreemptionContext {
                state: &self.state,
                active_goal: Some(&active_goal),
                incoming_goal,
                safe_point,
            })?;

            self.apply_preemption_decision(
                &active_goal,
                incoming_goal,
                &preemption,
                &mut output.events,
            )?;
            self.state.last_preemption = Some(preemption.clone());
            output
                .decisions
                .push(MindDecision::Preemption(preemption.clone()));
            output.events.push(MindEvent::PreemptionDecided {
                disposition: preemption.disposition,
            });
        }

        let intents = self
            .delegation_port
            .plan(&self.state, self.state.active_goal_id.as_ref())?;
        if !intents.is_empty() {
            self.state.push_intents(intents.clone());
            output.decisions.push(MindDecision::DelegationPlan(intents));
        }

        let evaluation_report = self.evaluator.evaluate(&self.state, &command)?;
        self.state.push_evaluation(evaluation_report.clone());
        output
            .decisions
            .push(MindDecision::Evaluation(evaluation_report.clone()));
        output.events.push(MindEvent::EvaluationCompleted);

        let conflict_cases = self.build_conflict_cases(&evaluation_report);
        let resolutions = self.conflict_resolver.resolve(&conflict_cases)?;
        for resolution in resolutions {
            output.decisions.push(MindDecision::Conflict(resolution));
        }
        output.events.push(MindEvent::ConflictResolved);

        let memory_directive = self.memory_policy.decide(&self.state, &evaluation_report)?;
        self.state.last_memory_directive = Some(memory_directive.clone());
        output
            .decisions
            .push(MindDecision::MemoryPolicy(memory_directive));
        output.events.push(MindEvent::MemoryPolicyApplied);

        let evolution = self
            .evolution_decider
            .decide(&self.state, &evaluation_report)?;
        output.decisions.push(MindDecision::Evolution(evolution));
        output.events.push(MindEvent::EvolutionDecided);

        GoalManager::assert_invariants(&self.state)?;

        Ok(output)
    }

    fn apply_command_base_effects(
        &mut self,
        command: &MindCommand,
        events: &mut Vec<MindEvent>,
    ) -> Result<Option<Goal>, MindError> {
        match command {
            MindCommand::ProposeGoal(goal) => {
                GoalManager::register_goal(&mut self.state, goal.clone())?;
                if self.state.active_goal_id.is_none() {
                    GoalManager::activate_goal(&mut self.state, &goal.id)?;
                    events.push(MindEvent::GoalActivated {
                        goal_id: goal.id.clone(),
                    });
                }
                Ok(Some(goal.clone()))
            }
            MindCommand::ObserveSignal {
                signal_id,
                fidelity_hint,
                payload,
            } => {
                self.state.push_signal(SignalObservation {
                    signal_id: signal_id.clone(),
                    fidelity_hint: *fidelity_hint,
                    payload: payload.clone(),
                });
                Ok(None)
            }
            MindCommand::SubmitDelegationResult(result) => {
                let mut clamped = result.clone();
                clamped.confidence = clamped.confidence.clamp(0.0, 1.0);
                self.state.push_delegation_result(clamped);
                Ok(None)
            }
            MindCommand::EvaluateNow => Ok(None),
        }
    }

    fn apply_preemption_decision(
        &mut self,
        active_goal: &GoalRecord,
        incoming_goal: &Goal,
        decision: &PreemptionDecision,
        events: &mut Vec<MindEvent>,
    ) -> Result<(), MindError> {
        match decision.disposition {
            PreemptionDisposition::Pause => {
                let paused_goal_id =
                    GoalManager::pause_active_goal(&mut self.state, &decision.rationale)?;
                events.push(MindEvent::GoalPaused {
                    goal_id: paused_goal_id,
                });
                GoalManager::activate_goal(&mut self.state, &incoming_goal.id)?;
                events.push(MindEvent::GoalActivated {
                    goal_id: incoming_goal.id.clone(),
                });
            }
            PreemptionDisposition::Cancel => {
                GoalManager::cancel_goal(
                    &mut self.state,
                    &active_goal.goal.id,
                    &decision.rationale,
                )?;
                events.push(MindEvent::GoalCancelled {
                    goal_id: active_goal.goal.id.clone(),
                });
                GoalManager::activate_goal(&mut self.state, &incoming_goal.id)?;
                events.push(MindEvent::GoalActivated {
                    goal_id: incoming_goal.id.clone(),
                });
            }
            PreemptionDisposition::Continue => {}
            PreemptionDisposition::Merge => {
                let merged_id = decision
                    .merge_goal_id
                    .clone()
                    .unwrap_or_else(|| merged_goal_id(&active_goal.goal.id, &incoming_goal.id));

                let mut metadata = BTreeMap::new();
                for (key, value) in &active_goal.goal.metadata {
                    metadata.insert(key.clone(), value.clone());
                }
                for (key, value) in &incoming_goal.metadata {
                    metadata.entry(key.clone()).or_insert_with(|| value.clone());
                }

                let merged_goal = Goal {
                    id: merged_id.clone(),
                    title: format!("{} + {}", active_goal.goal.title, incoming_goal.title),
                    level: active_goal.goal.level.min(incoming_goal.level),
                    parent_goal_id: if active_goal.goal.parent_goal_id
                        == incoming_goal.parent_goal_id
                    {
                        active_goal.goal.parent_goal_id.clone()
                    } else {
                        None
                    },
                    priority: active_goal.goal.priority.max(incoming_goal.priority),
                    created_cycle: self.state.cycle_id,
                    metadata,
                };

                GoalManager::merge_goals(
                    &mut self.state,
                    &active_goal.goal.id,
                    &incoming_goal.id,
                    merged_goal,
                )?;

                events.push(MindEvent::GoalMerged {
                    from_goal_id: incoming_goal.id.clone(),
                    into_goal_id: merged_id.clone(),
                });
                events.push(MindEvent::GoalActivated { goal_id: merged_id });
            }
        }

        Ok(())
    }

    fn build_conflict_cases(
        &self,
        current_report: &crate::mind::types::EvaluationReport,
    ) -> Vec<ConflictCase> {
        let mut cases = Vec::new();

        let mut by_intent: BTreeMap<String, Vec<DelegationResult>> = BTreeMap::new();
        for result in &self.state.recent_delegation_results {
            by_intent
                .entry(result.intent_id.clone())
                .or_default()
                .push(result.clone());
        }

        for (intent_id, candidates) in by_intent {
            let distinct_helpers: BTreeSet<String> = candidates
                .iter()
                .map(|candidate| candidate.helper_id.clone())
                .collect();
            if distinct_helpers.len() > 1 {
                cases.push(ConflictCase::HelperOutputSameIntent {
                    intent_id,
                    candidates,
                });
            }
        }

        if let Some(previous_report) = self.state.previous_evaluation() {
            let previous_by_criterion: BTreeMap<EvaluationCriterion, Vec<_>> = previous_report
                .judgments
                .iter()
                .cloned()
                .fold(BTreeMap::new(), |mut acc, judgment| {
                    acc.entry(judgment.criterion).or_default().push(judgment);
                    acc
                });

            let current_by_criterion: BTreeMap<EvaluationCriterion, Vec<_>> = current_report
                .judgments
                .iter()
                .cloned()
                .fold(BTreeMap::new(), |mut acc, judgment| {
                    acc.entry(judgment.criterion).or_default().push(judgment);
                    acc
                });

            for criterion in [
                EvaluationCriterion::GoalAlignment,
                EvaluationCriterion::SubsystemReliability,
                EvaluationCriterion::SignalFaithfulness,
            ] {
                let mut candidates = Vec::new();
                if let Some(previous) = previous_by_criterion.get(&criterion) {
                    candidates.extend(previous.clone());
                }
                if let Some(current) = current_by_criterion.get(&criterion) {
                    candidates.extend(current.clone());
                }

                let verdict_count: BTreeSet<_> = candidates.iter().map(|j| j.verdict).collect();
                if verdict_count.len() > 1 {
                    cases.push(ConflictCase::EvaluatorVerdictSameCriterion {
                        criterion,
                        candidates,
                    });
                }
            }
        }

        if let Some(last_preemption) = self.state.last_preemption.as_ref() {
            if matches!(last_preemption.disposition, PreemptionDisposition::Merge)
                && last_preemption.merge_goal_id.is_none()
            {
                if let Some(active_goal_id) = self.state.active_goal_id.clone() {
                    cases.push(ConflictCase::MergeCompatibility {
                        active_goal_id: active_goal_id.clone(),
                        incoming_goal_id: active_goal_id,
                        compatible: false,
                    });
                }
            }
        }

        cases
    }
}
