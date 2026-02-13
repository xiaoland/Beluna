use std::{collections::BTreeSet, sync::Arc, time::Duration};

use tokio::time::timeout;

use crate::cortex::{
    error::{CortexError, budget_exceeded, cycle_timeout, invalid_input},
    ports::{
        AttemptClampPort, AttemptClampRequest, AttemptExtractorPort, AttemptExtractorRequest,
        CortexTelemetryEvent, CortexTelemetryPort, PayloadFillerPort, PayloadFillerRequest,
        PrimaryReasonerPort, PrimaryReasonerRequest,
    },
    types::{ReactionInput, ReactionResult},
};

pub struct CortexPipeline {
    primary: Arc<dyn PrimaryReasonerPort>,
    extractor: Arc<dyn AttemptExtractorPort>,
    filler: Arc<dyn PayloadFillerPort>,
    clamp: Arc<dyn AttemptClampPort>,
    telemetry: Arc<dyn CortexTelemetryPort>,
}

impl CortexPipeline {
    pub fn new(
        primary: Arc<dyn PrimaryReasonerPort>,
        extractor: Arc<dyn AttemptExtractorPort>,
        filler: Arc<dyn PayloadFillerPort>,
        clamp: Arc<dyn AttemptClampPort>,
        telemetry: Arc<dyn CortexTelemetryPort>,
    ) -> Self {
        Self {
            primary,
            extractor,
            filler,
            clamp,
            telemetry,
        }
    }

