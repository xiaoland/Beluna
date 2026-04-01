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
        Chat, ChatMessage, ChatRole, ChatToolDefinition, ContentPart, ContextControlReason,
        DeriveContextOptions, FinishReason, OutputMode, SystemPromptAction, Thread,
        ThreadContextRequest, ThreadOptions, ToolCallResult, ToolExecutionRequest,
        ToolExecutionResult, ToolExecutor, ToolOverride, TurnInput, TurnLimits, TurnQuery,
        TurnResponse, TurnRetentionPolicy,
    },
    ai_gateway::types::{CHAT_CAPABILITY_ID, ChatRouteAlias, ChatRouteRef},
    config::CortexHelperRoutesConfig,
    continuity::ContinuityEngine,
    cortex::{
        clamp::derive_act_instance_id,
        error::{CortexError, extractor_failed, primary_failed},
        helpers::{
            self, CognitionOrgan, CortexHelper, HelperRuntime,
            goal_forest_helper::{GoalNode, goal_forest_ascii, goal_forest_empty_one_shot},
            sense_input_helper,
        },
        ir, prompts,
        testing::{PrimaryRequest as TestPrimaryRequest, TestHooks},
        types::{
            CognitionState, CortexControlDirective, CortexOutput, ReactionLimits,
            WaitForSenseControlDirective,
        },
    },
    observability::{
        contract::OrganResponseStatus, metrics as observability_metrics,
        runtime as observability_runtime,
    },
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
    chat: Option<Arc<Chat>>,
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
struct ExpandSenseTask {
    sense_id: String,
    #[serde(default)]
    use_subagent_and_instruction_is: Option<String>,
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
    might_emit_sense_ids: Vec<String>,
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
    goal_forest_nodes: Vec<GoalNode>,
}

#[derive(Clone)]
struct PrimaryThreadState {
    thread: Thread,
}

#[derive(Debug, Clone)]
struct PrimaryToolCallResult {
    payload: serde_json::Value,
    reset_messages_applied: bool,
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

#[derive(Clone)]
struct PrimaryToolExecutor {
    cortex: Cortex,
    cycle_id: u64,
    step: u64,
    thread: Thread,
    sense_tool_context: sense_input_helper::SenseToolContext,
    act_binding_map: HashMap<String, ActToolBinding>,
    goal_forest_nodes: Arc<Mutex<Vec<GoalNode>>>,
    turn_state: Arc<Mutex<PrimaryTurnState>>,
}

impl PrimaryToolExecutor {
    fn new(
        cortex: Cortex,
        cycle_id: u64,
        step: u64,
        thread: Thread,
        sense_tool_context: sense_input_helper::SenseToolContext,
        act_binding_map: HashMap<String, ActToolBinding>,
        goal_forest_nodes: Vec<GoalNode>,
        turn_state: PrimaryTurnState,
    ) -> Self {
        Self {
            cortex,
            cycle_id,
            step,
            thread,
            sense_tool_context,
            act_binding_map,
            goal_forest_nodes: Arc::new(Mutex::new(goal_forest_nodes)),
            turn_state: Arc::new(Mutex::new(turn_state)),
        }
    }

    async fn turn_state(&self) -> PrimaryTurnState {
        self.turn_state.lock().await.clone()
    }

    async fn goal_forest_nodes(&self) -> Vec<GoalNode> {
        self.goal_forest_nodes.lock().await.clone()
    }

