// cortex/runtime.ts invariants:
// - Should stops at sense, neural signal descriptor, goal tree, l1 memory, IR, act level.

use std::{collections::BTreeMap, sync::Arc, time::Instant};

use async_trait::async_trait;
use tokio::time::{Duration, timeout};

use crate::{
    ai_gateway::{
        gateway::AIGateway,
        types_chat::{
            BelunaContentPart, BelunaMessage, BelunaRole, ChatRequest, ChatResponse, OutputMode,
            RequestLimitOverrides, ToolChoice,
        },
    },
    config::CortexHelperRoutesConfig,
    cortex::{
        cognition::{CognitionState, GoalTreePatchOp},
        cognition_patch::apply_cognition_patches,
        error::{CortexError, extractor_failed, filler_failed, invalid_input, primary_failed},
        helpers::{
            self, CognitionOrgan, CortexHelpers, HelperRuntime, act_descriptor_input_helper,
            goal_tree_input_helper, sense_input_helper,
        },
        ir, prompts,
        testing::{PrimaryRequest as TestPrimaryRequest, TestHooks},
        types::{CortexOutput, ReactionLimits},
    },
    observability::metrics as observability_metrics,
    types::{PhysicalState, Sense},
};

#[derive(Debug, Clone)]
pub enum CortexTelemetryEvent {
    ReactionStarted { cycle_id: u64 },
    StageFailed { cycle_id: u64, stage: &'static str },
    ReactionCompleted { cycle_id: u64, act_count: usize },
    NoopFallback { cycle_id: u64, reason: &'static str },
}

pub type CortexTelemetryHook = Arc<dyn Fn(CortexTelemetryEvent) + Send + Sync>;

#[derive(Clone)]
pub struct Cortex {
    gateway: Option<Arc<AIGateway>>,
    helper_routes: CortexHelperRoutesConfig,
    hooks: Option<TestHooks>,
    helpers: CortexHelpers,
    telemetry_hook: Option<CortexTelemetryHook>,
    limits: ReactionLimits,
}

impl Cortex {
    pub fn from_config(
        config: &crate::config::CortexRuntimeConfig,
        gateway: Arc<AIGateway>,
        telemetry_hook: Option<CortexTelemetryHook>,
    ) -> Self {
        Self {
            gateway: Some(gateway),
            helper_routes: config.helper_routes.clone(),
            hooks: None,
            helpers: CortexHelpers::default(),
            telemetry_hook,
            limits: config.default_limits.clone(),
        }
    }

    pub(crate) fn for_test_with_hooks(hooks: TestHooks, limits: ReactionLimits) -> Self {
        Self {
            gateway: None,
            helper_routes: CortexHelperRoutesConfig::default(),
            hooks: Some(hooks),
            helpers: CortexHelpers::default(),
            telemetry_hook: None,
            limits,
        }
    }

    pub async fn cortex(
        &self,
        senses: &[Sense],
        physical_state: &PhysicalState,
        cognition_state: &CognitionState,
    ) -> Result<CortexOutput, CortexError> {
        if senses.iter().any(|sense| matches!(sense, Sense::Hibernate)) {
            return Err(invalid_input(
                "hibernate sense should not be sent to cortex",
            ));
        }

        self.emit(CortexTelemetryEvent::ReactionStarted {
            cycle_id: physical_state.cycle_id,
        });
        observability_metrics::record_cortex_cycle_id(physical_state.cycle_id);

        let deadline = Duration::from_millis(self.limits.max_cycle_time_ms.max(1));
        let sense_descriptors = helpers::sense_descriptors(&physical_state.capabilities.entries);
        let act_descriptors = helpers::act_descriptors(&physical_state.capabilities.entries);

        let senses_owned = senses.to_vec();
        let sense_descriptors_for_helper = sense_descriptors.clone();
        let act_descriptors_for_helper = act_descriptors.clone();
        let goal_tree = cognition_state.goal_tree.clone();

        let (sense_section_result, act_catalog_result, goal_tree_sections_result) = tokio::join!(
            timeout(
                deadline,
                self.helpers.input.sense.to_input_ir_section(
                    self,
                    physical_state.cycle_id,
                    &senses_owned,
                    &sense_descriptors_for_helper,
                )
            ),
            timeout(
                deadline,
                self.helpers.input.act_descriptor.to_input_ir_section(
                    self,
                    physical_state.cycle_id,
                    &act_descriptors_for_helper,
                )
            ),
            timeout(
                deadline,
                self.helpers.input.goal_tree.to_input_ir_sections(
                    self,
                    physical_state.cycle_id,
                    &goal_tree,
                )
            )
        );

        let senses_section = self.resolve_input_helper_fallback(
            physical_state.cycle_id,
            "sense_helper",
            sense_section_result,
            sense_input_helper::fallback_senses_section(senses, &sense_descriptors),
        );
        let act_descriptor_catalog_section = self.resolve_input_helper_fallback(
            physical_state.cycle_id,
            "act_descriptor_helper",
            act_catalog_result,
            act_descriptor_input_helper::fallback_act_descriptor_catalog_section(&act_descriptors),
        );
        let goal_tree_sections = self.resolve_goal_tree_input_helper_fallback(
            physical_state.cycle_id,
            goal_tree_sections_result,
            goal_tree_input_helper::fallback_input_ir_sections(&cognition_state.goal_tree),
        );
        let instincts_section = goal_tree_sections.instincts_section;
        let willpower_matrix_section = goal_tree_sections.willpower_matrix_section;
        let focal_awareness_section =
            goal_tree_input_helper::l1_memory_section(&cognition_state.l1_memory);

        observability_metrics::record_cortex_input_ir_act_descriptor_catalog_count(
            act_descriptors.len(),
        );
        tracing::debug!(
            target: "cortex",
            cycle_id = physical_state.cycle_id,
            input_ir_sense = %senses_section,
            "input_ir_sense"
        );
        tracing::debug!(
            target: "cortex",
            cycle_id = physical_state.cycle_id,
            input_ir_act_descriptor_catalog = %act_descriptor_catalog_section,
            "input_ir_act_descriptor_catalog"
        );
        tracing::debug!(
            target: "cortex",
            cycle_id = physical_state.cycle_id,
            input_ir_instincts = %instincts_section,
            "input_ir_instincts"
        );
        tracing::debug!(
            target: "cortex",
            cycle_id = physical_state.cycle_id,
            input_ir_willpower_matrix = %willpower_matrix_section,
            "input_ir_willpower_matrix"
        );
        tracing::debug!(
            target: "cortex",
            cycle_id = physical_state.cycle_id,
            input_ir_focal_awareness = %focal_awareness_section,
            "input_ir_focal_awareness"
        );

        let input_ir = ir::build_input_ir(
            &senses_section,
            &act_descriptor_catalog_section,
            &instincts_section,
            &willpower_matrix_section,
            &focal_awareness_section,
        );
        let primary_input_payload = ir::build_primary_input_payload(
            &senses_section,
            &act_descriptor_catalog_section,
            &instincts_section,
            &willpower_matrix_section,
            &focal_awareness_section,
        );

        let primary_result = timeout(
            deadline,
            self.run_primary_engine(
                physical_state.cycle_id,
                primary_input_payload,
                input_ir.text.clone(),
            ),
        )
        .await;
        let primary_output = match primary_result {
            Ok(Ok(output)) => output,
            Ok(Err(err)) => {
                self.emit(CortexTelemetryEvent::StageFailed {
                    cycle_id: physical_state.cycle_id,
                    stage: "primary",
                });
                tracing::warn!(
                    target: "cortex",
                    cycle_id = physical_state.cycle_id,
                    error = %err,
                    "primary_failed_noop"
                );
                return Ok(self.noop_output(
                    cognition_state,
                    physical_state.cycle_id,
                    "primary_failed",
                ));
            }
            Err(_) => {
                self.emit(CortexTelemetryEvent::StageFailed {
                    cycle_id: physical_state.cycle_id,
                    stage: "primary_timeout",
                });
                tracing::warn!(
                    target: "cortex",
                    cycle_id = physical_state.cycle_id,
                    deadline_ms = deadline.as_millis() as u64,
                    "primary_timeout_noop"
                );
                return Ok(self.noop_output(
                    cognition_state,
                    physical_state.cycle_id,
                    "primary_timeout",
                ));
            }
        };

        let (_output_ir, output_sections) = match ir::parse_output_ir(&primary_output) {
            Ok(parsed) => parsed,
            Err(err) => {
                self.emit(CortexTelemetryEvent::StageFailed {
                    cycle_id: physical_state.cycle_id,
                    stage: "primary_contract",
                });
                tracing::warn!(
                    target: "cortex",
                    cycle_id = physical_state.cycle_id,
                    error = %err,
                    "primary_contract_failed_noop"
                );
                return Ok(self.noop_output(
                    cognition_state,
                    physical_state.cycle_id,
                    "primary_contract",
                ));
            }
        };

        let (acts_result, goal_tree_ops_result, l1_memory_flush_result) = tokio::join!(
            timeout(
                deadline,
                self.helpers.output.acts.to_structured_output(
                    self,
                    physical_state.cycle_id,
                    &output_sections.acts_section,
                    &act_descriptors,
                )
            ),
            timeout(
                deadline,
                self.helpers.output.goal_tree_patch.to_structured_output(
                    self,
                    physical_state.cycle_id,
                    &output_sections.goal_tree_patch_section,
                    cognition_state,
                )
            ),
            timeout(
                deadline,
                self.helpers.output.l1_memory_flush.to_structured_output(
                    self,
                    physical_state.cycle_id,
                    &output_sections.l1_memory_flush_section,
                    cognition_state,
                )
            )
        );

        let acts = match acts_result {
            Ok(Ok(acts)) => acts,
            Ok(Err(err)) => {
                self.emit(CortexTelemetryEvent::StageFailed {
                    cycle_id: physical_state.cycle_id,
                    stage: "acts_helper",
                });
                tracing::warn!(
                    target: "cortex",
                    cycle_id = physical_state.cycle_id,
                    error = %err,
                    "acts_helper_failed_fallback_empty"
                );
                Vec::new()
            }
            Err(_) => {
                self.emit(CortexTelemetryEvent::StageFailed {
                    cycle_id: physical_state.cycle_id,
                    stage: "acts_helper_timeout",
                });
                tracing::warn!(
                    target: "cortex",
                    cycle_id = physical_state.cycle_id,
                    deadline_ms = deadline.as_millis() as u64,
                    "acts_helper_timeout_fallback_empty"
                );
                Vec::new()
            }
        };

        let goal_tree_ops = match goal_tree_ops_result {
            Ok(Ok(ops)) => ops,
            Ok(Err(err)) => {
                self.emit(CortexTelemetryEvent::StageFailed {
                    cycle_id: physical_state.cycle_id,
                    stage: "goal_tree_patch_helper",
                });
                tracing::warn!(
                    target: "cortex",
                    cycle_id = physical_state.cycle_id,
                    error = %err,
                    "goal_tree_patch_helper_failed_fallback_empty"
                );
                Vec::<GoalTreePatchOp>::new()
            }
            Err(_) => {
                self.emit(CortexTelemetryEvent::StageFailed {
                    cycle_id: physical_state.cycle_id,
                    stage: "goal_tree_patch_helper_timeout",
                });
                tracing::warn!(
                    target: "cortex",
                    cycle_id = physical_state.cycle_id,
                    deadline_ms = deadline.as_millis() as u64,
                    "goal_tree_patch_helper_timeout_fallback_empty"
                );
                Vec::<GoalTreePatchOp>::new()
            }
        };

        let l1_memory_flush = match l1_memory_flush_result {
            Ok(Ok(entries)) => entries,
            Ok(Err(err)) => {
                self.emit(CortexTelemetryEvent::StageFailed {
                    cycle_id: physical_state.cycle_id,
                    stage: "l1_memory_flush_helper",
                });
                tracing::warn!(
                    target: "cortex",
                    cycle_id = physical_state.cycle_id,
                    error = %err,
                    "l1_memory_flush_helper_failed_fallback_empty"
                );
                Vec::<String>::new()
            }
            Err(_) => {
                self.emit(CortexTelemetryEvent::StageFailed {
                    cycle_id: physical_state.cycle_id,
                    stage: "l1_memory_flush_helper_timeout",
                });
                tracing::warn!(
                    target: "cortex",
                    cycle_id = physical_state.cycle_id,
                    deadline_ms = deadline.as_millis() as u64,
                    "l1_memory_flush_helper_timeout_fallback_empty"
                );
                Vec::<String>::new()
            }
        };

        if cognition_state.goal_tree.user_partition.is_empty()
            && !goal_tree_ops.is_empty()
            && !goal_tree_ops
                .iter()
                .any(|op| matches!(op, GoalTreePatchOp::Sprout { .. }))
        {
            tracing::warn!(
                target: "cortex",
                cycle_id = physical_state.cycle_id,
                op_count = goal_tree_ops.len(),
                "goal_tree_patch_no_sprout_for_empty_user_partition"
            );
        }

        let apply_result = apply_cognition_patches(
            cognition_state,
            &goal_tree_ops,
            &l1_memory_flush,
            self.limits.max_l1_memory_entries,
        );
        if !goal_tree_ops.is_empty()
            && apply_result.new_cognition_state.goal_tree.user_partition
                == cognition_state.goal_tree.user_partition
        {
            tracing::warn!(
                target: "cortex",
                cycle_id = physical_state.cycle_id,
                op_count = goal_tree_ops.len(),
                "goal_tree_patch_ops_no_effect"
            );
        }
        if apply_result.l1_memory_overflow_count > 0 {
            tracing::warn!(
                target: "cortex",
                cycle_id = physical_state.cycle_id,
                max_l1_memory_entries = self.limits.max_l1_memory_entries,
                overflow_entries = apply_result.l1_memory_overflow_count,
                "l1_memory_flush_overflow_discarded"
            );
        }
        let new_cognition_state = apply_result.new_cognition_state;

        let acts_json = serde_json::to_string(&acts)
            .unwrap_or_else(|err| format!("{{\"serialization_error\":\"{}\"}}", err));
        tracing::debug!(
            target: "cortex",
            cycle_id = physical_state.cycle_id,
            output_ir_goal_tree_patch = %output_sections.goal_tree_patch_section,
            "output_ir_goal_tree_patch"
        );
        tracing::debug!(
            target: "cortex",
            cycle_id = physical_state.cycle_id,
            output_ir_l1_memory_flush = %output_sections.l1_memory_flush_section,
            "output_ir_l1_memory_flush"
        );
        tracing::debug!(
            target: "cortex",
            cycle_id = physical_state.cycle_id,
            output_ir_wait_for_sense = output_sections.wait_for_sense,
            "output_ir_wait_for_sense"
        );
        tracing::debug!(
            target: "cortex",
            cycle_id = physical_state.cycle_id,
            act_count = acts.len(),
            final_returned_acts = %acts_json,
            "final_returned_acts"
        );

        if acts.is_empty() {
            self.emit(CortexTelemetryEvent::NoopFallback {
                cycle_id: physical_state.cycle_id,
                reason: "acts_helper_empty",
            });
        }
        self.emit(CortexTelemetryEvent::ReactionCompleted {
            cycle_id: physical_state.cycle_id,
            act_count: acts.len(),
        });

        Ok(CortexOutput {
            acts,
            new_cognition_state,
            wait_for_sense: output_sections.wait_for_sense,
        })
    }

    async fn run_primary_engine(
        &self,
        cycle_id: u64,
        primary_input: String,
        input_ir_internal: String,
    ) -> Result<String, CortexError> {
        let stage = CognitionOrgan::Primary.stage();
        let input_payload = helpers::pretty_json(&serde_json::json!({
            "primary_input": &primary_input,
            "input_ir_internal": &input_ir_internal,
        }));
        helpers::log_organ_input(cycle_id, stage, &input_payload);

        if let Some(hooks) = &self.hooks {
            let output = (hooks.primary)(TestPrimaryRequest {
                cycle_id,
                input_ir: primary_input.clone(),
            })
            .await?;
            helpers::log_organ_output(cycle_id, stage, &output);
            return Ok(output);
        }

        let output = self
            .run_text_organ_with_system(
                cycle_id,
                CognitionOrgan::Primary,
                self.limits.max_primary_output_tokens,
                prompts::primary_system_prompt(),
                prompts::build_primary_user_prompt(&primary_input),
            )
            .await?;
        helpers::log_organ_output(cycle_id, stage, &output);
        Ok(output)
    }

    async fn run_text_organ_with_system(
        &self,
        cycle_id: u64,
        organ: CognitionOrgan,
        max_output_tokens: u64,
        system_prompt: String,
        user_prompt: String,
    ) -> Result<String, CortexError> {
        let response = self
            .run_organ(
                cycle_id,
                organ,
                max_output_tokens,
                system_prompt,
                user_prompt,
                OutputMode::Text,
            )
            .await?;
        let text = response.output_text.trim().to_string();
        if text.is_empty() {
            return Err(extractor_failed(format!(
                "{} produced empty output",
                organ.stage()
            )));
        }
        Ok(text)
    }

    async fn run_organ(
        &self,
        cycle_id: u64,
        organ: CognitionOrgan,
        max_output_tokens: u64,
        system_prompt: String,
        user_prompt: String,
        output_mode: OutputMode,
    ) -> Result<ChatResponse, CortexError> {
        let stage = organ.stage();
        let request_id = format!("cortex-{stage}-{cycle_id}");
        let started_at = Instant::now();
        let route = self.resolve_route(organ);
        let request = build_request(
            request_id.clone(),
            route.clone(),
            max_output_tokens,
            self.limits.max_cycle_time_ms,
            system_prompt,
            user_prompt,
            stage,
            output_mode,
        );

        let gateway = self.gateway.as_ref().ok_or_else(|| {
            CortexError::new(
                crate::cortex::error::CortexErrorKind::Internal,
                "AI Gateway is not configured for this Cortex instance",
            )
        })?;

        let response = gateway.chat_once(request).await.map_err(|err| {
            tracing::debug!(
                target: "cortex",
                stage = stage,
                cycle_id = cycle_id,
                request_id = %request_id,
                elapsed_ms = started_at.elapsed().as_millis() as u64,
                error_kind = ?err.kind,
                error = %err.message,
                "llm_call_failed"
            );
            match organ {
                CognitionOrgan::Primary => primary_failed(err.to_string()),
                CognitionOrgan::GoalTreePatch | CognitionOrgan::L1MemoryFlush => {
                    filler_failed(err.to_string())
                }
                CognitionOrgan::Sense
                | CognitionOrgan::ActDescriptor
                | CognitionOrgan::GoalTree
                | CognitionOrgan::Acts => extractor_failed(err.to_string()),
            }
        })?;

        Ok(response)
    }

    fn resolve_input_helper_fallback(
        &self,
        cycle_id: u64,
        stage: &'static str,
        result: Result<Result<String, CortexError>, tokio::time::error::Elapsed>,
        fallback: String,
    ) -> String {
        match result {
            Ok(Ok(text)) if !text.trim().is_empty() => text,
            Ok(Ok(_)) => fallback,
            Ok(Err(err)) => {
                self.emit(CortexTelemetryEvent::StageFailed { cycle_id, stage });
                tracing::warn!(
                    target: "cortex",
                    cycle_id = cycle_id,
                    stage = stage,
                    error = %err,
                    "input_helper_failed_fallback_raw"
                );
                fallback
            }
            Err(_) => {
                self.emit(CortexTelemetryEvent::StageFailed { cycle_id, stage });
                tracing::warn!(
                    target: "cortex",
                    cycle_id = cycle_id,
                    stage = stage,
                    "input_helper_timeout_fallback_raw"
                );
                fallback
            }
        }
    }

    fn resolve_goal_tree_input_helper_fallback(
        &self,
        cycle_id: u64,
        result: Result<
            Result<goal_tree_input_helper::GoalTreeInputSections, CortexError>,
            tokio::time::error::Elapsed,
        >,
        fallback: goal_tree_input_helper::GoalTreeInputSections,
    ) -> goal_tree_input_helper::GoalTreeInputSections {
        match result {
            Ok(Ok(sections))
                if !sections.willpower_matrix_section.trim().is_empty()
                    && !sections.instincts_section.trim().is_empty() =>
            {
                sections
            }
            Ok(Ok(_)) => fallback,
            Ok(Err(err)) => {
                self.emit(CortexTelemetryEvent::StageFailed {
                    cycle_id,
                    stage: "goal_tree_helper",
                });
                tracing::warn!(
                    target: "cortex",
                    cycle_id = cycle_id,
                    stage = "goal_tree_helper",
                    error = %err,
                    "input_helper_failed_fallback_raw"
                );
                fallback
            }
            Err(_) => {
                self.emit(CortexTelemetryEvent::StageFailed {
                    cycle_id,
                    stage: "goal_tree_helper",
                });
                tracing::warn!(
                    target: "cortex",
                    cycle_id = cycle_id,
                    stage = "goal_tree_helper",
                    "input_helper_timeout_fallback_raw"
                );
                fallback
            }
        }
    }

    fn noop_output(
        &self,
        cognition_state: &CognitionState,
        cycle_id: u64,
        reason: &'static str,
    ) -> CortexOutput {
        self.emit(CortexTelemetryEvent::NoopFallback { cycle_id, reason });
        CortexOutput {
            acts: Vec::new(),
            new_cognition_state: cognition_state.clone(),
            wait_for_sense: false,
        }
    }

    fn resolve_route(&self, organ: CognitionOrgan) -> Option<String> {
        let stage_route = match organ {
            CognitionOrgan::Primary => self.helper_routes.primary.clone(),
            CognitionOrgan::Sense => self.helper_routes.sense_helper.clone(),
            CognitionOrgan::ActDescriptor => self.helper_routes.act_descriptor_helper.clone(),
            CognitionOrgan::GoalTree => self.helper_routes.goal_tree_helper.clone(),
            CognitionOrgan::Acts => self.helper_routes.acts_helper.clone(),
            CognitionOrgan::GoalTreePatch => self.helper_routes.goal_tree_patch_helper.clone(),
            CognitionOrgan::L1MemoryFlush => self.helper_routes.l1_memory_flush_helper.clone(),
        };
        stage_route.or_else(|| self.helper_routes.default.clone())
    }

    fn emit(&self, event: CortexTelemetryEvent) {
        match &event {
            CortexTelemetryEvent::ReactionStarted { cycle_id } => {
                tracing::debug!(target: "cortex", cycle_id = *cycle_id, "reaction_started");
            }
            CortexTelemetryEvent::StageFailed { cycle_id, stage } => {
                tracing::warn!(
                    target: "cortex",
                    cycle_id = *cycle_id,
                    stage = *stage,
                    "stage_failed"
                );
            }
            CortexTelemetryEvent::ReactionCompleted {
                cycle_id,
                act_count,
            } => {
                tracing::debug!(
                    target: "cortex",
                    cycle_id = *cycle_id,
                    act_count = *act_count,
                    "reaction_completed"
                );
            }
            CortexTelemetryEvent::NoopFallback { cycle_id, reason } => {
                tracing::debug!(
                    target: "cortex",
                    cycle_id = *cycle_id,
                    reason = *reason,
                    "noop_fallback"
                );
            }
        }

        if let Some(hook) = &self.telemetry_hook {
            hook(event);
        }
    }
}

#[async_trait]
impl HelperRuntime for Cortex {
    fn limits(&self) -> &ReactionLimits {
        &self.limits
    }

    fn hooks(&self) -> Option<&TestHooks> {
        self.hooks.as_ref()
    }

    async fn run_text_organ_with_system(
        &self,
        cycle_id: u64,
        organ: CognitionOrgan,
        max_output_tokens: u64,
        system_prompt: String,
        user_prompt: String,
    ) -> Result<String, CortexError> {
        Cortex::run_text_organ_with_system(
            self,
            cycle_id,
            organ,
            max_output_tokens,
            system_prompt,
            user_prompt,
        )
        .await
    }

    async fn run_organ(
        &self,
        cycle_id: u64,
        organ: CognitionOrgan,
        max_output_tokens: u64,
        system_prompt: String,
        user_prompt: String,
        output_mode: OutputMode,
    ) -> Result<ChatResponse, CortexError> {
        Cortex::run_organ(
            self,
            cycle_id,
            organ,
            max_output_tokens,
            system_prompt,
            user_prompt,
            output_mode,
        )
        .await
    }
}

fn build_request(
    request_id: String,
    route: Option<String>,
    max_output_tokens: u64,
    max_request_time_ms: u64,
    system_prompt: String,
    user_prompt: String,
    stage: &'static str,
    output_mode: OutputMode,
) -> ChatRequest {
    let mut metadata = BTreeMap::new();
    metadata.insert("cortex_stage".to_string(), stage.to_string());
    ChatRequest {
        request_id: Some(request_id),
        route,
        messages: vec![
            BelunaMessage {
                role: BelunaRole::System,
                parts: vec![BelunaContentPart::Text {
                    text: system_prompt,
                }],
                tool_call_id: None,
                tool_name: None,
            },
            BelunaMessage {
                role: BelunaRole::User,
                parts: vec![BelunaContentPart::Text { text: user_prompt }],
                tool_call_id: None,
                tool_name: None,
            },
        ],
        tools: vec![],
        tool_choice: ToolChoice::None,
        output_mode,
        limits: RequestLimitOverrides {
            max_output_tokens: Some(max_output_tokens),
            max_request_time_ms: Some(max_request_time_ms),
        },
        metadata,
        cost_attribution_id: None,
    }
}
