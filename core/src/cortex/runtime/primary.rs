// cortex/runtime.ts invariants:
// - Should stop at sense, neural signal descriptor, goal forest, IR, act level.

use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
    time::Instant,
};

use async_trait::async_trait;
use tokio::sync::Mutex;
use tokio::time::{Duration, timeout};

use crate::{
    ai_gateway::chat::{
        Chat, ChatMessage, ChatRole, ContentPart, ContextControlReason, DeriveContextOptions,
        FinishReason, OutputMode, SystemPromptAction, Thread, ThreadContextRequest, ThreadOptions,
        ToolExecutor, ToolOverride, TurnInput, TurnLimits, TurnResponse, TurnRetentionPolicy,
    },
    ai_gateway::types::{CHAT_CAPABILITY_ID, ChatRouteAlias, ChatRouteRef},
    config::CortexRoutesConfig,
    continuity::ContinuityEngine,
    cortex::{
        error::{CortexError, extractor_failed, primary_failed},
        helpers::{
            self, CognitionOrgan, CortexHelper, HelperRuntime, goal_forest_helper::GoalNode,
            sense_input_helper,
        },
        ir, prompts,
        testing::{PrimaryRequest as TestPrimaryRequest, TestHooks},
        types::{CognitionState, CortexControlDirective, CortexOutput, ReactionLimits},
    },
    observability::{
        contract::OrganResponseStatus, metrics as observability_metrics,
        runtime as observability_runtime,
    },
    spine::ActDispatchResult,
    stem::{ActProducerHandle, AfferentRuleControlPort, EfferentActEnvelope},
    types::{Act, NeuralSignalDescriptor, PhysicalState, Sense},
};

mod apply;
mod attention;
mod cleanup;
mod executor;
mod session;
mod tools;

use executor::PrimaryToolExecutor;
use session::PrimarySession;
use tools::{
    ActToolBinding, build_act_tool_bindings, dynamic_act_tool_overrides, primary_internal_tools,
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
    chat: Option<Arc<Chat>>,
    tick_interval_ms: u64,
    routes: CortexRoutesConfig,
    hooks: Option<TestHooks>,
    helper: CortexHelper,
    telemetry_hook: Option<CortexTelemetryHook>,
    limits: ReactionLimits,
    continuity: Option<Arc<Mutex<ContinuityEngine>>>,
    afferent_rule_control: Option<Arc<dyn AfferentRuleControlPort>>,
    efferent_producer: Option<ActProducerHandle>,
    primary_session: PrimarySession,
}

#[derive(Debug, Clone, Default)]
struct PrimaryTurnState {
    next_act_seq_no: u64,
    dispatched_act_count: usize,
    break_primary_phase_requested: bool,
    protocol_violation: Option<String>,
}

#[derive(Clone)]
struct PrimaryEngineResult {
    output_text: String,
    dispatched_act_count: usize,
    pending_continuation: bool,
    goal_forest_nodes: Vec<GoalNode>,
    break_primary_phase_requested: bool,
    committed_thread: Option<Thread>,
}

#[derive(Clone)]
struct PrimaryThreadState {
    thread: Thread,
}

#[derive(Debug, Clone)]
struct PrimaryContinuationState {
    sense_tool_context: sense_input_helper::SenseToolContext,
    act_binding_map: HashMap<String, ActToolBinding>,
    dynamic_act_tool_overrides: Vec<ToolOverride>,
    working_goal_forest_nodes: Vec<GoalNode>,
    turn_state: PrimaryTurnState,
    next_step: u64,
}

impl Cortex {
    pub fn from_config(
        config: &crate::config::CortexRuntimeConfig,
        tick_interval_ms: u64,
        chat: Arc<Chat>,
        telemetry_hook: Option<CortexTelemetryHook>,
        continuity: Option<Arc<Mutex<ContinuityEngine>>>,
        afferent_rule_control: Option<Arc<dyn AfferentRuleControlPort>>,
        efferent_producer: Option<ActProducerHandle>,
    ) -> Self {
        let limits = config.default_limits.clone();
        log_output_token_limits_paused(&limits);
        Self {
            chat: Some(chat),
            tick_interval_ms: tick_interval_ms.max(1),
            routes: config.routes.clone(),
            hooks: None,
            helper: CortexHelper::default(),
            telemetry_hook,
            limits,
            continuity,
            afferent_rule_control,
            efferent_producer,
            primary_session: PrimarySession::new(),
        }
    }

