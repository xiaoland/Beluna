use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use tokio::sync::Mutex;

use crate::{
    ai_gateway::chat::{
        Thread, ToolCallResult, ToolExecutionRequest, ToolExecutionResult, ToolExecutor, TurnQuery,
    },
    cortex::{
        clamp::derive_act_instance_id,
        helpers::{
            goal_forest_helper::{GoalNode, goal_forest_ascii, goal_forest_empty_one_shot},
            sense_input_helper,
        },
        prompts,
        types::WaitForSenseControlDirective,
    },
    observability::runtime as observability_runtime,
    spine::ActDispatchResult,
    stem::DeferralRuleAddInput,
    types::{Act, build_fq_neural_signal_id},
};

use super::{
    ActToolBinding, Cortex, PrimaryTurnState,
    tools::{
        ActToolArgs, AddSenseDeferralRuleArgs, ExpandSenseTask,
        PRIMARY_TOOL_ADD_SENSE_DEFERRAL_RULE, PRIMARY_TOOL_BREAK_PRIMARY_PHASE,
        PRIMARY_TOOL_EXPAND_SENSES, PRIMARY_TOOL_PATCH_GOAL_FOREST,
        PRIMARY_TOOL_REMOVE_SENSE_DEFERRAL_RULE, PRIMARY_TOOL_SLEEP, RemoveSenseDeferralRuleArgs,
        SleepArgs, parse_patch_goal_forest_args,
    },
};

#[derive(Clone)]
pub(super) struct PrimaryToolExecutor {
    cortex: Cortex,
    cycle_id: u64,
    step: u64,
    thread: Thread,
    sense_tool_context: sense_input_helper::SenseToolContext,
    act_binding_map: HashMap<String, ActToolBinding>,
    goal_forest_nodes: Arc<Mutex<Vec<GoalNode>>>,
    turn_state: Arc<Mutex<PrimaryTurnState>>,
}

#[derive(Debug, Clone)]
struct PrimaryToolCallResult {
    payload: serde_json::Value,
    reset_messages_applied: bool,
}

impl PrimaryToolExecutor {
    pub(super) fn new(
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

    pub(super) async fn turn_state(&self) -> PrimaryTurnState {
        self.turn_state.lock().await.clone()
    }

    pub(super) async fn goal_forest_nodes(&self) -> Vec<GoalNode> {
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
                PRIMARY_TOOL_BREAK_PRIMARY_PHASE => {
                    let parsed = serde_json::from_str::<serde_json::Value>(&call.arguments_json)
                        .map_err(|err| err.to_string());
                    match parsed {
                        Ok(serde_json::Value::Object(map)) if map.is_empty() => {
                            let mut state = self.turn_state.lock().await;
                            if state.break_primary_phase_requested {
                                let error =
                                    "break-primary-phase was called more than once".to_string();
                                state.protocol_violation = Some(error.clone());
                                Err(error)
                            } else {
                                state.break_primary_phase_requested = true;
                                Ok((
                                    serde_json::json!({
                                        "primary_phase": "break_requested"
                                    }),
                                    false,
                                ))
                            }
                        }
                        Ok(_) => Err("break-primary-phase expects an empty object".to_string()),
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

fn render_goal_forest_section(goal_forest_nodes: &[GoalNode]) -> String {
    if goal_forest_nodes.is_empty() {
        goal_forest_empty_one_shot().to_string()
    } else {
        goal_forest_ascii(goal_forest_nodes)
    }
}
