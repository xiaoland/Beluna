use crate::mind::{
    error::MindError,
    state::MindState,
    types::{
        EvaluationCriterion, EvaluationReport, EvaluationVerdict, Judgment, MindCommand,
        clamp_confidence,
    },
};

pub trait NormativeEvaluator: Send + Sync {
    fn evaluate(
        &self,
        state: &MindState,
        command: &MindCommand,
    ) -> Result<EvaluationReport, MindError>;
}

#[derive(Debug, Clone, Default)]
pub struct DeterministicEvaluator;

impl NormativeEvaluator for DeterministicEvaluator {
    fn evaluate(
        &self,
        state: &MindState,
        command: &MindCommand,
    ) -> Result<EvaluationReport, MindError> {
        let goal_id = state.active_goal_id.clone();

        let alignment = if let Some(active_goal_id) = state.active_goal_id.as_ref() {
            Judgment {
                criterion: EvaluationCriterion::GoalAlignment,
                verdict: EvaluationVerdict::Pass,
                confidence: 0.80,
                rationale: format!("active goal '{}' exists", active_goal_id),
                evidence_refs: vec!["goal-state".to_string()],
            }
        } else {
            Judgment {
                criterion: EvaluationCriterion::GoalAlignment,
                verdict: EvaluationVerdict::Unknown,
                confidence: 0.30,
                rationale: "no active goal to align against".to_string(),
                evidence_refs: vec!["goal-state".to_string()],
            }
        };

        let reliability = match command {
            MindCommand::SubmitDelegationResult(result) if result.confidence < 0.35 => Judgment {
                criterion: EvaluationCriterion::SubsystemReliability,
                verdict: EvaluationVerdict::Fail,
                confidence: 0.85,
                rationale: "delegation result confidence is below reliability threshold"
                    .to_string(),
                evidence_refs: vec![result.intent_id.clone()],
            },
            MindCommand::SubmitDelegationResult(result) if result.confidence < 0.55 => Judgment {
                criterion: EvaluationCriterion::SubsystemReliability,
                verdict: EvaluationVerdict::Borderline,
                confidence: 0.70,
                rationale: "delegation result confidence is borderline".to_string(),
                evidence_refs: vec![result.intent_id.clone()],
            },
            _ => Judgment {
                criterion: EvaluationCriterion::SubsystemReliability,
                verdict: EvaluationVerdict::Pass,
                confidence: 0.65,
                rationale: "no reliability degradation signal observed".to_string(),
                evidence_refs: vec!["delegation-history".to_string()],
            },
        };

        let faithfulness = match command {
            MindCommand::ObserveSignal {
                signal_id,
                fidelity_hint: Some(hint),
                ..
            } if *hint < 0.40 => Judgment {
                criterion: EvaluationCriterion::SignalFaithfulness,
                verdict: EvaluationVerdict::Fail,
                confidence: 0.90,
                rationale: format!("signal '{}' has low fidelity hint", signal_id),
                evidence_refs: vec![signal_id.clone()],
            },
            MindCommand::ObserveSignal {
                signal_id,
                fidelity_hint: Some(hint),
                ..
            } if *hint < 0.70 => Judgment {
                criterion: EvaluationCriterion::SignalFaithfulness,
                verdict: EvaluationVerdict::Borderline,
                confidence: 0.72,
                rationale: format!("signal '{}' has borderline fidelity hint", signal_id),
                evidence_refs: vec![signal_id.clone()],
            },
            MindCommand::ObserveSignal {
                signal_id,
                fidelity_hint: Some(_),
                ..
            } => Judgment {
                criterion: EvaluationCriterion::SignalFaithfulness,
                verdict: EvaluationVerdict::Pass,
                confidence: 0.70,
                rationale: format!("signal '{}' appears faithful", signal_id),
                evidence_refs: vec![signal_id.clone()],
            },
            _ => Judgment {
                criterion: EvaluationCriterion::SignalFaithfulness,
                verdict: EvaluationVerdict::Unknown,
                confidence: 0.40,
                rationale: "no signal evidence in this cycle".to_string(),
                evidence_refs: vec!["signal-buffer".to_string()],
            },
        };

        let mut judgments = vec![alignment, reliability, faithfulness];
        for judgment in &mut judgments {
            judgment.confidence = clamp_confidence(judgment.confidence);
            if !matches!(judgment.verdict, EvaluationVerdict::Pass)
                && judgment.rationale.trim().is_empty()
            {
                judgment.rationale = "verdict rationale missing".to_string();
            }
        }

        Ok(EvaluationReport { goal_id, judgments })
    }
}
