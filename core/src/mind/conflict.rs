use crate::mind::{
    error::{MindError, conflict_resolution_error},
    types::{ConflictCase, ConflictResolution, EvaluationCriterion, EvaluationVerdict, Judgment},
};

pub trait ConflictResolver: Send + Sync {
    fn resolve(&self, cases: &[ConflictCase]) -> Result<Vec<ConflictResolution>, MindError>;
}

#[derive(Debug, Clone, Default)]
pub struct DeterministicConflictResolver;

impl ConflictResolver for DeterministicConflictResolver {
    fn resolve(&self, cases: &[ConflictCase]) -> Result<Vec<ConflictResolution>, MindError> {
        let mut ordered: Vec<ConflictCase> = cases.to_vec();
        ordered.sort_by(|lhs, rhs| conflict_case_key(lhs).cmp(&conflict_case_key(rhs)));

        let mut resolutions = Vec::with_capacity(ordered.len());
        for case in ordered {
            let resolution = match case {
                ConflictCase::HelperOutputSameIntent {
                    intent_id,
                    mut candidates,
                } => {
                    if candidates.is_empty() {
                        return Err(conflict_resolution_error(
                            "helper conflict has no candidates to resolve",
                        ));
                    }
                    candidates.sort_by(|a, b| {
                        b.confidence
                            .total_cmp(&a.confidence)
                            .then_with(|| a.helper_id.cmp(&b.helper_id))
                    });
                    let selected = candidates.remove(0);
                    ConflictResolution::SelectedHelperResult {
                        intent_id,
                        helper_id: selected.helper_id,
                    }
                }
                ConflictCase::EvaluatorVerdictSameCriterion {
                    criterion,
                    mut candidates,
                } => {
                    if candidates.is_empty() {
                        return Err(conflict_resolution_error(
                            "evaluation conflict has no candidates to resolve",
                        ));
                    }
                    candidates.sort_by(|a, b| compare_judgments(a, b));
                    let selected = candidates.remove(0);
                    ConflictResolution::SelectedJudgment {
                        criterion,
                        verdict: selected.verdict,
                    }
                }
                ConflictCase::MergeCompatibility {
                    active_goal_id,
                    incoming_goal_id,
                    compatible,
                } => {
                    if compatible {
                        ConflictResolution::MergeAllowed {
                            merged_goal_id: crate::mind::types::merged_goal_id(
                                &active_goal_id,
                                &incoming_goal_id,
                            ),
                        }
                    } else {
                        ConflictResolution::MergeRejected
                    }
                }
            };
            resolutions.push(resolution);
        }

        if resolutions.is_empty() {
            return Ok(vec![ConflictResolution::NoConflict]);
        }

        Ok(resolutions)
    }
}

fn conflict_case_key(case: &ConflictCase) -> (u8, String) {
    match case {
        ConflictCase::HelperOutputSameIntent { intent_id, .. } => (0, intent_id.clone()),
        ConflictCase::EvaluatorVerdictSameCriterion { criterion, .. } => {
            (1, format!("{:?}", criterion))
        }
        ConflictCase::MergeCompatibility {
            active_goal_id,
            incoming_goal_id,
            ..
        } => (2, format!("{}:{}", active_goal_id, incoming_goal_id)),
    }
}

fn compare_judgments(lhs: &Judgment, rhs: &Judgment) -> std::cmp::Ordering {
    verdict_rank(rhs.verdict)
        .cmp(&verdict_rank(lhs.verdict))
        .then_with(|| rhs.confidence.total_cmp(&lhs.confidence))
        .then_with(|| lhs.rationale.cmp(&rhs.rationale))
}

fn verdict_rank(verdict: EvaluationVerdict) -> u8 {
    match verdict {
        EvaluationVerdict::Fail => 4,
        EvaluationVerdict::Borderline => 3,
        EvaluationVerdict::Unknown => 2,
        EvaluationVerdict::Pass => 1,
    }
}

#[allow(dead_code)]
fn _criterion_order(criterion: EvaluationCriterion) -> u8 {
    match criterion {
        EvaluationCriterion::GoalAlignment => 0,
        EvaluationCriterion::SubsystemReliability => 1,
        EvaluationCriterion::SignalFaithfulness => 2,
    }
}
