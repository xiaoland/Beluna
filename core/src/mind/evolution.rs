use crate::mind::{
    error::MindError,
    state::MindState,
    types::{
        ChangeProposal, EvaluationCriterion, EvaluationReport, EvaluationVerdict, EvolutionAction,
        EvolutionDecision, EvolutionTarget, MemoryDirective, clamp_confidence,
    },
};

pub trait EvolutionDecider: Send + Sync {
    fn decide(
        &self,
        state: &MindState,
        evaluation: &EvaluationReport,
    ) -> Result<EvolutionDecision, MindError>;
}

#[derive(Debug, Clone)]
pub struct DeterministicEvolutionDecider {
    pub persistence_threshold: usize,
}

impl Default for DeterministicEvolutionDecider {
    fn default() -> Self {
        Self {
            persistence_threshold: 2,
        }
    }
}

impl EvolutionDecider for DeterministicEvolutionDecider {
    fn decide(
        &self,
        state: &MindState,
        evaluation: &EvaluationReport,
    ) -> Result<EvolutionDecision, MindError> {
        let active_goal_id = state.active_goal_id.as_ref();
        if active_goal_id.is_none() {
            return Ok(EvolutionDecision::NoChange {
                rationale: "no active goal; skip evolution decision".to_string(),
            });
        }

        let alignment_failures = state.recent_criterion_failures(
            active_goal_id,
            EvaluationCriterion::GoalAlignment,
            self.persistence_threshold,
        );
        let reliability_failures = state.recent_criterion_failures(
            active_goal_id,
            EvaluationCriterion::SubsystemReliability,
            self.persistence_threshold,
        );
        let faithfulness_failures = state.recent_criterion_failures(
            active_goal_id,
            EvaluationCriterion::SignalFaithfulness,
            self.persistence_threshold,
        );

        if alignment_failures < self.persistence_threshold
            && reliability_failures < self.persistence_threshold
            && faithfulness_failures < self.persistence_threshold
        {
            return Ok(EvolutionDecision::NoChange {
                rationale: "failure signals did not pass persistence threshold".to_string(),
            });
        }

        let peak_failure_confidence = evaluation
            .judgments
            .iter()
            .filter(|j| {
                matches!(
                    j.verdict,
                    EvaluationVerdict::Fail | EvaluationVerdict::Borderline
                )
            })
            .map(|j| j.confidence)
            .fold(0.0_f32, f32::max);
        if peak_failure_confidence < 0.5 {
            return Ok(EvolutionDecision::NoChange {
                rationale: "failure confidence is below evolution threshold".to_string(),
            });
        }

        let target = if matches!(
            state.last_memory_directive,
            Some(MemoryDirective::Forget { .. }) | Some(MemoryDirective::Remember { .. })
        ) && alignment_failures >= self.persistence_threshold
        {
            EvolutionTarget::MemoryStructure {
                id: "memory-policy".to_string(),
            }
        } else if faithfulness_failures >= reliability_failures {
            EvolutionTarget::PerceptionPipeline {
                id: "default-perception".to_string(),
            }
        } else {
            EvolutionTarget::Model {
                id: "default-model".to_string(),
            }
        };

        let action = if evaluation.judgments.iter().any(|j| {
            matches!(j.verdict, EvaluationVerdict::Fail)
                && j.rationale.to_lowercase().contains("data")
        }) {
            EvolutionAction::Retrain
        } else if reliability_failures + faithfulness_failures >= self.persistence_threshold + 1 {
            EvolutionAction::Replace
        } else {
            EvolutionAction::Reconfigure
        };

        let evidence_refs: Vec<String> = evaluation
            .judgments
            .iter()
            .filter(|j| {
                matches!(
                    j.verdict,
                    EvaluationVerdict::Fail | EvaluationVerdict::Borderline
                )
            })
            .flat_map(|j| j.evidence_refs.clone())
            .collect();

        let confidence_base = 0.55
            + ((alignment_failures + reliability_failures + faithfulness_failures) as f32 * 0.1);
        let confidence = clamp_confidence(confidence_base);

        Ok(EvolutionDecision::ChangeProposal(ChangeProposal {
            target,
            action,
            rationale: "persistent evaluation failures suggest controlled evolution".to_string(),
            evidence_refs,
            confidence,
        }))
    }
}