    pub(crate) fn for_test_with_hooks(hooks: TestHooks, limits: ReactionLimits) -> Self {
        log_output_token_limits_paused(&limits);
        Self {
            chat: None,
            tick_interval_ms: 1_000,
            routes: CortexRoutesConfig::default(),
            hooks: Some(hooks),
            helper: CortexHelper::default(),
            telemetry_hook: None,
            limits,
            continuity: None,
            afferent_rule_control: None,
            efferent_producer: None,
            primary_session: PrimarySession::new(),
        }
    }

    pub async fn cognition_state_snapshot(&self) -> Result<CognitionState, CortexError> {
        let continuity = self.continuity.as_ref().ok_or_else(|| {
            CortexError::new(
                crate::cortex::error::CortexErrorKind::Internal,
                "continuity is not configured for this Cortex instance",
            )
        })?;
        Ok(continuity.lock().await.cognition_state_snapshot())
    }

    pub async fn persist_cognition_state(&self, state: CognitionState) -> Result<(), CortexError> {
        let continuity = self.continuity.as_ref().ok_or_else(|| {
            CortexError::new(
                crate::cortex::error::CortexErrorKind::Internal,
                "continuity is not configured for this Cortex instance",
            )
        })?;
        continuity
            .lock()
            .await
            .persist_cognition_state(state)
            .map_err(|err| {
                CortexError::new(
                    crate::cortex::error::CortexErrorKind::Internal,
                    format!("persist_cognition_state_failed: {err}"),
                )
            })
    }

    pub async fn cortex(
        &self,
        senses: &[Sense],
        physical_state: &PhysicalState,
    ) -> Result<CortexOutput, CortexError> {
        self.emit(CortexTelemetryEvent::ReactionStarted {
            cycle_id: physical_state.cycle_id,
        });
        observability_metrics::record_cortex_cycle_id(physical_state.cycle_id);

        let cognition_state = match self.cognition_state_snapshot().await {
            Ok(state) => state,
            Err(err) => {
                self.emit(CortexTelemetryEvent::StageFailed {
                    cycle_id: physical_state.cycle_id,
                    stage: "cognition_snapshot",
                });
                tracing::warn!(
                    target: "cortex",
                    cycle_id = physical_state.cycle_id,
                    error = %err,
                    "cognition_snapshot_failed_noop"
                );
                return Ok(self.noop_output(physical_state.cycle_id, "cognition_snapshot_failed"));
            }
        };

        let deadline = Duration::from_millis(self.limits.max_cycle_time_ms.max(1));
        let sense_descriptors = helpers::sense_descriptors(&physical_state.ns_descriptor.entries);
        let act_descriptors = helpers::act_descriptors(&physical_state.ns_descriptor.entries);
        observability_metrics::record_cortex_input_ir_act_descriptor_catalog_count(
            act_descriptors.len(),
        );

        let senses_owned = senses.to_vec();
        let sense_tool_context =
            sense_input_helper::SenseToolContext::from_inputs(&senses_owned, &sense_descriptors);
        let goal_forest = cognition_state.goal_forest.clone();

        let (senses_section, proprioception_section, goal_forest_section) = tokio::join!(
            self.helper.input.sense.to_input_ir_section_from_context(
                physical_state.cycle_id,
                &sense_tool_context,
                self.limits.sense_passthrough_max_bytes,
            ),
            async {
                self.helper
                    .input
                    .proprioception
                    .to_input_ir_section(physical_state.cycle_id, &physical_state.proprioception)
            },
            self.helper.input.goal_forest.to_input_ir_section(
                self,
                physical_state.cycle_id,
                deadline,
                &goal_forest,
            )
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
            input_ir_goal_forest = %goal_forest_section,
            "input_ir_goal_forest"
        );
        let input_ir = ir::build_input_ir(
            &senses_section,
            &proprioception_section,
            &goal_forest_section,
        );
        let primary_input_payload = ir::build_primary_input_payload(
            &senses_section,
            &proprioception_section,
            &goal_forest_section,
        );

        let primary_result = timeout(
            deadline,
            self.run_primary_engine(
                physical_state.cycle_id,
                primary_input_payload,
                input_ir.text.clone(),
                sense_tool_context,
                act_descriptors.clone(),
                sense_descriptors.clone(),
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
                return Ok(self.noop_output(physical_state.cycle_id, "primary_failed"));
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
                return Ok(self.noop_output(physical_state.cycle_id, "primary_timeout"));
            }
        };
        let emit_goal_forest_snapshot = || {
            observability_runtime::emit_cortex_goal_forest_snapshot(
                physical_state.cycle_id,
                &primary_output.goal_forest_nodes,
            );
        };

        if !primary_output.pending_continuation && !primary_output.break_primary_phase_requested {
            let _output_ir = match ir::parse_output_ir(&primary_output.output_text) {
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
                    emit_goal_forest_snapshot();
                    return Ok(self.noop_output(physical_state.cycle_id, "primary_contract"));
                }
            };
        }

