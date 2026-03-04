// cortex/runtime.ts invariants:
// - Should stop at sense, neural signal descriptor, goal forest, IR, act level.

use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
    time::Instant,
};

use async_trait::async_trait;
use serde::Deserialize;
use tokio::sync::Mutex;
use tokio::time::{Duration, timeout};

use crate::{
    ai_gateway::chat::{
        Chat, ChatFactory, ChatMessage, ChatOptions, ChatRole, ChatToolDefinition, ContentPart,
        MessageBoundarySelector, MessageRangeSelector, OutputMode, SystemPromptUpdate, Thread,
        ThreadMessageMutationRequest, ThreadOptions, ToolCallResult, ToolOverride, TurnInput,
        TurnLimits, TurnResponse,
    },
    config::CortexHelperRoutesConfig,
    continuity::ContinuityEngine,
    cortex::{
        clamp::derive_act_instance_id,
        error::{CortexError, extractor_failed, primary_failed},
        helpers::{
            self, CognitionOrgan, CortexHelper, HelperRuntime,
            goal_forest_helper::{
                CognitionState, GoalNode, goal_forest_ascii, goal_forest_empty_one_shot,
            },
            sense_input_helper,
        },
        ir, prompts,
        testing::{PrimaryRequest as TestPrimaryRequest, TestHooks},
        types::{
            CortexControlDirective, CortexOutput, ReactionLimits, WaitForSenseControlDirective,
        },
    },
    observability::metrics as observability_metrics,
    spine::ActDispatchResult,
    stem::{ActProducerHandle, AfferentRuleControlPort, DeferralRuleAddInput, EfferentActEnvelope},
    types::{Act, NeuralSignalDescriptor, PhysicalState, Sense, build_fq_neural_signal_id},
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
    chat_factory: Option<Arc<ChatFactory>>,
    tick_interval_ms: u64,
    helper_routes: CortexHelperRoutesConfig,
    hooks: Option<TestHooks>,
    helper: CortexHelper,
    telemetry_hook: Option<CortexTelemetryHook>,
    limits: ReactionLimits,
    continuity: Option<Arc<Mutex<ContinuityEngine>>>,
    afferent_rule_control: Option<Arc<dyn AfferentRuleControlPort>>,
    efferent_producer: Option<ActProducerHandle>,
    primary_thread_state: Arc<Mutex<Option<PrimaryThreadState>>>,
    primary_continuation_state: Arc<Mutex<Option<PrimaryContinuationState>>>,
}

const PRIMARY_TOOL_EXPAND_SENSES: &str = "expand-senses";
const PRIMARY_TOOL_PATCH_GOAL_FOREST: &str = "patch-goal-forest";
const PRIMARY_TOOL_ADD_SENSE_DEFERRAL_RULE: &str = "add-sense-deferral-rule";
const PRIMARY_TOOL_REMOVE_SENSE_DEFERRAL_RULE: &str = "remove-sense-deferral-rule";
const PRIMARY_TOOL_SLEEP: &str = "sleep";

#[derive(Debug, Deserialize)]
struct ExpandSenseItem {
    sense_id: String,
    #[serde(default)]
    instruction: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ExpandSensesArgs {
    mode: String,
    senses_to_expand: Vec<ExpandSenseItem>,
}

#[derive(Debug, Deserialize)]
struct ActToolArgs {
    #[serde(default)]
    payload: serde_json::Value,
    wait_for_sense: u64,
}

#[derive(Debug, Deserialize)]
struct SleepArgs {
    ticks: u64,
}

#[derive(Debug, Deserialize)]
struct AddSenseDeferralRuleArgs {
    rule_id: String,
    #[serde(default)]
    min_weight: Option<f64>,
    #[serde(default)]
    fq_sense_id_pattern: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RemoveSenseDeferralRuleArgs {
    rule_id: String,
}

#[derive(Debug, Deserialize)]
struct PatchGoalForestArgs {
    patch_instructions: String,
    #[serde(default)]
    reset_context: bool,
}

#[derive(Debug, Clone)]
struct ActToolBinding {
    alias: String,
    descriptor: NeuralSignalDescriptor,
}

#[derive(Debug, Clone, Default)]
struct PrimaryTurnState {
    next_act_seq_no: u64,
    dispatched_act_count: usize,
    ignore_all_trigger_for_ticks: Option<u64>,
    wait_for_sense: Option<WaitForSenseControlDirective>,
}

#[derive(Debug, Clone)]
struct PrimaryEngineResult {
    output_text: String,
    dispatched_act_count: usize,
    control: CortexControlDirective,
    pending_continuation: bool,
}

#[derive(Clone)]
struct PrimaryThreadState {
    _chat: Chat,
    thread: Thread,
}

#[derive(Debug, Clone)]
struct PrimaryToolCallResult {
    payload: serde_json::Value,
    reset_messages_applied: bool,
}

#[derive(Debug, Clone, Default)]
struct PrimaryToolCallBatchResult {
    tool_messages: Vec<ChatMessage>,
    reset_messages_applied: bool,
}

#[derive(Debug, Clone)]
struct PrimaryContinuationState {
    pending_tool_messages: Vec<ChatMessage>,
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
        chat_factory: Arc<ChatFactory>,
        telemetry_hook: Option<CortexTelemetryHook>,
        continuity: Option<Arc<Mutex<ContinuityEngine>>>,
        afferent_rule_control: Option<Arc<dyn AfferentRuleControlPort>>,
        efferent_producer: Option<ActProducerHandle>,
    ) -> Self {
        let limits = config.default_limits.clone();
        log_output_token_limits_paused(&limits);
        Self {
            chat_factory: Some(chat_factory),
            tick_interval_ms: tick_interval_ms.max(1),
            helper_routes: config.helper_routes.clone(),
            hooks: None,
            helper: CortexHelper::default(),
            telemetry_hook,
            limits,
            continuity,
            afferent_rule_control,
            efferent_producer,
            primary_thread_state: Arc::new(Mutex::new(None)),
            primary_continuation_state: Arc::new(Mutex::new(None)),
        }
    }

