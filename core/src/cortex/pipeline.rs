use std::sync::Arc;

use async_trait::async_trait;
use tokio::time::{Duration, timeout};

use crate::{
    cortex::{
        error::{CortexError, budget_exceeded, cycle_timeout, invalid_input},
        ports::{
            AttemptClampPort, AttemptClampRequest, AttemptExtractorPort, AttemptExtractorRequest,
            CortexPort, CortexTelemetryEvent, CortexTelemetryPort, PayloadFillerPort,
            PayloadFillerRequest, PrimaryReasonerPort, PrimaryReasonerRequest,
        },
        types::{CortexOutput, ReactionLimits},
    },
    runtime_types::{CognitionState, GoalFrame, PhysicalState, Sense},
};

pub struct CortexPipeline {
    primary: Arc<dyn PrimaryReasonerPort>,
    extractor: Arc<dyn AttemptExtractorPort>,
    filler: Arc<dyn PayloadFillerPort>,
    clamp: Arc<dyn AttemptClampPort>,
    telemetry: Arc<dyn CortexTelemetryPort>,
    limits: ReactionLimits,
}

impl CortexPipeline {
    pub fn new(
        primary: Arc<dyn PrimaryReasonerPort>,
        extractor: Arc<dyn AttemptExtractorPort>,
        filler: Arc<dyn PayloadFillerPort>,
        clamp: Arc<dyn AttemptClampPort>,
        telemetry: Arc<dyn CortexTelemetryPort>,
        limits: ReactionLimits,
    ) -> Self {
        Self {
            primary,
            extractor,
            filler,
            clamp,
            telemetry,
            limits,
        }
    }
}