    pub async fn react_once(&self, input: ReactionInput) -> ReactionResult {
        self.telemetry
            .on_event(CortexTelemetryEvent::ReactionStarted {
                reaction_id: input.reaction_id,
            });

        if let Err(err) = validate_input_bounds(&input) {
            self.telemetry.on_event(CortexTelemetryEvent::StageFailed {
                reaction_id: input.reaction_id,
                stage: "input_validation",
            });
            return noop_result(&input, "invalid_input", Some(err), &*self.telemetry);
        }

        let mut budget = CycleBudgetGuard::new(&input.limits);
        let deadline = Duration::from_millis(input.limits.max_cycle_time_ms.max(1));

        let primary_req = PrimaryReasonerRequest {
            reaction_id: input.reaction_id,
            prompt_context: build_primary_prompt_context(&input),
            sense_window: input.sense_window.clone(),
            limits: input.limits.clone(),
        };
        if let Err(err) = budget.record_primary_call() {
            return noop_result(
                &input,
                "primary_budget_exceeded",
                Some(err),
                &*self.telemetry,
            );
        }
        let ir = match timeout(deadline, self.primary.infer_ir(primary_req)).await {
            Ok(Ok(ir)) => ir,
            Ok(Err(err)) => {
                self.telemetry.on_event(CortexTelemetryEvent::StageFailed {
                    reaction_id: input.reaction_id,
                    stage: "primary",
                });
                return noop_result(&input, "primary_failed", Some(err), &*self.telemetry);
            }
            Err(_) => {
                self.telemetry.on_event(CortexTelemetryEvent::StageFailed {
                    reaction_id: input.reaction_id,
                    stage: "primary_timeout",
                });
                return noop_result(
                    &input,
                    "primary_timeout",
                    Some(cycle_timeout("primary call timed out")),
                    &*self.telemetry,
                );
            }
        };

        if let Err(err) = budget.record_sub_call() {
            return noop_result(&input, "sub_budget_exceeded", Some(err), &*self.telemetry);
        }

        let extract_req = AttemptExtractorRequest {
            reaction_id: input.reaction_id,
            prose_ir: ir,
            capability_catalog: input.capability_catalog.clone(),
            sense_window: input.sense_window.clone(),
            limits: input.limits.clone(),
        };
        let drafts = match timeout(deadline, self.extractor.extract(extract_req)).await {
            Ok(Ok(drafts)) => drafts,
            Ok(Err(err)) => {
                self.telemetry.on_event(CortexTelemetryEvent::StageFailed {
                    reaction_id: input.reaction_id,
                    stage: "extractor",
                });
                return noop_result(&input, "extractor_failed", Some(err), &*self.telemetry);
            }
            Err(_) => {
                self.telemetry.on_event(CortexTelemetryEvent::StageFailed {
                    reaction_id: input.reaction_id,
                    stage: "extractor_timeout",
                });
                return noop_result(
                    &input,
                    "extractor_timeout",
                    Some(cycle_timeout("extractor call timed out")),
                    &*self.telemetry,
                );
            }
        };

        let clamped_1 = match self.clamp.clamp(AttemptClampRequest {
            reaction_id: input.reaction_id,
            drafts: drafts.clone(),
            capability_catalog: input.capability_catalog.clone(),
            sense_window: input.sense_window.clone(),
            limits: input.limits.clone(),
        }) {
            Ok(result) => result,
            Err(err) => return noop_result(&input, "clamp_failed", Some(err), &*self.telemetry),
        };

        if !clamped_1.attempts.is_empty() {
            let result = ReactionResult {
                reaction_id: input.reaction_id,
                based_on: clamped_1.based_on,
                attention_tags: clamped_1.attention_tags,
                attempts: clamped_1.attempts,
            };
            self.telemetry
                .on_event(CortexTelemetryEvent::ReactionCompleted {
                    reaction_id: input.reaction_id,
                    attempt_count: result.attempts.len(),
                });
            return result;
        }

        let can_repair = input.limits.max_repair_attempts > 0 && budget.can_attempt_repair();
        if !can_repair {
            return noop_result(&input, "no_repair_budget", None, &*self.telemetry);
        }

        if let Err(err) = budget.record_repair_call() {
            return noop_result(
                &input,
                "repair_budget_exceeded",
                Some(err),
                &*self.telemetry,
            );
        }

        let fill_req = PayloadFillerRequest {
            reaction_id: input.reaction_id,
            drafts: drafts.clone(),
            capability_catalog: input.capability_catalog.clone(),
            clamp_violations: clamped_1.violations,
            limits: input.limits.clone(),
        };
        let repaired = match timeout(deadline, self.filler.fill(fill_req)).await {
            Ok(Ok(repaired)) => repaired,
            Ok(Err(err)) => {
                self.telemetry.on_event(CortexTelemetryEvent::StageFailed {
                    reaction_id: input.reaction_id,
                    stage: "filler",
                });
                return noop_result(&input, "filler_failed", Some(err), &*self.telemetry);
            }
            Err(_) => {
                self.telemetry.on_event(CortexTelemetryEvent::StageFailed {
                    reaction_id: input.reaction_id,
                    stage: "filler_timeout",
                });
                return noop_result(
                    &input,
                    "filler_timeout",
                    Some(cycle_timeout("filler call timed out")),
                    &*self.telemetry,
                );
            }
        };

        let clamped_2 = match self.clamp.clamp(AttemptClampRequest {
            reaction_id: input.reaction_id,
            drafts: repaired,
            capability_catalog: input.capability_catalog.clone(),
            sense_window: input.sense_window.clone(),
            limits: input.limits.clone(),
        }) {
            Ok(result) => result,
            Err(err) => return noop_result(&input, "clamp_failed", Some(err), &*self.telemetry),
        };

        if clamped_2.attempts.is_empty() {
            return noop_result(&input, "repaired_clamp_empty", None, &*self.telemetry);
        }

        let result = ReactionResult {
            reaction_id: input.reaction_id,
            based_on: clamped_2.based_on,
            attention_tags: clamped_2.attention_tags,
            attempts: clamped_2.attempts,
        };
        self.telemetry
            .on_event(CortexTelemetryEvent::ReactionCompleted {
                reaction_id: input.reaction_id,
                attempt_count: result.attempts.len(),
            });
        result
    }
}

struct CycleBudgetGuard {
    primary_calls: u8,
    sub_calls: u8,
    repair_calls: u8,
    max_primary_calls: u8,
    max_sub_calls: u8,
    max_repair_attempts: u8,
}

