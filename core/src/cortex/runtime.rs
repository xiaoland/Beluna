// cortex/runtime.ts invariants:
// - Should stop at sense, neural signal descriptor, goal forest, l1 memory, IR, act level.

use std::{collections::BTreeMap, sync::Arc, time::Instant};

use async_trait::async_trait;
use serde::Deserialize;
use tokio::time::{Duration, timeout};

use crate::{
    ai_gateway::{
        chat::{ChatSessionOpenRequest, ChatThreadOpenRequest, ChatTurnRequest},
        gateway::AIGateway,
        types_chat::{
            BelunaContentPart, BelunaMessage, BelunaRole, BelunaToolDefinition, CanonicalToolCall,
            ChatResponse, OutputMode, RequestLimitOverrides, ToolChoice,
        },
    },
    config::CortexHelperRoutesConfig,
    cortex::{
        cognition::{CognitionState, GoalForestPatchOp, GoalNode},
        cognition_patch::{apply_cognition_patches, apply_goal_forest_op},
        error::{CortexError, extractor_failed, filler_failed, invalid_input, primary_failed},
        helpers::{self, CognitionOrgan, CortexHelper, HelperRuntime, sense_input_helper},
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
    helper: CortexHelper,
    telemetry_hook: Option<CortexTelemetryHook>,
    limits: ReactionLimits,
}

const PRIMARY_TOOL_EXPAND_SENSE_RAW: &str = "expand-sense-raw";
const PRIMARY_TOOL_EXPAND_SENSE_WITH_SUB_AGENT: &str = "expand-sense-with-sub-agent";
const PRIMARY_TOOL_PATCH_GOAL_FOREST: &str = "patch-goal-forest";

#[derive(Debug, Deserialize)]
struct ExpandSenseRawArgs {
    sense_ids: Vec<u64>,
}

#[derive(Debug, Deserialize)]
struct ExpandSenseWithSubAgentArgs {
    tasks: Vec<sense_input_helper::SenseSubAgentTask>,
}

#[derive(Debug, Clone)]
struct PrimaryEngineResult {
    output_text: String,
    goal_forest_nodes: Vec<GoalNode>,
}

impl Cortex {
    pub fn from_config(
        config: &crate::config::CortexRuntimeConfig,
        gateway: Arc<AIGateway>,
        telemetry_hook: Option<CortexTelemetryHook>,
    ) -> Self {
        let limits = config.default_limits.clone();
        log_output_token_limits_paused(&limits);
        Self {
            gateway: Some(gateway),
            helper_routes: config.helper_routes.clone(),
            hooks: None,
            helper: CortexHelper::default(),
            telemetry_hook,
            limits,
        }
    }

    pub(crate) fn for_test_with_hooks(hooks: TestHooks, limits: ReactionLimits) -> Self {
        log_output_token_limits_paused(&limits);
        Self {
            gateway: None,
            helper_routes: CortexHelperRoutesConfig::default(),
            hooks: Some(hooks),
            helper: CortexHelper::default(),
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
        let sense_tool_context =
            sense_input_helper::SenseToolContext::from_inputs(&senses_owned, &sense_descriptors);
        let sense_descriptors_for_helper = sense_descriptors.clone();
        let act_descriptors_for_helper = act_descriptors.clone();
        let goal_forest = cognition_state.goal_forest.clone();

        let (
            senses_section,
            proprioception_section,
            act_descriptor_catalog_section,
            goal_forest_section,
            focal_awareness_section,
        ) = tokio::join!(
            self.helper.input.sense.to_input_ir_section(
                self,
                physical_state.cycle_id,
                deadline,
                &senses_owned,
                &sense_descriptors_for_helper,
            ),
            async {
                self.helper
                    .input
                    .proprioception
                    .to_input_ir_section(physical_state.cycle_id, &physical_state.proprioception)
            },
            self.helper.input.act_descriptor.to_input_ir_section(
                self,
                physical_state.cycle_id,
                deadline,
                &act_descriptors_for_helper,
            ),
            self.helper.input.goal_forest.to_input_ir_section(
                self,
                physical_state.cycle_id,
                deadline,
                &goal_forest,
            ),
            async {
                self.helper
                    .input
                    .l1_memory
                    .to_input_ir_section(physical_state.cycle_id, &cognition_state.l1_memory)
            }
        );

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
            input_ir_proprioception = %proprioception_section,
            "input_ir_proprioception"
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
            input_ir_goal_forest = %goal_forest_section,
            "input_ir_goal_forest"
        );
        tracing::debug!(
            target: "cortex",
            cycle_id = physical_state.cycle_id,
            input_ir_focal_awareness = %focal_awareness_section,
            "input_ir_focal_awareness"
        );

        let input_ir = ir::build_input_ir(
            &senses_section,
            &proprioception_section,
            &act_descriptor_catalog_section,
            &goal_forest_section,
            &focal_awareness_section,
        );
        let primary_input_payload = ir::build_primary_input_payload(
            &senses_section,
            &proprioception_section,
            &act_descriptor_catalog_section,
            &goal_forest_section,
            &focal_awareness_section,
        );

        let primary_result = timeout(
            deadline,
            self.run_primary_engine(
                physical_state.cycle_id,
                primary_input_payload,
                input_ir.text.clone(),
                sense_tool_context,
                goal_forest.nodes.clone(),
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

        let (_output_ir, output_sections) = match ir::parse_output_ir(&primary_output.output_text) {
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

        let acts_future = async {
            match output_sections.acts_section.as_deref() {
                Some(acts_section) => {
                    self.helper
                        .output
                        .acts
                        .to_structured_output(
                            self,
                            physical_state.cycle_id,
                            deadline,
                            acts_section,
                            &act_descriptors,
                        )
                        .await
                }
                None => Vec::new(),
            }
        };
        let l1_memory_flush_future = async {
            match output_sections.l1_memory_flush_section.as_deref() {
                Some(l1_memory_flush_section) => {
                    self.helper
                        .output
                        .l1_memory_flush
                        .to_structured_output(
                            self,
                            physical_state.cycle_id,
                            deadline,
                            l1_memory_flush_section,
                            cognition_state,
                        )
                        .await
                }
                None => cognition_state.l1_memory.clone(),
            }
        };
        let (acts, l1_memory_flush) = tokio::join!(acts_future, l1_memory_flush_future);

        let apply_result = apply_cognition_patches(
            cognition_state,
            &[],
            &l1_memory_flush,
            self.limits.max_l1_memory_entries,
        );
        if apply_result.l1_memory_overflow_count > 0 {
            tracing::warn!(
                target: "cortex",
                cycle_id = physical_state.cycle_id,
                max_l1_memory_entries = self.limits.max_l1_memory_entries,
                overflow_entries = apply_result.l1_memory_overflow_count,
                "l1_memory_flush_overflow_discarded"
            );
        }
        let mut new_cognition_state = apply_result.new_cognition_state;
        let goal_forest_changed =
            new_cognition_state.goal_forest.nodes != primary_output.goal_forest_nodes;
        if goal_forest_changed {
            new_cognition_state.goal_forest.nodes = primary_output.goal_forest_nodes.clone();
            if new_cognition_state.revision == cognition_state.revision {
                new_cognition_state.revision = new_cognition_state.revision.saturating_add(1);
            }
        }

        let acts_json = serde_json::to_string(&acts)
            .unwrap_or_else(|err| format!("{{\"serialization_error\":\"{}\"}}", err));
        tracing::debug!(
            target: "cortex",
            cycle_id = physical_state.cycle_id,
            output_ir_l1_memory_flush = output_sections
                .l1_memory_flush_section
                .as_deref()
                .unwrap_or(""),
            output_ir_l1_memory_flush_present = output_sections.l1_memory_flush_section.is_some(),
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
            goal_forest_changed = goal_forest_changed,
            "goal_forest_state_update"
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
        sense_tool_context: sense_input_helper::SenseToolContext,
        initial_goal_forest_nodes: Vec<GoalNode>,
    ) -> Result<PrimaryEngineResult, CortexError> {
        let stage = CognitionOrgan::Primary.stage();
        let input_payload = helpers::pretty_json(&serde_json::json!({
            "primary_input": &primary_input,
            "input_ir_internal": &input_ir_internal,
            "max_internal_steps": self.limits.max_internal_steps,
            "goal_forest_size": initial_goal_forest_nodes.len(),
            "sense_ids": sense_tool_context
                .entries()
                .iter()
                .map(|entry| entry.sense_instance_id)
                .collect::<Vec<_>>(),
        }));
        helpers::log_organ_input(cycle_id, stage, &input_payload);

        if let Some(hooks) = &self.hooks {
            let output = (hooks.primary)(TestPrimaryRequest {
                cycle_id,
                input_ir: primary_input.clone(),
            })
            .await?;
            helpers::log_organ_output(cycle_id, stage, &output);
            return Ok(PrimaryEngineResult {
                output_text: output,
                goal_forest_nodes: initial_goal_forest_nodes,
            });
        }

        let gateway = self.gateway.as_ref().ok_or_else(|| {
            CortexError::new(
                crate::cortex::error::CortexErrorKind::Internal,
                "AI Gateway is not configured for this Cortex instance",
            )
        })?;
        let chat_gateway = gateway.clone().chat();
        let route = self.resolve_route(CognitionOrgan::Primary);
        let session = chat_gateway
            .open_session(ChatSessionOpenRequest {
                session_id: Some(format!("cortex-primary-session-{cycle_id}")),
                default_route_ref: route,
                metadata: BTreeMap::new(),
            })
            .await
            .map_err(|err| primary_failed(err.to_string()))?;
        let thread = match session
            .open_thread(ChatThreadOpenRequest {
                thread_id: Some(format!("cortex-primary-thread-{cycle_id}")),
                seed_messages: vec![BelunaMessage {
                    role: BelunaRole::System,
                    parts: vec![BelunaContentPart::Text {
                        text: prompts::primary_system_prompt(),
                    }],
                    tool_call_id: None,
                    tool_name: None,
                    tool_calls: vec![],
                }],
                metadata: BTreeMap::new(),
            })
            .await
        {
            Ok(thread) => thread,
            Err(err) => {
                session.close().await;
                return Err(primary_failed(err.to_string()));
            }
        };

        let mut turn_messages = vec![BelunaMessage {
            role: BelunaRole::User,
            parts: vec![BelunaContentPart::Text {
                text: prompts::build_primary_user_prompt(&primary_input),
            }],
            tool_call_id: None,
            tool_name: None,
            tool_calls: vec![],
        }];
        let mut working_goal_forest_nodes = initial_goal_forest_nodes;

        let max_internal_steps = self.limits.max_internal_steps.max(1);
        for step in 0..max_internal_steps {
            let response = match self
                .run_primary_micro_loop_turn(cycle_id, step, &thread, turn_messages.clone())
                .await
            {
                Ok(response) => response,
                Err(err) => {
                    session.close().await;
                    return Err(err);
                }
            };
            let assistant_text = response.output_text.trim().to_string();

            if response.tool_calls.is_empty() {
                if assistant_text.is_empty() {
                    session.close().await;
                    return Err(primary_failed(
                        "primary produced empty output without internal tool actions",
                    ));
                }
                helpers::log_organ_output(cycle_id, stage, &assistant_text);
                session.close().await;
                return Ok(PrimaryEngineResult {
                    output_text: assistant_text,
                    goal_forest_nodes: working_goal_forest_nodes,
                });
            }

            let tool_messages = self
                .run_primary_internal_tool_calls(
                    cycle_id,
                    step,
                    &response.tool_calls,
                    &sense_tool_context,
                    &mut working_goal_forest_nodes,
                )
                .await;
            turn_messages = tool_messages;
        }

        session.close().await;
        Err(primary_failed(format!(
            "primary micro-loop exceeded max_internal_steps={}",
            max_internal_steps
        )))
    }

    async fn run_primary_micro_loop_turn(
        &self,
        cycle_id: u64,
        step: u8,
        thread: &crate::ai_gateway::chat::ChatThreadHandle,
        input_messages: Vec<BelunaMessage>,
    ) -> Result<ChatResponse, CortexError> {
        let stage = CognitionOrgan::Primary.stage();
        let request_id = format!("cortex-{stage}-{cycle_id}-turn-{step}");
        let started_at = Instant::now();
        let request = build_turn_request(
            request_id.clone(),
            self.limits.max_primary_output_tokens,
            self.limits.max_cycle_time_ms,
            input_messages,
            primary_internal_tools(),
            ToolChoice::Auto,
            stage,
            OutputMode::Text,
        );
        let response = thread.turn_once(request).await.map_err(|err| {
            tracing::debug!(
                target: "cortex",
                stage = stage,
                cycle_id = cycle_id,
                step = step,
                request_id = %request_id,
                elapsed_ms = started_at.elapsed().as_millis() as u64,
                error_kind = ?err.kind,
                error = %err.message,
                "llm_call_failed"
            );
            primary_failed(err.to_string())
        })?;

        Ok(response.response)
    }

    async fn run_primary_internal_tool_calls(
        &self,
        cycle_id: u64,
        step: u8,
        tool_calls: &[CanonicalToolCall],
        sense_tool_context: &sense_input_helper::SenseToolContext,
        goal_forest_nodes: &mut Vec<GoalNode>,
    ) -> Vec<BelunaMessage> {
        let mut tool_messages = Vec::with_capacity(tool_calls.len());
        for call in tool_calls {
            let payload = self
                .run_primary_internal_tool_call(
                    cycle_id,
                    step,
                    call,
                    sense_tool_context,
                    goal_forest_nodes,
                )
                .await;
            tool_messages.push(BelunaMessage {
                role: BelunaRole::Tool,
                parts: vec![BelunaContentPart::Json { value: payload }],
                tool_call_id: Some(call.id.clone()),
                tool_name: Some(call.name.clone()),
                tool_calls: vec![],
            });
        }
        tool_messages
    }

    async fn run_primary_internal_tool_call(
        &self,
        cycle_id: u64,
        step: u8,
        call: &CanonicalToolCall,
        sense_tool_context: &sense_input_helper::SenseToolContext,
        goal_forest_nodes: &mut Vec<GoalNode>,
    ) -> serde_json::Value {
        let tool_result = match call.name.as_str() {
            PRIMARY_TOOL_EXPAND_SENSE_RAW => {
                let parsed = serde_json::from_str::<ExpandSenseRawArgs>(&call.arguments_json)
                    .map_err(|err| err.to_string());
                match parsed {
                    Ok(args) => Ok(sense_input_helper::expand_sense_raw(
                        sense_tool_context,
                        &args.sense_ids,
                    )),
                    Err(err) => Err(err),
                }
            }
            PRIMARY_TOOL_EXPAND_SENSE_WITH_SUB_AGENT => {
                let parsed =
                    serde_json::from_str::<ExpandSenseWithSubAgentArgs>(&call.arguments_json)
                        .map_err(|err| err.to_string());
                match parsed {
                    Ok(args) => {
                        if args
                            .tasks
                            .iter()
                            .any(|task| task.instruction.trim().is_empty())
                        {
                            Err("task instruction cannot be empty".to_string())
                        } else {
                            sense_input_helper::expand_sense_with_sub_agent(
                                self,
                                cycle_id,
                                sense_tool_context,
                                &args.tasks,
                            )
                            .await
                            .map_err(|err| err.to_string())
                        }
                    }
                    Err(err) => Err(err),
                }
            }
            PRIMARY_TOOL_PATCH_GOAL_FOREST => {
                let parsed = parse_patch_goal_forest_ops(&call.arguments_json)
                    .map_err(|err| err.to_string());
                match parsed {
                    Ok(ops) => {
                        for op in &ops {
                            apply_goal_forest_op(goal_forest_nodes, op);
                        }
                        Ok(serde_json::json!(
                            helpers::goal_forest_input_helper::goal_forest_ascii(goal_forest_nodes)
                        ))
                    }
                    Err(err) => Err(err),
                }
            }
            _ => Err(format!(
                "unknown internal cognitive action tool '{}'",
                call.name
            )),
        };

        match tool_result {
            Ok(value) => {
                tracing::debug!(
                    target: "cortex",
                    cycle_id = cycle_id,
                    step = step,
                    tool_name = %call.name,
                    tool_call_id = %call.id,
                    "primary_internal_cognitive_action_completed"
                );
                serde_json::json!({
                    "ok": true,
                    "tool": call.name,
                    "data": value,
                })
            }
            Err(error) => {
                tracing::warn!(
                    target: "cortex",
                    cycle_id = cycle_id,
                    step = step,
                    tool_name = %call.name,
                    tool_call_id = %call.id,
                    error = %error,
                    "primary_internal_cognitive_action_failed"
                );
                serde_json::json!({
                    "ok": false,
                    "tool": call.name,
                    "error": error,
                })
            }
        }
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
        let request = build_turn_request(
            request_id.clone(),
            max_output_tokens,
            self.limits.max_cycle_time_ms,
            vec![BelunaMessage {
                role: BelunaRole::User,
                parts: vec![BelunaContentPart::Text { text: user_prompt }],
                tool_call_id: None,
                tool_name: None,
                tool_calls: vec![],
            }],
            vec![],
            ToolChoice::None,
            stage,
            output_mode,
        );

        let gateway = self.gateway.as_ref().ok_or_else(|| {
            CortexError::new(
                crate::cortex::error::CortexErrorKind::Internal,
                "AI Gateway is not configured for this Cortex instance",
            )
        })?;
        let chat_gateway = gateway.clone().chat();
        let session = chat_gateway
            .open_session(ChatSessionOpenRequest {
                session_id: Some(format!("cortex-{stage}-{cycle_id}-session")),
                default_route_ref: route,
                metadata: BTreeMap::new(),
            })
            .await
            .map_err(|err| map_organ_gateway_error(organ, err.to_string()))?;
        let thread = match session
            .open_thread(ChatThreadOpenRequest {
                thread_id: Some(format!("cortex-{stage}-{cycle_id}-thread")),
                seed_messages: vec![BelunaMessage {
                    role: BelunaRole::System,
                    parts: vec![BelunaContentPart::Text {
                        text: system_prompt,
                    }],
                    tool_call_id: None,
                    tool_name: None,
                    tool_calls: vec![],
                }],
                metadata: BTreeMap::new(),
            })
            .await
        {
            Ok(thread) => thread,
            Err(err) => {
                session.close().await;
                return Err(map_organ_gateway_error(organ, err.to_string()));
            }
        };

        let response_result = thread.turn_once(request).await;
        session.close().await;
        let response = response_result.map_err(|err| {
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
            map_organ_gateway_error(organ, err.to_string())
        })?;

        Ok(response.response)
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
            CognitionOrgan::GoalForest => self.helper_routes.goal_forest_helper.clone(),
            CognitionOrgan::Acts => self.helper_routes.acts_helper.clone(),
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

    fn emit_stage_failed(&self, cycle_id: u64, stage: &'static str) {
        self.emit(CortexTelemetryEvent::StageFailed { cycle_id, stage });
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

fn build_turn_request(
    request_id: String,
    _max_output_tokens: u64,
    max_request_time_ms: u64,
    input_messages: Vec<BelunaMessage>,
    tools: Vec<BelunaToolDefinition>,
    tool_choice: ToolChoice,
    stage: &'static str,
    output_mode: OutputMode,
) -> ChatTurnRequest {
    let mut metadata = BTreeMap::new();
    metadata.insert("cortex_stage".to_string(), stage.to_string());
    ChatTurnRequest {
        request_id: Some(request_id),
        route_ref_override: None,
        input_messages,
        tools,
        tool_choice,
        output_mode,
        limits: RequestLimitOverrides {
            // Paused: keep config contract for future resume, but do not enforce token caps now.
            max_output_tokens: None,
            max_request_time_ms: Some(max_request_time_ms),
        },
        metadata,
        cost_attribution_id: None,
    }
}

fn map_organ_gateway_error(organ: CognitionOrgan, message: String) -> CortexError {
    match organ {
        CognitionOrgan::Primary => primary_failed(message),
        CognitionOrgan::L1MemoryFlush => filler_failed(message),
        CognitionOrgan::Sense
        | CognitionOrgan::ActDescriptor
        | CognitionOrgan::GoalForest
        | CognitionOrgan::Acts => extractor_failed(message),
    }
}

fn log_output_token_limits_paused(limits: &ReactionLimits) {
    tracing::info!(
        target: "cortex",
        max_primary_output_tokens = limits.max_primary_output_tokens,
        max_sub_output_tokens = limits.max_sub_output_tokens,
        "output_token_limits_paused"
    );
}

fn primary_internal_tools() -> Vec<BelunaToolDefinition> {
    vec![
        BelunaToolDefinition {
            name: PRIMARY_TOOL_EXPAND_SENSE_RAW.to_string(),
            description: Some(
                "Get the raw payload plus schema of the senses you asked for."
                    .to_string(),
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "sense_ids": {
                        "type": "array",
                        "items": { "type": "integer", "minimum": 1 },
                        "minItems": 1
                    }
                },
                "required": ["sense_ids"]
            }),
        },
        BelunaToolDefinition {
            name: PRIMARY_TOOL_EXPAND_SENSE_WITH_SUB_AGENT.to_string(),
            description: Some(
                "Delegate few sub-agents summarizing the sense for you to avoid your own cognitive load."
                    .to_string(),
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "tasks": {
                        "type": "array",
                        "minItems": 1,
                        "items": {
                            "type": "object",
                            "properties": {
                                "sense_id": { "type": "integer", "minimum": 1 },
                                "instruction": { "type": "string", "minLength": 1 }
                            },
                            "required": ["sense_id", "instruction"],
                        }
                    }
                },
                "required": ["tasks"],
            }),
        },
        BelunaToolDefinition {
            name: PRIMARY_TOOL_PATCH_GOAL_FOREST.to_string(),
            description: Some(
                "Patch the goal-forest"
                    .to_string(),
            ),
            input_schema: patch_goal_forest_input_schema(),
        },
    ]
}

fn patch_goal_forest_input_schema() -> serde_json::Value {
    serde_json::json!({
      "type": "array",
      "minItems": 1,
      "description": "patch operations to apply, in order",
      "items": {
        "oneOf": [
          {
            "type": "object",
            "description": "add a new root goal (a new tree root)",
            "properties": {
              "op": {
                "type": "string",
                "const": "plant"
              },
              "status": {
                "type": "string",
                "default": "open"
              },
              "weight": {
                "type": "number",
                "minimum": 0,
                "maximum": 1,
                "default": 0
              },
              "id": {
                "type": "string",
                "description": "kebab-case phrase"
              },
              "summary": {
                "type": "string"
              }
            },
            "required": ["op", "id", "summary"],
            "additionalProperties": false
          },
          {
            "type": "object",
            "description": "add a non-root goal under selected parent",
            "properties": {
              "op": {
                "type": "string",
                "const": "sprout"
              },
              "parent_numbering": {
                "type": "string",
                "minLength": 1
              },
              "parent_id": {
                "type": "string",
                "minLength": 1
              },
              "numbering": {
                "type": "string",
                "description": "direct child numbering under parent, optional; auto-assign when omitted"
              },
              "status": {
                "type": "string",
                "default": "open"
              },
              "weight": {
                "type": "number",
                "minimum": 0,
                "maximum": 1,
                "default": 0
              },
              "id": {
                "type": "string",
                "description": "kebab-case phrase"
              },
              "summary": {
                "type": "string"
              }
            },
            "required": ["op", "id", "summary"],
            "additionalProperties": false,
            "anyOf": [
              { "required": ["parent_numbering"] },
              { "required": ["parent_id"] }
            ]
          },
          {
            "type": "object",
            "description": "change node fields; select with numbering or id",
            "properties": {
              "op": {
                "type": "string",
                "const": "trim"
              },
              "numbering": {
                "type": "string",
                "minLength": 1
              },
              "id": {
                "type": "string",
                "minLength": 1
              },
              "weight": {
                "type": "number",
                "description": "the new weight",
                "minimum": 0,
                "maximum": 1
              },
              "status": {
                "type": "string",
                "description": "the new status"
              }
            },
            "required": ["op"],
            "additionalProperties": false,
            "allOf": [
              {
                "anyOf": [
                  { "required": ["numbering"] },
                  { "required": ["id"] }
                ]
              },
              {
                "anyOf": [
                  { "required": ["weight"] },
                  { "required": ["status"] }
                ]
              }
            ]
          },
          {
            "type": "object",
            "description": "remove a goal node and its children; select with numbering or id",
            "properties": {
              "op": {
                "type": "string",
                "const": "prune"
              },
              "numbering": {
                "type": "string",
                "minLength": 1
              },
              "id": {
                "type": "string",
                "minLength": 1
              }
            },
            "required": ["op"],
            "additionalProperties": false,
            "anyOf": [
              { "required": ["numbering"] },
              { "required": ["id"] }
            ]
          }
        ]
      }
    })
}

fn parse_patch_goal_forest_ops(arguments_json: &str) -> Result<Vec<GoalForestPatchOp>, String> {
    serde_json::from_str::<Vec<GoalForestPatchOp>>(arguments_json).map_err(|err| err.to_string())
}