    async fn execute_internal_tool_call(&self, call: &ToolCallResult) -> PrimaryToolCallResult {
        let cycle_id = self.cycle_id;
        let step = self.step;
        let tool_result = if let Some(binding) = self.act_binding_map.get(call.name.as_str()) {
            let parsed = serde_json::from_str::<ActToolArgs>(&call.arguments_json)
                .map_err(|err| err.to_string());
            match parsed {
                Ok(args) => {
                    if args.wait_for_sense > self.cortex.limits.max_waiting_ticks {
                        Err(format!(
                            "wait_for_sense must be <= {}",
                            self.cortex.limits.max_waiting_ticks
                        ))
                    } else if args.wait_for_sense > 0 && binding.might_emit_sense_ids.is_empty() {
                        Err(
                            "wait_for_sense requires act.might_emit_sense_ids to be non-empty"
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
                            might_emit_sense_ids: binding.might_emit_sense_ids.clone(),
                            payload: args.payload,
                        };
                        let act_instance_id = act.act_instance_id.clone();
                        let act_seq_no = {
                            let mut state = self.turn_state.lock().await;
                            let next = state.next_act_seq_no.saturating_add(1);
                            state.next_act_seq_no = next;
                            next
                        };
                        match self
                            .cortex
                            .dispatch_act_and_wait(
                                cycle_id,
                                act_seq_no,
                                act.clone(),
                                args.wait_for_sense,
                                &act.might_emit_sense_ids,
                            )
                            .await
                        {
                            Ok(dispatch_result) => {
                                let mut state = self.turn_state.lock().await;
                                state.dispatched_act_count =
                                    state.dispatched_act_count.saturating_add(1);
                                if args.wait_for_sense > 0
                                    && matches!(
                                        dispatch_result,
                                        ActDispatchResult::Acknowledged { .. }
                                    )
                                {
                                    let mut expected_fq_sense_ids =
                                        act.might_emit_sense_ids.clone();
                                    expected_fq_sense_ids.sort();
                                    expected_fq_sense_ids.dedup();
                                    state.wait_for_sense = Some(WaitForSenseControlDirective {
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
                    let parsed = serde_json::from_str::<Vec<ExpandSenseTask>>(&call.arguments_json)
                        .map_err(|err| err.to_string());
                    match parsed {
                        Ok(tasks) => {
                            if tasks.is_empty() {
                                return PrimaryToolCallResult {
                                    payload: serde_json::json!({
                                        "ok": false,
                                        "tool": call.name,
                                        "error": "tasks cannot be empty",
                                    }),
                                    reset_messages_applied: false,
                                };
                            }

                            let mut raw_sense_ids = Vec::new();
                            let mut sub_agent_tasks = Vec::new();

                            for task in tasks {
                                if let Some(raw_instruction) = task.use_subagent_and_instruction_is
                                {
                                    let instruction = raw_instruction.trim().to_string();
                                    if instruction.is_empty() {
                                        return PrimaryToolCallResult {
                                            payload: serde_json::json!({
                                                "ok": false,
                                                "tool": call.name,
                                                "error": "use_subagent_and_instruction_is cannot be blank",
                                            }),
                                            reset_messages_applied: false,
                                        };
                                    }
                                    sub_agent_tasks.push(sense_input_helper::SenseSubAgentTask {
                                        sense_id: task.sense_id,
                                        instruction: Some(instruction),
                                    });
                                } else {
                                    raw_sense_ids.push(task.sense_id);
                                }
                            }

                            let mut raw_items = Vec::new();
                            let mut not_found_sense_ids = Vec::new();

                            if !raw_sense_ids.is_empty() {
                                let raw_response = sense_input_helper::expand_sense_raw(
                                    &self.sense_tool_context,
                                    &raw_sense_ids,
                                );
                                raw_items = value_array_field(&raw_response, "items");
                                not_found_sense_ids.extend(string_array_field(
                                    &raw_response,
                                    "not_found_sense_ids",
                                ));
                            }

                            let sub_response = if sub_agent_tasks.is_empty() {
                                Ok(serde_json::json!({
                                    "results": [],
                                    "not_found_sense_ids": [],
                                }))
                            } else {
                                sense_input_helper::expand_sense_with_sub_agent(
                                    &self.cortex,
                                    cycle_id,
                                    &self.sense_tool_context,
                                    &sub_agent_tasks,
                                )
                                .await
                                .map_err(|err| err.to_string())
                            };

                            match sub_response {
                                Ok(sub_response) => {
                                    let sub_agent_results =
                                        value_array_field(&sub_response, "results");
                                    not_found_sense_ids.extend(string_array_field(
                                        &sub_response,
                                        "not_found_sense_ids",
                                    ));
                                    not_found_sense_ids.sort();
                                    not_found_sense_ids.dedup();

                                    Ok(serde_json::json!({
                                        "raw_items": raw_items,
                                        "sub_agent_results": sub_agent_results,
                                        "not_found_sense_ids": not_found_sense_ids,
                                    }))
                                    .map(|value| (value, false))
                                }
                                Err(err) => Err(err),
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
                            let Some(port) = self.cortex.afferent_rule_control.as_ref() else {
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
                    let Some(port) = self.cortex.afferent_rule_control.as_ref() else {
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
                            } else if args.ticks > self.cortex.limits.max_waiting_ticks {
                                Err(format!(
                                    "ticks must be <= {}",
                                    self.cortex.limits.max_waiting_ticks
                                ))
                            } else {
                                let mut state = self.turn_state.lock().await;
                                state.ignore_all_trigger_for_ticks = Some(
                                    state
                                        .ignore_all_trigger_for_ticks
                                        .unwrap_or(0)
                                        .max(args.ticks),
                                );
                                Ok((
                                    serde_json::json!({
                                        "ignore_all_trigger_for_ticks": state.ignore_all_trigger_for_ticks
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
                            let goal_forest_patch_span_id =
                                format!("cortex.goal-forest.patch:{}", call.id);
                            let patch_instructions = args.patch_instructions.trim().to_string();
                            if patch_instructions.trim().is_empty() {
                                Err("patch_instructions cannot be empty".to_string())
                            } else {
                                let mut goal_forest_nodes = self.goal_forest_nodes.lock().await;
                                match self
                                    .cortex
                                    .helper
                                    .input
                                    .goal_forest
                                    .patch_goal_forest_with_sub_agent(
                                        &self.cortex,
                                        cycle_id,
                                        &mut goal_forest_nodes,
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
                                            .cortex
                                            .persist_goal_forest_nodes(&goal_forest_nodes)
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
                                                render_goal_forest_section(&goal_forest_nodes);
                                            let updated_system_prompt =
                                                prompts::primary_system_prompt_with_goal_forest(
                                                    &goal_forest_section,
                                                );
                                            let mut selected_turn_ids = self
                                                .thread
                                                .find_turns(TurnQuery::default())
                                                .await
                                                .into_iter()
                                                .map(|turn_ref| turn_ref.turn_id)
                                                .rev()
                                                .take(2)
                                                .collect::<Vec<_>>();
                                            selected_turn_ids.reverse();

                                            match self
                                                .cortex
                                                .replace_primary_thread_with_selected_turns(
                                                    cycle_id,
                                                    &self.thread,
                                                    &selected_turn_ids,
                                                    updated_system_prompt,
                                                    Some(goal_forest_patch_span_id.clone()),
                                                )
                                                .await
                                            {
                                                Ok(()) => {
                                                    let selected_turn_ids_value =
                                                        selected_turn_ids.clone();
                                                    data.insert(
                                                        "reset_context_applied".to_string(),
                                                        serde_json::json!(true),
                                                    );
                                                    data.insert(
                                                        "selected_turn_ids".to_string(),
                                                        serde_json::json!(selected_turn_ids),
                                                    );
                                                    observability_runtime::emit_cortex_goal_forest_patch(
                                                        cycle_id,
                                                        &goal_forest_patch_span_id,
                                                        Some(serde_json::json!({
                                                            "patch_instructions": patch_instructions,
                                                            "reset_context": args.reset_context,
                                                        })),
                                                        Some(serde_json::Value::Object(data.clone())),
                                                        Some(persisted_revision),
                                                        Some(true),
                                                        Some(selected_turn_ids_value),
                                                    );
                                                    Ok((serde_json::Value::Object(data), true))
                                                }
                                                Err(err) => Err(err.to_string()),
                                            }
                                        } else {
                                            observability_runtime::emit_cortex_goal_forest_patch(
                                                cycle_id,
                                                &goal_forest_patch_span_id,
                                                Some(serde_json::json!({
                                                    "patch_instructions": patch_instructions,
                                                    "reset_context": args.reset_context,
                                                })),
                                                Some(serde_json::Value::Object(data.clone())),
                                                Some(persisted_revision),
                                                Some(false),
                                                None,
                                            );
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
}

#[async_trait]
impl ToolExecutor for PrimaryToolExecutor {
    async fn execute_call(
        &self,
        request: ToolExecutionRequest,
    ) -> Result<ToolExecutionResult, crate::ai_gateway::error::GatewayError> {
        let result = self.execute_internal_tool_call(&request.call).await;
        Ok(ToolExecutionResult {
            payload: result.payload,
            reset_messages_applied: result.reset_messages_applied,
        })
    }
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
            chat: None,
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
                    emit_goal_forest_snapshot();
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
        emit_goal_forest_snapshot();

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
        sense_descriptors: Vec<NeuralSignalDescriptor>,
        initial_goal_forest_nodes: Vec<GoalNode>,
    ) -> Result<PrimaryEngineResult, CortexError> {
        let act_bindings = build_act_tool_bindings(&act_descriptors, &sense_descriptors);
        let fresh_act_binding_map = act_bindings
            .iter()
            .map(|binding| (binding.alias.clone(), binding.clone()))
            .collect::<HashMap<_, _>>();
        let fresh_dynamic_act_tool_overrides = act_bindings
            .iter()
            .map(|binding| {
                ToolOverride::Set(ChatToolDefinition {
                    name: binding.alias.clone(),
                    // TODO: replace with NSDescriptor.description
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
                goal_forest_nodes: initial_goal_forest_nodes,
            });
        }

        let mut continuation_state_guard = self.primary_continuation_state.lock().await;
        let prior_continuation = continuation_state_guard.take();
        drop(continuation_state_guard);
        let prior_continuation_backup = prior_continuation.clone();

        let (
            input_messages,
            effective_sense_tool_context,
            effective_act_binding_map,
            dynamic_act_tool_overrides,
            working_goal_forest_nodes,
            turn_state,
            step,
            mode,
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

        let thread = self.ensure_primary_thread(cycle_id).await?;
        let tool_executor = Arc::new(PrimaryToolExecutor::new(
            self.clone(),
            cycle_id,
            step,
            thread.clone(),
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
                    let mut guard = self.primary_continuation_state.lock().await;
                    *guard = Some(state);
                }
                return Err(err);
            }
        };
        let working_goal_forest_nodes = tool_executor.goal_forest_nodes().await;
        let turn_state = tool_executor.turn_state().await;
        let assistant_text = response.output_text.trim().to_string();

        if !response.pending_tool_call_continuation {
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
                goal_forest_nodes: working_goal_forest_nodes,
            });
        }

        let goal_forest_nodes = working_goal_forest_nodes.clone();
        let mut continuation_state_guard = self.primary_continuation_state.lock().await;
        *continuation_state_guard = Some(PrimaryContinuationState {
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
            goal_forest_nodes,
        })
    }

    async fn ensure_primary_thread(&self, cycle_id: u64) -> Result<Thread, CortexError> {
        let mut guard = self.primary_thread_state.lock().await;
        if let Some(state) = guard.as_ref() {
            return Ok(state.thread.clone());
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
        let state = PrimaryThreadState {
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
        let _ = prior_thread_state;

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

        {
            let mut continuation_guard = self.primary_continuation_state.lock().await;
            *continuation_guard = None;
        }
        {
            let mut thread_guard = self.primary_thread_state.lock().await;
            *thread_guard = Some(PrimaryThreadState { thread: derived });
        }
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

fn build_act_tool_bindings(
    act_descriptors: &[NeuralSignalDescriptor],
    sense_descriptors: &[NeuralSignalDescriptor],
) -> Vec<ActToolBinding> {
    let mut endpoint_emitted_sense_catalog: HashMap<String, Vec<String>> = HashMap::new();
    for descriptor in sense_descriptors {
        let fq_sense_id = build_fq_neural_signal_id(
            &descriptor.endpoint_id,
            &descriptor.neural_signal_descriptor_id,
        );
        endpoint_emitted_sense_catalog
            .entry(descriptor.endpoint_id.clone())
            .or_default()
            .push(fq_sense_id);
    }
    for might_emit_sense_ids in endpoint_emitted_sense_catalog.values_mut() {
        might_emit_sense_ids.sort();
        might_emit_sense_ids.dedup();
    }

    act_descriptors
        .iter()
        .map(|descriptor| {
            let might_emit_sense_ids = endpoint_emitted_sense_catalog
                .get(&descriptor.endpoint_id)
                .cloned()
                .unwrap_or_default();
            ActToolBinding {
                alias: transport_safe_act_tool_alias(
                    &descriptor.endpoint_id,
                    &descriptor.neural_signal_descriptor_id,
                ),
                descriptor: descriptor.clone(),
                might_emit_sense_ids,
            }
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

fn value_array_field(object: &serde_json::Value, key: &str) -> Vec<serde_json::Value> {
    object
        .get(key)
        .and_then(|value| value.as_array())
        .cloned()
        .unwrap_or_default()
}

fn string_array_field(object: &serde_json::Value, key: &str) -> Vec<String> {
    object
        .get(key)
        .and_then(|value| value.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(|value| value.to_string()))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
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

fn primary_internal_tools() -> Vec<ChatToolDefinition> {
    vec![
        ChatToolDefinition {
            name: PRIMARY_TOOL_EXPAND_SENSES.to_string(),
            description: Some(
                "Expand senses with raw payload or per-task sub-agent instruction.".to_string(),
            ),
            input_schema: serde_json::json!({
                "type": "array",
                "minItems": 1,
                "items": {
                    "type": "object",
                    "properties": {
                        "sense_id": { "type": "string", "minLength": 1 },
                        "use_subagent_and_instruction_is": { "type": "string", "minLength": 1 }
                    },
                    "required": ["sense_id"]
                }
            }),
        },
        ChatToolDefinition {
            name: PRIMARY_TOOL_ADD_SENSE_DEFERRAL_RULE.to_string(),
            description: Some("Add one sense deferral rule.".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "rule_id": { "type": "string", "minLength": 1 },
                    "min_weight": {
                        "type": "number", "minimum": 0, "maximum": 1,
                        "description": "Senses with a weight < this value will be deferred"
                    },
                    "fq_sense_id_pattern": {
                        "type": "string", "minLength": 1,
                        "description": "The senses matching this pattern will be deferred."
                    }
                },
                "required": ["rule_id"]
            }),
        },
        ChatToolDefinition {
            name: PRIMARY_TOOL_REMOVE_SENSE_DEFERRAL_RULE.to_string(),
            description: Some("Remove one sense deferral rule by rule_id.".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "rule_id": { "type": "string", "minLength": 1 }
                },
                "required": ["rule_id"],
            }),
        },
        ChatToolDefinition {
            name: PRIMARY_TOOL_SLEEP.to_string(),
            description: Some("Sleep for N ticks.".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "ticks": {
                        "type": "integer",
                        "minimum": 1
                    }
                },
                "required": ["ticks"]
            }),
        },
        ChatToolDefinition {
            name: PRIMARY_TOOL_PATCH_GOAL_FOREST.to_string(),
            description: Some("Patch the goal-forest as your will.".to_string()),
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
                "default": false,
                "description": "Reset to avoid context rot, the goal forest will maintains your cognition continuity"
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