        let mut control = CortexControlDirective::default();
        if primary_output.break_primary_phase_requested {
            if let Some(committed_thread) = primary_output.committed_thread.as_ref() {
                let (attention_result, cleanup_result) = tokio::join!(
                    self.run_attention_phase(physical_state.cycle_id, committed_thread),
                    self.run_cleanup_phase(
                        physical_state.cycle_id,
                        committed_thread,
                        &primary_output.goal_forest_nodes,
                    ),
                );

                match attention_result {
                    Ok(attention_output) => {
                        match self.apply_attention_result(attention_output).await {
                            Ok(attention_control) => {
                                control.ignore_all_trigger_for_ticks =
                                    attention_control.ignore_all_trigger_for_ticks;
                            }
                            Err(err) => {
                                self.emit(CortexTelemetryEvent::StageFailed {
                                    cycle_id: physical_state.cycle_id,
                                    stage: "attention_apply",
                                });
                                tracing::warn!(
                                    target: "cortex",
                                    cycle_id = physical_state.cycle_id,
                                    error = %err,
                                    "attention_apply_failed"
                                );
                            }
                        }
                    }
                    Err(err) => {
                        self.emit(CortexTelemetryEvent::StageFailed {
                            cycle_id: physical_state.cycle_id,
                            stage: "attention",
                        });
                        tracing::warn!(
                            target: "cortex",
                            cycle_id = physical_state.cycle_id,
                            error = %err,
                            "attention_failed"
                        );
                    }
                }

                match cleanup_result {
                    Ok(cleanup_output) => {
                        if let Err(err) = self
                            .apply_cleanup_result(physical_state.cycle_id, cleanup_output)
                            .await
                        {
                            self.emit(CortexTelemetryEvent::StageFailed {
                                cycle_id: physical_state.cycle_id,
                                stage: "cleanup_apply",
                            });
                            tracing::warn!(
                                target: "cortex",
                                cycle_id = physical_state.cycle_id,
                                error = %err,
                                "cleanup_apply_failed"
                            );
                        }
                    }
                    Err(err) => {
                        self.emit(CortexTelemetryEvent::StageFailed {
                            cycle_id: physical_state.cycle_id,
                            stage: "cleanup",
                        });
                        tracing::warn!(
                            target: "cortex",
                            cycle_id = physical_state.cycle_id,
                            error = %err,
                            "cleanup_failed"
                        );
                    }
                }
            }
        }

        if !primary_output.pending_continuation && primary_output.dispatched_act_count == 0 {
            self.emit(CortexTelemetryEvent::NoopFallback {
                cycle_id: physical_state.cycle_id,
                reason: "no_dispatched_acts",
            });
        }
        self.emit(CortexTelemetryEvent::ReactionCompleted {
            cycle_id: physical_state.cycle_id,
            act_count: primary_output.dispatched_act_count,
        });
        emit_goal_forest_snapshot();