#[async_trait]
impl CortexPort for CortexPipeline {
    async fn cortex(
        &self,
        senses: &[Sense],
        physical_state: &PhysicalState,
        cognition_state: &CognitionState,
    ) -> Result<CortexOutput, CortexError> {
        if senses.is_empty() {
            return Err(invalid_input("sense batch cannot be empty"));
        }
        if senses.iter().any(|sense| matches!(sense, Sense::Sleep)) {
            return Err(invalid_input("sleep sense should not be sent to cortex"));
        }

        self.telemetry
            .on_event(CortexTelemetryEvent::ReactionStarted {
                cycle_id: physical_state.cycle_id,
            });

        if let Err(err) = validate_input_bounds(&self.limits) {
            self.telemetry.on_event(CortexTelemetryEvent::StageFailed {
                cycle_id: physical_state.cycle_id,
                stage: "input_validation",
            });
            return Err(err);
        }

        let known_sense_ids = known_sense_ids(senses);
        let deadline = Duration::from_millis(self.limits.max_cycle_time_ms.max(1));
        let mut budget = CycleBudgetGuard::new(&self.limits);

        if let Err(err) = budget.record_primary_call() {
            return Err(err);
        }

        let primary_req = PrimaryReasonerRequest {
            cycle_id: physical_state.cycle_id,
            senses: senses.to_vec(),
            physical_state: physical_state.clone(),
            cognition_state: cognition_state.clone(),
            limits: self.limits.clone(),
        };
        let ir = match timeout(deadline, self.primary.infer_ir(primary_req)).await {
            Ok(Ok(ir)) => ir,
            Ok(Err(err)) => {
                self.telemetry.on_event(CortexTelemetryEvent::StageFailed {
                    cycle_id: physical_state.cycle_id,
                    stage: "primary",
                });
                return Err(err);
            }
            Err(_) => {
                self.telemetry.on_event(CortexTelemetryEvent::StageFailed {
                    cycle_id: physical_state.cycle_id,
                    stage: "primary_timeout",
                });
                return Err(cycle_timeout("primary call timed out"));
            }
        };

        if let Err(err) = budget.record_sub_call() {
            return Err(err);
        }

        let extract_req = AttemptExtractorRequest {
            cycle_id: physical_state.cycle_id,
            prose_ir: ir,
            capability_catalog: physical_state.capabilities.clone(),
            senses: senses.to_vec(),
            cognition_state: cognition_state.clone(),
            limits: self.limits.clone(),
        };
        let drafts = match timeout(deadline, self.extractor.extract(extract_req)).await {
            Ok(Ok(drafts)) => drafts,
            Ok(Err(err)) => {
                self.telemetry.on_event(CortexTelemetryEvent::StageFailed {
                    cycle_id: physical_state.cycle_id,
                    stage: "extractor",
                });
                return Err(err);
            }
            Err(_) => {
                self.telemetry.on_event(CortexTelemetryEvent::StageFailed {
                    cycle_id: physical_state.cycle_id,
                    stage: "extractor_timeout",
                });
                return Err(cycle_timeout("extractor call timed out"));
            }
        };

        let clamped_first = self.clamp.clamp(AttemptClampRequest {
            cycle_id: physical_state.cycle_id,
            drafts: drafts.clone(),
            capability_catalog: physical_state.capabilities.clone(),
            known_sense_ids: known_sense_ids.clone(),
            limits: self.limits.clone(),
        })?;

        let acts = if !clamped_first.acts.is_empty() {
            clamped_first.acts
        } else if self.limits.max_repair_attempts == 0 || !budget.can_attempt_repair() {
            self.telemetry.on_event(CortexTelemetryEvent::NoopFallback {
                cycle_id: physical_state.cycle_id,
                reason: "no_repair_budget",
            });
            Vec::new()
        } else {
            budget.record_repair_call()?;
            let fill_req = PayloadFillerRequest {
                cycle_id: physical_state.cycle_id,
                drafts,
                capability_catalog: physical_state.capabilities.clone(),
                clamp_violations: clamped_first.violations,
                limits: self.limits.clone(),
            };
            let repaired = match timeout(deadline, self.filler.fill(fill_req)).await {
                Ok(Ok(repaired)) => repaired,
                Ok(Err(err)) => {
                    self.telemetry.on_event(CortexTelemetryEvent::StageFailed {
                        cycle_id: physical_state.cycle_id,
                        stage: "filler",
                    });
                    return Err(err);
                }
                Err(_) => {
                    self.telemetry.on_event(CortexTelemetryEvent::StageFailed {
                        cycle_id: physical_state.cycle_id,
                        stage: "filler_timeout",
                    });
                    return Err(cycle_timeout("filler call timed out"));
                }
            };

            self.clamp
                .clamp(AttemptClampRequest {
                    cycle_id: physical_state.cycle_id,
                    drafts: repaired,
                    capability_catalog: physical_state.capabilities.clone(),
                    known_sense_ids,
                    limits: self.limits.clone(),
                })?
                .acts
        };

        let new_cognition_state = evolve_cognition_state(cognition_state, senses);
        self.telemetry
            .on_event(CortexTelemetryEvent::ReactionCompleted {
                cycle_id: physical_state.cycle_id,
                act_count: acts.len(),
            });

        Ok(CortexOutput {
            acts,
            new_cognition_state,
        })
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
    fn new(limits: &ReactionLimits) -> Self {
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

fn validate_input_bounds(limits: &ReactionLimits) -> Result<(), CortexError> {
    if limits.max_primary_calls != 1 {
        return Err(invalid_input("max_primary_calls must be exactly 1"));
    }
    if limits.max_repair_attempts > 1 {
        return Err(invalid_input("max_repair_attempts must be <= 1"));
    }
    Ok(())
}

fn known_sense_ids(senses: &[Sense]) -> Vec<String> {
    let mut known = std::collections::BTreeSet::new();
    for (index, sense) in senses.iter().enumerate() {
        match sense {
            Sense::Domain(item) => {
                known.insert(item.sense_id.clone());
            }
            Sense::NewCapabilities(_) => {
                known.insert(format!("control:new_capabilities:{index}"));
            }
            Sense::DropCapabilities(_) => {
                known.insert(format!("control:drop_capabilities:{index}"));
            }
            Sense::Sleep => {
                known.insert(format!("control:sleep:{index}"));
            }
        }
    }
    known.into_iter().collect()
}

fn evolve_cognition_state(previous: &CognitionState, senses: &[Sense]) -> CognitionState {
    let mut next = previous.clone();
    next.revision = next.revision.saturating_add(1);

    for sense in senses {
        if let Sense::Domain(datum) = sense {
            if let Some(goal_push) = datum.payload.get("goal_push") {
                let goal_id = goal_push
                    .get("goal_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("goal:auto")
                    .to_string();
                let summary = goal_push
                    .get("summary")
                    .and_then(|v| v.as_str())
                    .unwrap_or("auto goal")
                    .to_string();
                next.goal_stack.push(GoalFrame { goal_id, summary });
            }

            if datum
                .payload
                .get("goal_pop")
                .and_then(|value| value.as_bool())
                .unwrap_or(false)
            {
                next.goal_stack.pop();
            }
        }
    }

    next
}