impl CycleBudgetGuard {
    fn new(limits: &crate::cortex::types::ReactionLimits) -> Self {
        Self {
            primary_calls: 0,
            sub_calls: 0,
            repair_calls: 0,
            max_primary_calls: limits.max_primary_calls,
            max_sub_calls: limits.max_sub_calls,
            max_repair_attempts: limits.max_repair_attempts,
        }
    }

    fn record_primary_call(&mut self) -> Result<(), CortexError> {
        if self.primary_calls >= self.max_primary_calls {
            return Err(budget_exceeded("primary call budget exceeded"));
        }
        self.primary_calls = self.primary_calls.saturating_add(1);
        Ok(())
    }

    fn record_sub_call(&mut self) -> Result<(), CortexError> {
        if self.sub_calls >= self.max_sub_calls {
            return Err(budget_exceeded("sub-call budget exceeded"));
        }
        self.sub_calls = self.sub_calls.saturating_add(1);
        Ok(())
    }

    fn can_attempt_repair(&self) -> bool {
        self.repair_calls < self.max_repair_attempts && self.sub_calls < self.max_sub_calls
    }

    fn record_repair_call(&mut self) -> Result<(), CortexError> {
        if self.repair_calls >= self.max_repair_attempts {
            return Err(budget_exceeded("repair budget exceeded"));
        }
        self.repair_calls = self.repair_calls.saturating_add(1);
        self.record_sub_call()
    }
}

fn validate_input_bounds(input: &ReactionInput) -> Result<(), CortexError> {
    if input.limits.max_primary_calls != 1 {
        return Err(invalid_input("max_primary_calls must be exactly 1"));
    }
    if input.limits.max_repair_attempts > 1 {
        return Err(invalid_input("max_repair_attempts must be <= 1"));
    }
    if input.sense_window.len() > input.limits.max_sense_items {
        return Err(invalid_input("sense window exceeds max_sense_items"));
    }
    if input.env_snapshots.len() > input.limits.max_snapshot_items {
        return Err(invalid_input("env snapshots exceed max_snapshot_items"));
    }
    if input
        .env_snapshots
        .iter()
        .any(|snap| snap.blob_bytes > input.limits.max_snapshot_bytes_per_item)
    {
        return Err(invalid_input(
            "env snapshot blob exceeds max_snapshot_bytes_per_item",
        ));
    }
    let mut seen = BTreeSet::new();
    for sense in &input.sense_window {
        if !seen.insert(sense.sense_id.clone()) {
            return Err(invalid_input("duplicate sense_id in sense window"));
        }
    }
    Ok(())
}

fn build_primary_prompt_context(input: &ReactionInput) -> String {
    let constitutional = input
        .context
        .constitutional
        .iter()
        .map(|item| item.intent_key.clone())
        .collect::<Vec<_>>();
    let environmental = input
        .context
        .environmental
        .iter()
        .map(|item| item.signal_key.clone())
        .collect::<Vec<_>>();
    let emergent = input
        .context
        .emergent_candidates
        .iter()
        .map(|item| item.candidate_key.clone())
        .collect::<Vec<_>>();

    serde_json::json!({
        "reaction_id": input.reaction_id,
        "constitutional": constitutional,
        "environmental": environmental,
        "emergent": emergent,
        "senses": input.sense_window.iter().map(|sense| &sense.sense_id).collect::<Vec<_>>(),
        "feedback_codes": input.admission_feedback.iter().map(|signal| &signal.code).collect::<Vec<_>>(),
    })
    .to_string()
}

fn noop_result(
    input: &ReactionInput,
    reason: &'static str,
    err: Option<CortexError>,
    telemetry: &dyn CortexTelemetryPort,
) -> ReactionResult {
    if err.is_some() {
        telemetry.on_event(CortexTelemetryEvent::StageFailed {
            reaction_id: input.reaction_id,
            stage: reason,
        });
    }
    telemetry.on_event(CortexTelemetryEvent::NoopFallback {
        reaction_id: input.reaction_id,
        reason,
    });
    let based_on = input
        .sense_window
        .iter()
        .map(|sense| sense.sense_id.clone())
        .collect();
    ReactionResult {
        reaction_id: input.reaction_id,
        based_on,
        attention_tags: Vec::new(),
        attempts: Vec::new(),
    }
}