        Ok(CortexOutput {
            control,
            pending_primary_continuation: primary_output.pending_continuation,
        })
    }

    async fn run_primary_engine(
        &self,
        cycle_id: u64,
        primary_input: String,
        input_ir_internal: String,
        sense_tool_context: sense_input_helper::SenseToolContext,
        act_descriptors: Vec<NeuralSignalDescriptor>,
        sense_descriptors: Vec<NeuralSignalDescriptor>,
        initial_goal_forest_nodes: Vec<GoalNode>,
    ) -> Result<PrimaryEngineResult, CortexError> {
        let act_bindings = build_act_tool_bindings(&act_descriptors, &sense_descriptors);
        let fresh_act_binding_map = act_bindings
            .iter()
            .map(|binding| (binding.alias.clone(), binding.clone()))
            .collect::<HashMap<_, _>>();
        let fresh_dynamic_act_tool_overrides =
            dynamic_act_tool_overrides(&act_bindings, self.limits.max_waiting_ticks);

        if let Some(hooks) = &self.hooks {
            let output = (hooks.primary)(TestPrimaryRequest {
                cycle_id,
                input_ir: primary_input.clone(),
            })
            .await?;
            let stage = CognitionOrgan::Primary.stage();
            helpers::log_organ_output(cycle_id, stage, &output);
            return Ok(PrimaryEngineResult {
                output_text: output,
                dispatched_act_count: 0,
                pending_continuation: false,
                goal_forest_nodes: initial_goal_forest_nodes,
                break_primary_phase_requested: false,
                committed_thread: None,
            });
        }

        let prior_continuation = self.primary_session.take_continuation().await;
        let prior_continuation_backup = prior_continuation.clone();

        let (
            mut input_messages,
            effective_sense_tool_context,
            effective_act_binding_map,
            dynamic_act_tool_overrides,
            mut working_goal_forest_nodes,
            mut turn_state,
            mut step,
            mut mode,
        ) = match prior_continuation {
            Some(state) => (
                vec![build_primary_user_message(&primary_input)],
                sense_input_helper::SenseToolContext::merged(
                    &state.sense_tool_context,
                    &sense_tool_context,
                ),
                state.act_binding_map,
                state.dynamic_act_tool_overrides,
                state.working_goal_forest_nodes,
                state.turn_state,
                state.next_step,
                "continuation",
            ),
            None => (
                vec![build_primary_user_message(&primary_input)],
                sense_tool_context,
                fresh_act_binding_map,
                fresh_dynamic_act_tool_overrides,
                initial_goal_forest_nodes,
                PrimaryTurnState::default(),
                0,
                "new_turn",
            ),
        };
        let thread = self.ensure_primary_thread(cycle_id).await?;

        for turn_index in 0..usize::from(self.limits.max_primary_turns_per_tick.max(1)) {
            let mut act_tool_aliases = effective_act_binding_map
                .keys()
                .cloned()
                .collect::<Vec<_>>();
            act_tool_aliases.sort();

            let stage = CognitionOrgan::Primary.stage();
            let input_payload = helpers::pretty_json(&serde_json::json!({
                "primary_input": &primary_input,
                "input_ir_internal": &input_ir_internal,
                "mode": mode,
                "step": step,
                "turn_index": turn_index,
                "goal_forest_size": working_goal_forest_nodes.len(),
                "sense_ids": effective_sense_tool_context
                    .entries()
                    .iter()
                    .map(|entry| entry.sense_ref_id.clone())
                    .collect::<Vec<_>>(),
                "act_tool_aliases": act_tool_aliases,
            }));
            helpers::log_organ_input(cycle_id, stage, &input_payload);

            let tool_executor = Arc::new(PrimaryToolExecutor::new(
                self.clone(),
                cycle_id,
                step,
                effective_sense_tool_context.clone(),
                effective_act_binding_map.clone(),
                working_goal_forest_nodes,
                turn_state,
            ));
            let response = match self
                .run_primary_turn(
                    cycle_id,
                    step,
                    &thread,
                    input_messages.clone(),
                    dynamic_act_tool_overrides.clone(),
                    Some(tool_executor.clone()),
                )
                .await
            {
                Ok(response) => response,
                Err(err) => {
                    if should_reset_primary_context_after_tool_history_error(&err) {
                        self.reset_primary_thread_state("invalid_tool_message_chain")
                            .await;
                    } else if let Some(state) = prior_continuation_backup {
                        self.primary_session.set_continuation(state).await;
                    }
                    return Err(err);
                }
            };
            working_goal_forest_nodes = tool_executor.goal_forest_nodes().await;
            turn_state = tool_executor.turn_state().await;
            let assistant_text = response.output_text.trim().to_string();

            if let Some(protocol_violation) = turn_state.protocol_violation.as_ref() {
                return Err(primary_failed(protocol_violation.clone()));
            }

            if turn_state.break_primary_phase_requested {
                if !assistant_text.is_empty() {
                    helpers::log_organ_output(cycle_id, stage, &assistant_text);
                }
                return Ok(PrimaryEngineResult {
                    output_text: assistant_text,
                    dispatched_act_count: turn_state.dispatched_act_count,
                    pending_continuation: false,
                    goal_forest_nodes: working_goal_forest_nodes,
                    break_primary_phase_requested: true,
                    committed_thread: Some(thread.clone()),
                });
            }

            step = step.saturating_add(1);
            if response.pending_tool_call_continuation {
                input_messages = Vec::new();
                mode = "tool_continuation";
            } else {
                if !assistant_text.is_empty() {
                    helpers::log_organ_output(cycle_id, stage, &assistant_text);
                }
                input_messages = vec![build_primary_user_message(
                    &prompts::primary_break_reminder_prompt(),
                )];
                mode = "break_reminder";
            }
        }

        Err(primary_failed(format!(
            "primary exceeded max_primary_turns_per_tick={}",
            self.limits.max_primary_turns_per_tick
        )))
    }

    async fn ensure_primary_thread(&self, cycle_id: u64) -> Result<Thread, CortexError> {
        if let Some(thread) = self.primary_session.thread().await {
            return Ok(thread);
        }

        let gateway = self.chat.as_ref().ok_or_else(|| {
            CortexError::new(
                crate::cortex::error::CortexErrorKind::Internal,
                "AI Gateway is not configured for this Cortex instance",
            )
        })?;
        let route_ref = alias_route_ref(self.resolve_route(CognitionOrgan::Primary));
        let options = ThreadOptions {
            thread_id: Some("cortex-primary-thread".to_string()),
            route_ref,
            tools: primary_internal_tools(),
            system_prompt: Some(prompts::primary_system_prompt()),
            metadata: organ_thread_metadata(cycle_id, CognitionOrgan::Primary.stage()),
            ..ThreadOptions::default()
        };
        let thread = gateway
            .open_thread(ThreadOptions { ..options })
            .await
            .map_err(|err| primary_failed(err.to_string()))?;
        self.primary_session.set_thread(thread.clone()).await;
        Ok(thread)
    }

    async fn reset_primary_thread_state(&self, reason: &'static str) {
        self.primary_session.reset().await;
        tracing::warn!(target: "cortex", reason = reason, "primary_thread_state_reset");
    }

    async fn replace_primary_thread_with_selected_turns(
        &self,
        cycle_id: u64,
        source_thread: &Thread,
        selected_turn_ids: &[u64],
        system_prompt: String,
        parent_span_id_when_present: Option<String>,
    ) -> Result<(), CortexError> {
        let chat = self.chat.as_ref().ok_or_else(|| {
            CortexError::new(
                crate::cortex::error::CortexErrorKind::Internal,
                "AI Gateway is not configured for this Cortex instance",
            )
        })?;
        let route_ref = alias_route_ref(self.resolve_route(CognitionOrgan::Primary));
        let mut metadata = organ_thread_metadata(cycle_id, CognitionOrgan::Primary.stage());
        if let Some(parent_span_id) = parent_span_id_when_present {
            metadata.insert("parent_span_id".to_string(), parent_span_id);
        }
        let (derived, _) = chat
            .derive_context(
                source_thread,
                ThreadContextRequest {
                    retention: TurnRetentionPolicy::KeepSelectedTurnIds {
                        turn_ids: selected_turn_ids.to_vec(),
                    },
                    system_prompt: SystemPromptAction::Replace {
                        prompt: system_prompt,
                    },
                    drop_unfinished_continuation: true,
                    reason: ContextControlReason::CortexReset,
                },
                DeriveContextOptions {
                    thread_id: Some("cortex-primary-thread".to_string()),
                    route_ref,
                    metadata,
                },
            )
            .await
            .map_err(|err| primary_failed(err.to_string()))?;

        self.primary_session.clear_continuation().await;
        self.primary_session.set_thread(derived).await;
        Ok(())
    }

    async fn run_primary_turn(
        &self,
        cycle_id: u64,
        step: u64,
        thread: &Thread,
        input_messages: Vec<ChatMessage>,
        tool_overrides: Vec<ToolOverride>,
        tool_executor: Option<Arc<dyn ToolExecutor>>,
    ) -> Result<TurnResponse, CortexError> {
        let stage = CognitionOrgan::Primary.stage();
        let request_id = format!("cortex-{stage}-{cycle_id}-turn-{step}");
        let started_at = Instant::now();
        let route_or_organ = self
            .resolve_route(CognitionOrgan::Primary)
            .unwrap_or_else(|| stage.to_string());
        observability_runtime::emit_cortex_organ_start(
            cycle_id,
            stage,
            Some(&route_or_organ),
            &request_id,
            serde_json::json!({
                "input_messages": input_messages,
                "tool_overrides": tool_overrides_payload(&tool_overrides),
                "max_output_tokens": self.limits.max_primary_output_tokens,
                "max_request_time_ms": self.limits.max_cycle_time_ms,
                "output_mode": "text",
            }),
        );
        let mut input = build_turn_input(
            cycle_id,
            request_id.clone(),
            self.limits.max_primary_output_tokens,
            self.limits.max_cycle_time_ms,
            input_messages,
            tool_overrides,
            stage,
            OutputMode::Text,
        );
        input.tool_executor = tool_executor;
        let output = thread.complete(input).await.map_err(|err| {
            observability_runtime::emit_cortex_organ_end(
                cycle_id,
                stage,
                &request_id,
                OrganResponseStatus::Error,
                None,
                Some(gateway_error_summary(&err)),
                None,
                Some(thread.thread_id()),
                None,
            );
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
        observability_runtime::emit_cortex_organ_end(
            cycle_id,
            stage,
            &request_id,
            OrganResponseStatus::Ok,
            Some(turn_response_payload(&output.response)),
            None,
            ai_gateway_request_id(&output.response),
            Some(output.thread_id.as_str()),
            Some(output.turn_id),
        );

        Ok(output.response)
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
    ) -> Result<TurnResponse, CortexError> {
        let stage = organ.stage();
        let request_id = format!("cortex-{stage}-{cycle_id}");
        let started_at = Instant::now();
        let route = self.resolve_route(organ);
        let route_or_organ = route.clone().unwrap_or_else(|| stage.to_string());
        let output_mode_label = output_mode_label(&output_mode);
        observability_runtime::emit_cortex_organ_start(
            cycle_id,
            stage,
            Some(&route_or_organ),
            &request_id,
            serde_json::json!({
                "system_prompt": system_prompt,
                "user_prompt": user_prompt,
                "max_output_tokens": max_output_tokens,
                "output_mode": output_mode_label,
            }),
        );
        let input = build_turn_input(
            cycle_id,
            request_id.clone(),
            max_output_tokens,
            self.limits.max_cycle_time_ms,
            vec![ChatMessage {
                role: ChatRole::User,
                parts: vec![ContentPart::Text {
                    text: user_prompt.clone(),
                }],
                tool_call_id: None,
                tool_name: None,
                tool_calls: vec![],
            }],
            Vec::new(),
            stage,
            output_mode,
        );

        let chat = self.chat.as_ref().ok_or_else(|| {
            CortexError::new(
                crate::cortex::error::CortexErrorKind::Internal,
                "AI Gateway is not configured for this Cortex instance",
            )
        })?;
        let thread = match chat
            .open_thread(ThreadOptions {
                thread_id: Some(format!("cortex-{stage}-{cycle_id}-thread")),
                route_ref: alias_route_ref(route),
                system_prompt: Some(system_prompt),
                metadata: {
                    let mut metadata = organ_thread_metadata(cycle_id, stage);
                    metadata.insert("parent_span_id".to_string(), request_id.clone());
                    metadata
                },
                ..ThreadOptions::default()
            })
            .await
        {
            Ok(thread) => thread,
            Err(err) => {
                observability_runtime::emit_cortex_organ_end(
                    cycle_id,
                    stage,
                    &request_id,
                    OrganResponseStatus::Error,
                    None,
                    Some(gateway_error_summary(&err)),
                    None,
                    None,
                    None,
                );
                return Err(map_organ_gateway_error(organ, err.to_string()));
            }
        };

        let result = thread.complete(input).await;
        let output = result.map_err(|err| {
            observability_runtime::emit_cortex_organ_end(
                cycle_id,
                stage,
                &request_id,
                OrganResponseStatus::Error,
                None,
                Some(gateway_error_summary(&err)),
                None,
                Some(thread.thread_id()),
                None,
            );
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
        observability_runtime::emit_cortex_organ_end(
            cycle_id,
            stage,
            &request_id,
            OrganResponseStatus::Ok,
            Some(turn_response_payload(&output.response)),
            None,
            ai_gateway_request_id(&output.response),
            Some(output.thread_id.as_str()),
            Some(output.turn_id),
        );

        Ok(output.response)
    }

    async fn persist_goal_forest_nodes(
        &self,
        goal_forest_nodes: &[GoalNode],
    ) -> Result<u64, CortexError> {
        let mut state = self.cognition_state_snapshot().await?;
        if state.goal_forest.nodes == goal_forest_nodes {
            return Ok(state.revision);
        }
        state.goal_forest.nodes = goal_forest_nodes.to_vec();
        state.revision = state.revision.saturating_add(1);
        self.persist_cognition_state(state.clone()).await?;
        Ok(state.revision)
    }

    async fn dispatch_act(
        &self,
        cycle_id: u64,
        act_seq_no: u64,
        act: Act,
    ) -> Result<ActDispatchResult, String> {
        let Some(producer) = self.efferent_producer.as_ref() else {
            return Err("efferent producer is not configured".to_string());
        };

        Ok(producer
            .dispatch_and_wait(
                EfferentActEnvelope::with_response(cycle_id, act_seq_no, act),
                Duration::from_millis(1),
            )
            .await)
    }

    fn noop_output(&self, cycle_id: u64, reason: &'static str) -> CortexOutput {
        self.emit(CortexTelemetryEvent::NoopFallback { cycle_id, reason });
        CortexOutput {
            control: CortexControlDirective::default(),
            pending_primary_continuation: false,
        }
    }

    fn resolve_route(&self, organ: CognitionOrgan) -> Option<String> {
        match organ {
            CognitionOrgan::Primary => self.routes.primary.clone(),
            CognitionOrgan::Attention => self.routes.attention.clone(),
            CognitionOrgan::Cleanup => self.routes.cleanup.clone(),
            CognitionOrgan::Sense => self.routes.sense_helper.clone(),
            CognitionOrgan::GoalForest => None,
            CognitionOrgan::Acts => self.routes.acts_helper.clone(),
        }
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
    ) -> Result<TurnResponse, CortexError> {
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

fn build_turn_input(
    cycle_id: u64,
    request_id: String,
    _max_output_tokens: u64,
    max_request_time_ms: u64,
    messages: Vec<ChatMessage>,
    tool_overrides: Vec<ToolOverride>,
    stage: &'static str,
    output_mode: OutputMode,
) -> TurnInput {
    let mut metadata = BTreeMap::new();
    metadata.insert("cortex_stage".to_string(), stage.to_string());
    metadata.insert("organ_id".to_string(), stage.to_string());
    metadata.insert("request_id".to_string(), request_id);
    metadata.insert("tick".to_string(), cycle_id.to_string());
    TurnInput {
        messages,
        tool_overrides,
        output_mode: Some(output_mode),
        limits: Some(TurnLimits {
            // Paused: keep config contract for future resume, but do not enforce token caps now.
            max_output_tokens: None,
            max_request_time_ms: Some(max_request_time_ms),
        }),
        enable_thinking: Some(false),
        metadata,
        ..TurnInput::default()
    }
}

fn organ_thread_metadata(cycle_id: u64, organ_id: &'static str) -> BTreeMap<String, String> {
    let mut metadata = BTreeMap::new();
    metadata.insert("tick".to_string(), cycle_id.to_string());
    metadata.insert("organ_id".to_string(), organ_id.to_string());
    metadata
}

fn alias_route_ref(route_alias: Option<String>) -> Option<ChatRouteRef> {
    route_alias.map(|alias| {
        ChatRouteRef::Alias(ChatRouteAlias {
            capability: CHAT_CAPABILITY_ID.to_string(),
            alias,
        })
    })
}

fn map_organ_gateway_error(organ: CognitionOrgan, message: String) -> CortexError {
    match organ {
        CognitionOrgan::Primary => primary_failed(message),
        CognitionOrgan::Attention
        | CognitionOrgan::Cleanup
        | CognitionOrgan::Sense
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

fn tool_overrides_payload(tool_overrides: &[ToolOverride]) -> serde_json::Value {
    serde_json::Value::Array(
        tool_overrides
            .iter()
            .map(|override_item| match override_item {
                ToolOverride::Set(definition) => serde_json::json!({
                    "kind": "set",
                    "tool": definition,
                }),
                ToolOverride::Remove(name) => serde_json::json!({
                    "kind": "remove",
                    "name": name,
                }),
            })
            .collect(),
    )
}

fn turn_response_payload(response: &TurnResponse) -> serde_json::Value {
    serde_json::to_value(response).unwrap_or_else(|_| {
        serde_json::json!({
            "serialization_error": true,
            "finish_reason": finish_reason_label(&response.finish_reason),
        })
    })
}

fn ai_gateway_request_id(response: &TurnResponse) -> Option<&str> {
    response
        .backend_metadata
        .get("request_id")
        .and_then(|value| value.as_str())
}

fn gateway_error_summary(err: &crate::ai_gateway::error::GatewayError) -> serde_json::Value {
    serde_json::json!({
        "code": serde_json::to_value(err.kind)
            .ok()
            .and_then(|value| value.as_str().map(str::to_string))
            .unwrap_or_else(|| "internal".to_string()),
        "message": err.message.clone(),
        "backend_id": err.backend_id.clone(),
        "provider_code": err.provider_code.clone(),
        "provider_http_status": err.provider_http_status,
    })
}

fn finish_reason_label(reason: &FinishReason) -> &str {
    match reason {
        FinishReason::Stop => "stop",
        FinishReason::Length => "length",
        FinishReason::ToolCalls => "tool_calls",
        FinishReason::Other(_) => "other",
    }
}

fn output_mode_label(output_mode: &OutputMode) -> &str {
    match output_mode {
        OutputMode::Text => "text",
        OutputMode::JsonObject => "json_object",
        OutputMode::JsonSchema { .. } => "json_schema",
    }
}

fn should_reset_primary_context_after_tool_history_error(err: &CortexError) -> bool {
    err.message.contains("messages with role")
        && err.message.contains("tool")
        && err.message.contains("tool_calls")
}

fn build_primary_user_message(primary_input: &str) -> ChatMessage {
    ChatMessage {
        role: ChatRole::User,
        parts: vec![ContentPart::Text {
            text: prompts::build_primary_user_prompt(primary_input),
        }],
        tool_call_id: None,
        tool_name: None,
        tool_calls: vec![],
    }
}
