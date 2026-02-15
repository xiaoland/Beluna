use std::sync::Arc;

use async_trait::async_trait;
use tokio::time::{Duration, timeout};

use crate::{
    cortex::{
        clamp::derive_act_id,
        error::{CortexError, budget_exceeded, cycle_timeout, invalid_input},
        ports::{
            AttemptExtractorPort, AttemptExtractorRequest, CortexPort, CortexTelemetryEvent,
            CortexTelemetryPort, PrimaryReasonerPort, PrimaryReasonerRequest,
        },
        types::{CortexOutput, ReactionLimits},
    },
    runtime_types::{Act, CognitionState, GoalFrame, PhysicalState, RequestedResources, Sense},
};

pub struct CortexPipeline {
    primary: Arc<dyn PrimaryReasonerPort>,
    extractor: Arc<dyn AttemptExtractorPort>,
    telemetry: Arc<dyn CortexTelemetryPort>,
    limits: ReactionLimits,
}

impl CortexPipeline {
    pub fn new(
        primary: Arc<dyn PrimaryReasonerPort>,
        extractor: Arc<dyn AttemptExtractorPort>,
        telemetry: Arc<dyn CortexTelemetryPort>,
        limits: ReactionLimits,
    ) -> Self {
        Self {
            primary,
            extractor,
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
                eprintln!(
                    "[cortex] primary_timeout cycle_id={} deadline_ms={} max_cycle_time_ms={}",
                    physical_state.cycle_id,
                    deadline.as_millis(),
                    self.limits.max_cycle_time_ms
                );
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
                eprintln!(
                    "[cortex] extractor_timeout cycle_id={} deadline_ms={} max_cycle_time_ms={}",
                    physical_state.cycle_id,
                    deadline.as_millis(),
                    self.limits.max_cycle_time_ms
                );
                return Err(cycle_timeout("extractor call timed out"));
            }
        };

        let acts = drafts_to_acts(physical_state.cycle_id, drafts, &self.limits);
        eprintln!(
            "[cortex] drafts_to_acts cycle_id={} act_count={}",
            physical_state.cycle_id,
            acts.len()
        );
        if acts.is_empty() {
            self.telemetry.on_event(CortexTelemetryEvent::NoopFallback {
                cycle_id: physical_state.cycle_id,
                reason: "extractor_no_drafts",
            });
        }

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
    max_primary_calls: u8,
    max_sub_calls: u8,
}

impl CycleBudgetGuard {
    fn new(limits: &ReactionLimits) -> Self {
        Self {
            primary_calls: 0,
            sub_calls: 0,
            max_primary_calls: limits.max_primary_calls,
            max_sub_calls: limits.max_sub_calls,
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
}

fn validate_input_bounds(limits: &ReactionLimits) -> Result<(), CortexError> {
    if limits.max_primary_calls != 1 {
        return Err(invalid_input("max_primary_calls must be exactly 1"));
    }
    Ok(())
}

fn drafts_to_acts(
    cycle_id: u64,
    drafts: Vec<crate::cortex::types::AttemptDraft>,
    limits: &ReactionLimits,
) -> Vec<Act> {
    let mut acts = Vec::with_capacity(drafts.len());
    for draft in drafts {
        let crate::cortex::types::AttemptDraft {
            based_on,
            endpoint_id,
            capability_id,
            capability_instance_id,
            payload_draft,
            requested_resources,
            ..
        } = draft;

        let requested_resources = clamp_resources(requested_resources);
        let act_id = derive_act_id(
            cycle_id,
            &based_on,
            &endpoint_id,
            &capability_id,
            &payload_draft,
            &requested_resources,
        );
        let capability_instance_id = if capability_instance_id.trim().is_empty() {
            act_id.clone()
        } else {
            capability_instance_id
        };

        acts.push(Act {
            act_id,
            based_on,
            body_endpoint_name: endpoint_id,
            capability_id,
            capability_instance_id,
            normalized_payload: payload_draft,
            requested_resources,
        });
    }

    acts.sort_by(|lhs, rhs| lhs.act_id.cmp(&rhs.act_id));
    acts.truncate(limits.max_attempts);
    acts
}

fn clamp_resources(resources: RequestedResources) -> RequestedResources {
    RequestedResources {
        survival_micro: resources.survival_micro.max(0),
        time_ms: resources.time_ms,
        io_units: resources.io_units,
        token_units: resources.token_units,
    }
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