    pub(crate) fn for_test_with_hooks(hooks: TestHooks, limits: ReactionLimits) -> Self {
        log_output_token_limits_paused(&limits);
        Self {
            chat_factory: None,
            tick_interval_ms: 1_000,
            helper_routes: CortexHelperRoutesConfig::default(),
            hooks: Some(hooks),
            helper: CortexHelper::default(),
            telemetry_hook: None,
            limits,
            continuity: None,
            afferent_rule_control: None,
            efferent_producer: None,
            primary_thread_state: Arc::new(Mutex::new(None)),
            primary_continuation_state: Arc::new(Mutex::new(None)),
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

        let senses_owned = senses.to_vec();
        let sense_tool_context =
            sense_input_helper::SenseToolContext::from_inputs(&senses_owned, &sense_descriptors);
        let sense_descriptors_for_helper = sense_descriptors.clone();
        let goal_forest = cognition_state.goal_forest.clone();

        let (senses_section, proprioception_section, goal_forest_section) = tokio::join!(
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

        if !primary_output.pending_continuation {
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
                    return Ok(self.noop_output(physical_state.cycle_id, "primary_contract"));
                }
            };
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

        Ok(CortexOutput {
            control: primary_output.control,
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
        initial_goal_forest_nodes: Vec<GoalNode>,
    ) -> Result<PrimaryEngineResult, CortexError> {
        let act_bindings = build_act_tool_bindings(&act_descriptors);
        let fresh_act_binding_map = act_bindings
            .iter()
            .map(|binding| (binding.alias.clone(), binding.clone()))
            .collect::<HashMap<_, _>>();
        let fresh_dynamic_act_tool_overrides = act_bindings
            .iter()
            .map(|binding| {
                ToolOverride::Set(ChatToolDefinition {
                    name: binding.alias.clone(),
                    description: Some(format!(
                        "Emit {}",
                        build_fq_neural_signal_id(
                            &binding.descriptor.endpoint_id,
                            &binding.descriptor.neural_signal_descriptor_id
                        )
                    )),
                    input_schema: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "payload": binding.descriptor.payload_schema.clone(),
                            "wait_for_sense": {
                                "type": "integer",
                                "description": "Number of ticks to wait for the specified senses after dispatching the act.",
                                "minimum": 0,
                                "maximum": self.limits.max_waiting_ticks,
                            }
                        },
                        "required": ["payload", "wait_for_sense"],
                        "additionalProperties": false
                    }),
                })
            })
            .collect::<Vec<_>>();

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
                control: CortexControlDirective::default(),
                pending_continuation: false,
            });
        }

        let mut continuation_state_guard = self.primary_continuation_state.lock().await;
        let prior_continuation = continuation_state_guard.take();
        drop(continuation_state_guard);
        let prior_continuation_backup = prior_continuation.clone();

        let (
            turn_messages,
            effective_sense_tool_context,
            effective_act_binding_map,
            dynamic_act_tool_overrides,
            mut working_goal_forest_nodes,
            mut turn_state,
            step,
            mode,
        ) = match prior_continuation {
            Some(state) => {
                let mut input_messages = state.pending_tool_messages;
                input_messages.push(build_primary_user_message(&primary_input));
                (
                    input_messages,
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
                )
            }
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
            "goal_forest_size": working_goal_forest_nodes.len(),
            "sense_ids": effective_sense_tool_context
                .entries()
                .iter()
                .map(|entry| entry.sense_ref_id.clone())
                .collect::<Vec<_>>(),
            "act_tool_aliases": act_tool_aliases,
        }));
        helpers::log_organ_input(cycle_id, stage, &input_payload);

        let thread = self.ensure_primary_thread().await?;
        let response = match self
            .run_primary_turn(
                cycle_id,
                step,
                &thread,
                turn_messages.clone(),
                dynamic_act_tool_overrides.clone(),
            )
            .await
        {
            Ok(response) => response,
            Err(err) => {
                if should_reset_primary_context_after_tool_history_error(&err) {
                    self.reset_primary_thread_state("invalid_tool_message_chain")
                        .await;
                } else if let Some(state) = prior_continuation_backup {
                    let mut guard = self.primary_continuation_state.lock().await;
                    *guard = Some(state);
                }
                return Err(err);
            }
        };
        let assistant_text = response.output_text.trim().to_string();

        if response.tool_calls.is_empty() {
            if assistant_text.is_empty() {
                return Err(primary_failed(
                    "primary produced empty output without internal tool actions",
                ));
            }
            helpers::log_organ_output(cycle_id, stage, &assistant_text);
            return Ok(PrimaryEngineResult {
                output_text: assistant_text,
                dispatched_act_count: turn_state.dispatched_act_count,
                control: CortexControlDirective {
                    ignore_all_trigger_for_ticks: turn_state.ignore_all_trigger_for_ticks,
                    wait_for_sense: turn_state.wait_for_sense.clone(),
                },
                pending_continuation: false,
            });
        }

        let batch = self
            .run_primary_internal_tool_calls(
                cycle_id,
                step,
                &thread,
                &response.tool_calls,
                &effective_sense_tool_context,
                &mut working_goal_forest_nodes,
                &effective_act_binding_map,
                &mut turn_state,
            )
            .await;

        let next_pending_tool_messages = if batch.reset_messages_applied {
            Vec::new()
        } else {
            batch.tool_messages
        };

        let mut continuation_state_guard = self.primary_continuation_state.lock().await;
        *continuation_state_guard = Some(PrimaryContinuationState {
            pending_tool_messages: next_pending_tool_messages,
            sense_tool_context: effective_sense_tool_context,
            act_binding_map: effective_act_binding_map,
            dynamic_act_tool_overrides,
            working_goal_forest_nodes,
            turn_state: turn_state.clone(),
            next_step: step.saturating_add(1),
        });
        drop(continuation_state_guard);

        Ok(PrimaryEngineResult {
            output_text: String::new(),
            dispatched_act_count: turn_state.dispatched_act_count,
            control: CortexControlDirective {
                ignore_all_trigger_for_ticks: turn_state.ignore_all_trigger_for_ticks,
                wait_for_sense: turn_state.wait_for_sense,
            },
            pending_continuation: true,
        })
    }

    async fn ensure_primary_thread(&self) -> Result<Thread, CortexError> {
        let mut guard = self.primary_thread_state.lock().await;
        if let Some(state) = guard.as_ref() {
            return Ok(state.thread.clone());
        }

        let gateway = self.chat_factory.as_ref().ok_or_else(|| {
            CortexError::new(
                crate::cortex::error::CortexErrorKind::Internal,
                "AI Gateway is not configured for this Cortex instance",
            )
        })?;
        let route = self.resolve_route(CognitionOrgan::Primary);
        let chat = gateway
            .create(ChatOptions {
                chat_id: Some("cortex-primary".to_string()),
                tools: primary_internal_tools(),
                system_prompt: Some(prompts::primary_system_prompt()),
                default_route: route,
                ..ChatOptions::default()
            })
            .await;
        let thread = chat
            .open_thread(ThreadOptions {
                thread_id: Some("cortex-primary-thread".to_string()),
                ..ThreadOptions::default()
            })
            .await
            .map_err(|err| primary_failed(err.to_string()))?;
        let state = PrimaryThreadState {
            _chat: chat,
            thread: thread.clone(),
        };
        *guard = Some(state);
        Ok(thread)
    }

    async fn reset_primary_thread_state(&self, reason: &'static str) {
        {
            let mut continuation_guard = self.primary_continuation_state.lock().await;
            *continuation_guard = None;
        }

        let prior_thread_state = {
            let mut thread_guard = self.primary_thread_state.lock().await;
            thread_guard.take()
        };
        if let Some(state) = prior_thread_state {
            state._chat.close().await;
        }

        tracing::warn!(target: "cortex", reason = reason, "primary_thread_state_reset");
    }

    async fn run_primary_turn(
        &self,
        cycle_id: u64,
        step: u64,
        thread: &Thread,
        input_messages: Vec<ChatMessage>,
        tool_overrides: Vec<ToolOverride>,
    ) -> Result<TurnResponse, CortexError> {
        let stage = CognitionOrgan::Primary.stage();
        let request_id = format!("cortex-{stage}-{cycle_id}-turn-{step}");
        let started_at = Instant::now();
        let input = build_turn_input(
            request_id.clone(),
            self.limits.max_primary_output_tokens,
            self.limits.max_cycle_time_ms,
            input_messages,
            tool_overrides,
            stage,
            OutputMode::Text,
        );
        let output = thread.complete(input).await.map_err(|err| {
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

        Ok(output.response)
    }

    async fn run_primary_internal_tool_calls(
        &self,
        cycle_id: u64,
        step: u64,
        thread: &Thread,
        tool_calls: &[ToolCallResult],
        sense_tool_context: &sense_input_helper::SenseToolContext,
        goal_forest_nodes: &mut Vec<GoalNode>,
        act_binding_map: &HashMap<String, ActToolBinding>,
        turn_state: &mut PrimaryTurnState,
    ) -> PrimaryToolCallBatchResult {
        let mut tool_messages = Vec::with_capacity(tool_calls.len());
        let mut reset_messages_applied = false;
        for call in tool_calls {
            let result = self
                .run_primary_internal_tool_call(
                    cycle_id,
                    step,
                    thread,
                    call,
                    sense_tool_context,
                    goal_forest_nodes,
                    act_binding_map,
                    turn_state,
                )
                .await;
            if result.reset_messages_applied {
                reset_messages_applied = true;
            }
            tool_messages.push(ChatMessage {
                role: ChatRole::Tool,
                parts: vec![ContentPart::Json {
                    value: result.payload,
                }],
                tool_call_id: Some(call.id.clone()),
                tool_name: Some(call.name.clone()),
                tool_calls: vec![],
            });
        }
        PrimaryToolCallBatchResult {
            tool_messages,
            reset_messages_applied,
        }
    }

    async fn run_primary_internal_tool_call(
        &self,
        cycle_id: u64,
        step: u64,
        thread: &Thread,
        call: &ToolCallResult,
        sense_tool_context: &sense_input_helper::SenseToolContext,
        goal_forest_nodes: &mut Vec<GoalNode>,
        act_binding_map: &HashMap<String, ActToolBinding>,
        turn_state: &mut PrimaryTurnState,
    ) -> PrimaryToolCallResult {
        let tool_result = if let Some(binding) = act_binding_map.get(call.name.as_str()) {
            let parsed = serde_json::from_str::<ActToolArgs>(&call.arguments_json)
                .map_err(|err| err.to_string());
            match parsed {
                Ok(args) => {
                    if args.wait_for_sense > self.limits.max_waiting_ticks {
                        Err(format!(
                            "wait_for_sense must be <= {}",
                            self.limits.max_waiting_ticks
                        ))
                    } else if args.wait_for_sense > 0
                        && binding.descriptor.emitted_sense_ids.is_empty()
                    {
                        Err(
                            "wait_for_sense requires descriptor.emitted_sense_ids to be non-empty"
                                .to_string(),
                        )
                    } else if !payload_matches_schema(
                        &args.payload,
                        &binding.descriptor.payload_schema,
                    ) {
                        Err("payload does not match act descriptor schema".to_string())
                    } else {
                        let act = Act {
                            act_instance_id: derive_act_instance_id(
                                cycle_id,
                                &[],
                                &binding.descriptor.endpoint_id,
                                &binding.descriptor.neural_signal_descriptor_id,
                                &args.payload,
                            ),
                            endpoint_id: binding.descriptor.endpoint_id.clone(),
                            neural_signal_descriptor_id: binding
                                .descriptor
                                .neural_signal_descriptor_id
                                .clone(),
                            payload: args.payload,
                        };
                        let act_instance_id = act.act_instance_id.clone();
                        let act_seq_no = turn_state.next_act_seq_no.saturating_add(1);
                        turn_state.next_act_seq_no = act_seq_no;
                        match self
                            .dispatch_act_and_wait(
                                cycle_id,
                                act_seq_no,
                                act,
                                args.wait_for_sense,
                                &binding.descriptor.emitted_sense_ids,
                            )
                            .await
                        {
                            Ok(dispatch_result) => {
                                turn_state.dispatched_act_count =
                                    turn_state.dispatched_act_count.saturating_add(1);
                                if args.wait_for_sense > 0
                                    && matches!(
                                        dispatch_result,
                                        ActDispatchResult::Acknowledged { .. }
                                    )
                                {
                                    let mut expected_fq_sense_ids =
                                        binding.descriptor.emitted_sense_ids.clone();
                                    expected_fq_sense_ids.sort();
                                    expected_fq_sense_ids.dedup();
                                    turn_state.wait_for_sense =
                                        Some(WaitForSenseControlDirective {
                                            act_instance_id,
                                            expected_fq_sense_ids,
                                            wait_ticks: args.wait_for_sense,
                                        });
                                }
                                Ok((
                                    serde_json::json!({
                                        "act_tool_alias": binding.alias,
                                        "act_fq_id": build_fq_neural_signal_id(
                                            &binding.descriptor.endpoint_id,
                                            &binding.descriptor.neural_signal_descriptor_id,
                                        ),
                                        "wait_for_sense": args.wait_for_sense,
                                        "dispatch_result": dispatch_result,
                                    }),
                                    false,
                                ))
                            }
                            Err(err) => Err(err),
                        }
                    }
                }
                Err(err) => Err(err),
            }
        } else {
            match call.name.as_str() {
                PRIMARY_TOOL_EXPAND_SENSES => {
                    let parsed = serde_json::from_str::<ExpandSensesArgs>(&call.arguments_json)
                        .map_err(|err| err.to_string());
                    match parsed {
                        Ok(args) => {
                            if args.senses_to_expand.is_empty() {
                                return PrimaryToolCallResult {
                                    payload: serde_json::json!({
                                        "ok": false,
                                        "tool": call.name,
                                        "error": "senses_to_expand cannot be empty",
                                    }),
                                    reset_messages_applied: false,
                                };
                            }
                            match args.mode.as_str() {
                                "raw" => {
                                    let sense_ids = args
                                        .senses_to_expand
                                        .iter()
                                        .map(|item| item.sense_id.clone())
                                        .collect::<Vec<_>>();
                                    Ok(sense_input_helper::expand_sense_raw(
                                        sense_tool_context,
                                        &sense_ids,
                                    ))
                                    .map(|value| (value, false))
                                }
                                "sub-agent" => {
                                    let mut tasks = Vec::with_capacity(args.senses_to_expand.len());
                                    for item in args.senses_to_expand {
                                        let instruction =
                                            item.instruction.unwrap_or_default().trim().to_string();
                                        if instruction.is_empty() {
                                            return PrimaryToolCallResult {
                                                payload: serde_json::json!({
                                                    "ok": false,
                                                    "tool": call.name,
                                                    "error": "instruction is required for sub-agent mode",
                                                }),
                                                reset_messages_applied: false,
                                            };
                                        }
                                        tasks.push(sense_input_helper::SenseSubAgentTask {
                                            sense_id: item.sense_id,
                                            instruction: Some(instruction),
                                        });
                                    }
                                    sense_input_helper::expand_sense_with_sub_agent(
                                        self,
                                        cycle_id,
                                        sense_tool_context,
                                        &tasks,
                                    )
                                    .await
                                    .map_err(|err| err.to_string())
                                    .map(|value| (value, false))
                                }
                                _ => Err("mode must be one of: raw | sub-agent".to_string()),
                            }
                        }
                        Err(err) => Err(err),
                    }
                }
                PRIMARY_TOOL_ADD_SENSE_DEFERRAL_RULE => {
                    let parsed =
                        serde_json::from_str::<AddSenseDeferralRuleArgs>(&call.arguments_json)
                            .map_err(|err| err.to_string());
                    match parsed {
                        Ok(args) => {
                            let Some(port) = self.afferent_rule_control.as_ref() else {
                                return PrimaryToolCallResult {
                                    payload: serde_json::json!({
                                        "ok": false,
                                        "tool": call.name,
                                        "error": "afferent rule-control port is not configured",
                                    }),
                                    reset_messages_applied: false,
                                };
                            };
                            port.add_rule(DeferralRuleAddInput {
                                rule_id: args.rule_id,
                                min_weight: args.min_weight,
                                fq_sense_id_pattern: args.fq_sense_id_pattern,
                            })
                            .await
                            .map(|revision| serde_json::json!({ "revision": revision }))
                            .map_err(|err| err.to_string())
                            .map(|value| (value, false))
                        }
                        Err(err) => Err(err),
                    }
                }
                PRIMARY_TOOL_REMOVE_SENSE_DEFERRAL_RULE => {
                    let args = match serde_json::from_str::<RemoveSenseDeferralRuleArgs>(
                        &call.arguments_json,
                    ) {
                        Ok(args) => args,
                        Err(err) => {
                            return PrimaryToolCallResult {
                                payload: serde_json::json!({
                                    "ok": false,
                                    "tool": call.name,
                                    "error": err.to_string(),
                                }),
                                reset_messages_applied: false,
                            };
                        }
                    };
                    let Some(port) = self.afferent_rule_control.as_ref() else {
                        return PrimaryToolCallResult {
                            payload: serde_json::json!({
                                "ok": false,
                                "tool": call.name,
                                "error": "afferent rule-control port is not configured",
                            }),
                            reset_messages_applied: false,
                        };
                    };
                    port.remove_rule(args.rule_id)
                        .await
                        .map(|revision| serde_json::json!({ "revision": revision }))
                        .map_err(|err| err.to_string())
                        .map(|value| (value, false))
                }
                PRIMARY_TOOL_SLEEP => {
                    let parsed = serde_json::from_str::<SleepArgs>(&call.arguments_json)
                        .map_err(|err| err.to_string());
                    match parsed {
                        Ok(args) => {
                            if args.ticks == 0 {
                                Err("ticks must be >= 1".to_string())
                            } else if args.ticks > self.limits.max_waiting_ticks {
                                Err(format!(
                                    "ticks must be <= {}",
                                    self.limits.max_waiting_ticks
                                ))
                            } else {
                                turn_state.ignore_all_trigger_for_ticks = Some(
                                    turn_state
                                        .ignore_all_trigger_for_ticks
                                        .unwrap_or(0)
                                        .max(args.ticks),
                                );
                                Ok((
                                    serde_json::json!({
                                        "ignore_all_trigger_for_ticks": turn_state.ignore_all_trigger_for_ticks
                                    }),
                                    false,
                                ))
                            }
                        }
                        Err(err) => Err(err),
                    }
                }
                PRIMARY_TOOL_PATCH_GOAL_FOREST => {
                    let parsed = parse_patch_goal_forest_args(&call.arguments_json);
                    match parsed {
                        Ok(args) => {
                            let patch_instructions = args.patch_instructions.trim().to_string();
                            if patch_instructions.trim().is_empty() {
                                Err("patch_instructions cannot be empty".to_string())
                            } else {
                                match self
                                    .helper
                                    .input
                                    .goal_forest
                                    .patch_goal_forest_with_sub_agent(
                                        self,
                                        cycle_id,
                                        goal_forest_nodes,
                                        &patch_instructions,
                                    )
                                    .await
                                {
                                    Ok(patch_output) => {
                                        let mut data = match patch_output {
                                            serde_json::Value::Object(map) => map,
                                            other => {
                                                let mut map = serde_json::Map::new();
                                                map.insert("patch_result".to_string(), other);
                                                map
                                            }
                                        };

                                        data.insert(
                                            "reset_context_applied".to_string(),
                                            serde_json::json!(false),
                                        );
                                        let persisted_revision = match self
                                            .persist_goal_forest_nodes(goal_forest_nodes)
                                            .await
                                        {
                                            Ok(revision) => revision,
                                            Err(err) => {
                                                return PrimaryToolCallResult {
                                                    payload: serde_json::json!({
                                                        "ok": false,
                                                        "tool": call.name,
                                                        "error": format!("persist_goal_forest_failed: {err}"),
                                                    }),
                                                    reset_messages_applied: false,
                                                };
                                            }
                                        };
                                        data.insert(
                                            "cognition_persisted_revision".to_string(),
                                            serde_json::json!(persisted_revision),
                                        );

                                        if args.reset_context {
                                            let goal_forest_section =
                                                render_goal_forest_section(goal_forest_nodes);
                                            let updated_system_prompt =
                                                prompts::primary_system_prompt_with_goal_forest(
                                                    &goal_forest_section,
                                                );
                                            match thread
                                                .mutate_messages_atomically(
                                                    ThreadMessageMutationRequest {
                                                        trim_message_range: Some(
                                                            MessageRangeSelector {
                                                                start: MessageBoundarySelector::FirstUserMessage,
                                                                end: MessageBoundarySelector::LatestAssistantToolBatchEnd,
                                                            },
                                                        ),
                                                        system_prompt_update:
                                                            SystemPromptUpdate::Replace(
                                                                updated_system_prompt,
                                                            ),
                                                    },
                                                )
                                                .await
                                            {
                                                Ok(mutation_outcome) => {
                                                    data.insert(
                                                        "reset_context_applied".to_string(),
                                                        serde_json::json!(true),
                                                    );
                                                    data.insert(
                                                        "thread_message_mutation".to_string(),
                                                        serde_json::json!({
                                                            "removed_messages": mutation_outcome.removed_messages,
                                                            "remaining_messages": mutation_outcome.remaining_messages,
                                                            "effective_system_prompt_changed": mutation_outcome.effective_system_prompt_changed,
                                                        }),
                                                    );
                                                    Ok((serde_json::Value::Object(data), true))
                                                }
                                                Err(err) => Err(err.to_string()),
                                            }
                                        } else {
                                            Ok((serde_json::Value::Object(data), false))
                                        }
                                    }
                                    Err(err) => Err(err.to_string()),
                                }
                            }
                        }
                        Err(err) => {
                            tracing::warn!(
                                target: "cortex",
                                cycle_id = cycle_id,
                                step = step,
                                tool_name = %call.name,
                                tool_call_id = %call.id,
                                error = %err,
                                arguments_json = %call.arguments_json,
                                "primary_patch_goal_forest_args_parse_failed"
                            );
                            Err(err)
                        }
                    }
                }
                _ => Err(format!(
                    "unknown internal cognitive action tool '{}'",
                    call.name
                )),
            }
        };

        match tool_result {
            Ok((value, reset_messages_applied)) => {
                tracing::debug!(
                    target: "cortex",
                    cycle_id = cycle_id,
                    step = step,
                    tool_name = %call.name,
                    tool_call_id = %call.id,
                    "primary_internal_cognitive_action_completed"
                );
                PrimaryToolCallResult {
                    payload: serde_json::json!({
                        "ok": true,
                        "tool": call.name,
                        "data": value,
                    }),
                    reset_messages_applied,
                }
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
                PrimaryToolCallResult {
                    payload: serde_json::json!({
                        "ok": false,
                        "tool": call.name,
                        "error": error,
                    }),
                    reset_messages_applied: false,
                }
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
    ) -> Result<TurnResponse, CortexError> {
        let stage = organ.stage();
        let request_id = format!("cortex-{stage}-{cycle_id}");
        let started_at = Instant::now();
        let route = self.resolve_route(organ);
        let input = build_turn_input(
            request_id.clone(),
            max_output_tokens,
            self.limits.max_cycle_time_ms,
            vec![ChatMessage {
                role: ChatRole::User,
                parts: vec![ContentPart::Text { text: user_prompt }],
                tool_call_id: None,
                tool_name: None,
                tool_calls: vec![],
            }],
            Vec::new(),
            stage,
            output_mode,
        );

        let factory = self.chat_factory.as_ref().ok_or_else(|| {
            CortexError::new(
                crate::cortex::error::CortexErrorKind::Internal,
                "AI Gateway is not configured for this Cortex instance",
            )
        })?;
        let chat = factory
            .create(ChatOptions {
                chat_id: Some(format!("cortex-{stage}-{cycle_id}")),
                system_prompt: Some(system_prompt),
                default_route: route,
                ..ChatOptions::default()
            })
            .await;
        let thread = match chat
            .open_thread(ThreadOptions {
                thread_id: Some(format!("cortex-{stage}-{cycle_id}-thread")),
                ..ThreadOptions::default()
            })
            .await
        {
            Ok(thread) => thread,
            Err(err) => {
                chat.close().await;
                return Err(map_organ_gateway_error(organ, err.to_string()));
            }
        };

        let result = thread.complete(input).await;
        chat.close().await;
        let output = result.map_err(|err| {
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

    async fn dispatch_act_and_wait(
        &self,
        cycle_id: u64,
        act_seq_no: u64,
        act: Act,
        wait_for_sense_ticks: u64,
        _expected_fq_sense_ids: &[String],
    ) -> Result<ActDispatchResult, String> {
        let Some(producer) = self.efferent_producer.as_ref() else {
            return Err("efferent producer is not configured".to_string());
        };

        let timeout_duration = if wait_for_sense_ticks == 0 {
            Duration::from_millis(1)
        } else {
            let capped_ticks = wait_for_sense_ticks.min(self.limits.max_waiting_ticks);
            Duration::from_millis(self.tick_interval_ms.saturating_mul(capped_ticks).max(1))
        };
        Ok(producer
            .dispatch_and_wait(
                EfferentActEnvelope::with_response(cycle_id, act_seq_no, act),
                timeout_duration,
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
        let stage_route = match organ {
            CognitionOrgan::Primary => self.helper_routes.primary.clone(),
            CognitionOrgan::Sense => self.helper_routes.sense_helper.clone(),
            CognitionOrgan::GoalForest => self.helper_routes.goal_forest_helper.clone(),
            CognitionOrgan::Acts => self.helper_routes.acts_helper.clone(),
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
    metadata.insert("request_id".to_string(), request_id);
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

fn map_organ_gateway_error(organ: CognitionOrgan, message: String) -> CortexError {
    match organ {
        CognitionOrgan::Primary => primary_failed(message),
        CognitionOrgan::Sense | CognitionOrgan::GoalForest | CognitionOrgan::Acts => {
            extractor_failed(message)
        }
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

fn build_act_tool_bindings(act_descriptors: &[NeuralSignalDescriptor]) -> Vec<ActToolBinding> {
    act_descriptors
        .iter()
        .map(|descriptor| ActToolBinding {
            alias: transport_safe_act_tool_alias(
                &descriptor.endpoint_id,
                &descriptor.neural_signal_descriptor_id,
            ),
            descriptor: descriptor.clone(),
        })
        .collect()
}

fn transport_safe_act_tool_alias(endpoint_id: &str, neural_signal_descriptor_id: &str) -> String {
    let fq_act_id = build_fq_neural_signal_id(endpoint_id, neural_signal_descriptor_id);
    let mut normalized = String::with_capacity(fq_act_id.len());
    for ch in fq_act_id.chars() {
        match ch {
            '.' => normalized.push('-'),
            '/' => normalized.push('_'),
            c if c.is_ascii_alphanumeric() => normalized.push(c),
            _ => normalized.push('_'),
        }
    }
    format!("act_{normalized}")
}

fn payload_matches_schema(payload: &serde_json::Value, schema: &serde_json::Value) -> bool {
    let Ok(compiled) = jsonschema::JSONSchema::compile(schema) else {
        return false;
    };
    compiled.validate(payload).is_ok()
}

fn primary_internal_tools() -> Vec<ChatToolDefinition> {
    vec![
        ChatToolDefinition {
            name: PRIMARY_TOOL_EXPAND_SENSES.to_string(),
            description: Some(
                "Expand selected senses in raw mode or using sub-agent instructions.".to_string(),
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "mode": {
                        "type": "string",
                        "enum": ["raw", "sub-agent"]
                    },
                    "senses_to_expand": {
                        "type": "array",
                        "minItems": 1,
                        "items": {
                            "type": "object",
                            "properties": {
                                "sense_id": { "type": "string", "minLength": 1 },
                                "instruction": { "type": "string", "minLength": 1 }
                            },
                            "required": ["sense_id"],
                            "additionalProperties": false
                        }
                    }
                },
                "required": ["mode", "senses_to_expand"],
                "additionalProperties": false
            }),
        },
        ChatToolDefinition {
            name: PRIMARY_TOOL_ADD_SENSE_DEFERRAL_RULE.to_string(),
            description: Some("Add one afferent deferral rule.".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "rule_id": { "type": "string", "minLength": 1 },
                    "min_weight": { "type": "number", "minimum": 0, "maximum": 1 },
                    "fq_sense_id_pattern": { "type": "string", "minLength": 1 }
                },
                "required": ["rule_id"],
                "additionalProperties": false
            }),
        },
        ChatToolDefinition {
            name: PRIMARY_TOOL_REMOVE_SENSE_DEFERRAL_RULE.to_string(),
            description: Some("Remove one afferent deferral rule by rule_id.".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "rule_id": { "type": "string", "minLength": 1 }
                },
                "required": ["rule_id"],
                "additionalProperties": false
            }),
        },
        ChatToolDefinition {
            name: PRIMARY_TOOL_SLEEP.to_string(),
            description: Some("Ignore all triggers for N ticks.".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "ticks": {
                        "type": "integer",
                        "minimum": 1
                    }
                },
                "required": ["ticks"],
                "additionalProperties": false
            }),
        },
        ChatToolDefinition {
            name: PRIMARY_TOOL_PATCH_GOAL_FOREST.to_string(),
            description: Some("Patch the goal-forest with natural-language.".to_string()),
            input_schema: patch_goal_forest_tool_input_schema(),
        },
    ]
}

fn patch_goal_forest_tool_input_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "patch_instructions": {
                "type": "string",
                "minLength": 1
            },
            "reset_context": {
                "type": "boolean",
                "default": false
            }
        },
        "required": ["patch_instructions"],
        "additionalProperties": false
    })
}

fn parse_patch_goal_forest_args(arguments_json: &str) -> Result<PatchGoalForestArgs, String> {
    serde_json::from_str::<PatchGoalForestArgs>(arguments_json).map_err(|err| err.to_string())
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

fn render_goal_forest_section(goal_forest_nodes: &[GoalNode]) -> String {
    if goal_forest_nodes.is_empty() {
        goal_forest_empty_one_shot().to_string()
    } else {
        goal_forest_ascii(goal_forest_nodes)
    }
}
