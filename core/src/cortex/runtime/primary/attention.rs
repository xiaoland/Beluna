use std::{sync::Arc, time::Instant};

use async_trait::async_trait;
use tokio::sync::Mutex;

use crate::{
    ai_gateway::chat::{
        ChatMessage, ChatRole, ContentPart, ContextControlReason, DeriveContextOptions, OutputMode,
        SystemPromptAction, Thread, ThreadContextRequest, ToolCallResult, ToolExecutionRequest,
        ToolExecutionResult, ToolExecutor, ToolOverride, TurnRetentionPolicy,
    },
    cortex::{
        error::{CortexError, extractor_failed},
        helpers::{self, CognitionOrgan},
        prompts,
    },
    observability::{contract::OrganResponseStatus, runtime as observability_runtime},
    stem::DeferralRuleAddInput,
};

use super::{
    Cortex, ai_gateway_request_id, alias_route_ref, build_turn_input, gateway_error_summary,
    map_organ_gateway_error, organ_thread_metadata, output_mode_label, tool_overrides_payload,
    tools::{PRIMARY_TOOL_BREAK_PRIMARY_PHASE, PRIMARY_TOOL_EXPAND_SENSES},
    turn_response_payload,
};

const ATTENTION_TOOL_REPLACE_AFFERENT_GATING: &str = "replace-afferent-gating";
const ATTENTION_TOOL_SLEEP: &str = "sleep";

#[derive(Debug, Clone, Default, serde::Serialize)]
pub(super) struct AttentionPhaseOutput {
    pub(super) gating_rules: Option<Vec<DeferralRuleAddInput>>,
    pub(super) sleep_ticks: Option<u64>,
}

#[derive(Debug, Default)]
struct AttentionToolState {
    output: AttentionPhaseOutput,
    protocol_violation: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct ReplaceAfferentGatingArgs {
    #[serde(default)]
    rules: Vec<DeferralRuleAddInput>,
}

#[derive(Debug, serde::Deserialize)]
struct SleepArgs {
    ticks: u64,
}

#[derive(Clone)]
struct AttentionToolExecutor {
    cortex: Cortex,
    cycle_id: u64,
    state: Arc<Mutex<AttentionToolState>>,
}

impl AttentionToolExecutor {
    fn new(cortex: Cortex, cycle_id: u64) -> Self {
        Self {
            cortex,
            cycle_id,
            state: Arc::new(Mutex::new(AttentionToolState::default())),
        }
    }

    async fn output(&self) -> Result<AttentionPhaseOutput, CortexError> {
        let state = self.state.lock().await;
        if let Some(message) = state.protocol_violation.as_ref() {
            return Err(extractor_failed(message.clone()));
        }
        Ok(state.output.clone())
    }

    async fn execute_attention_tool_call(&self, call: &ToolCallResult) -> serde_json::Value {
        let result = match call.name.as_str() {
            ATTENTION_TOOL_REPLACE_AFFERENT_GATING => {
                let parsed =
                    serde_json::from_str::<ReplaceAfferentGatingArgs>(&call.arguments_json)
                        .map_err(|err| err.to_string());
                match parsed {
                    Ok(args) => {
                        let mut state = self.state.lock().await;
                        if state.output.gating_rules.is_some() {
                            fail_closed(
                                &mut state,
                                "replace-afferent-gating was called more than once",
                            )
                        } else {
                            state.output.gating_rules = Some(args.rules);
                            Ok(serde_json::json!({ "gating_state": "replace_requested" }))
                        }
                    }
                    Err(err) => {
                        let mut state = self.state.lock().await;
                        fail_closed(&mut state, err)
                    }
                }
            }
            ATTENTION_TOOL_SLEEP => {
                let parsed = serde_json::from_str::<SleepArgs>(&call.arguments_json)
                    .map_err(|err| err.to_string());
                match parsed {
                    Ok(args) => {
                        let mut state = self.state.lock().await;
                        if state.output.sleep_ticks.is_some() {
                            fail_closed(&mut state, "sleep was called more than once")
                        } else if args.ticks == 0 {
                            fail_closed(&mut state, "ticks must be >= 1")
                        } else if args.ticks > self.cortex.limits.max_waiting_ticks {
                            fail_closed(
                                &mut state,
                                format!(
                                    "ticks must be <= {}",
                                    self.cortex.limits.max_waiting_ticks
                                ),
                            )
                        } else {
                            state.output.sleep_ticks = Some(args.ticks);
                            Ok(serde_json::json!({ "sleep_ticks": args.ticks }))
                        }
                    }
                    Err(err) => {
                        let mut state = self.state.lock().await;
                        fail_closed(&mut state, err)
                    }
                }
            }
            _ => Err(format!("unknown attention tool '{}'", call.name)),
        };

        match result {
            Ok(data) => serde_json::json!({
                "ok": true,
                "tool": call.name,
                "data": data,
            }),
            Err(error) => {
                tracing::warn!(
                    target: "cortex",
                    cycle_id = self.cycle_id,
                    tool_name = %call.name,
                    tool_call_id = %call.id,
                    error = %error,
                    "attention_tool_failed"
                );
                serde_json::json!({
                    "ok": false,
                    "tool": call.name,
                    "error": error,
                })
            }
        }
    }
}

#[async_trait]
impl ToolExecutor for AttentionToolExecutor {
    async fn execute_call(
        &self,
        request: ToolExecutionRequest,
    ) -> Result<ToolExecutionResult, crate::ai_gateway::error::GatewayError> {
        Ok(ToolExecutionResult {
            payload: self.execute_attention_tool_call(&request.call).await,
            reset_messages_applied: false,
        })
    }
}

impl Cortex {
    pub(super) async fn run_attention_phase(
        &self,
        cycle_id: u64,
        source_thread: &Thread,
    ) -> Result<AttentionPhaseOutput, CortexError> {
        let organ = CognitionOrgan::Attention;
        let stage = organ.stage();
        let request_id = format!("cortex-{stage}-{cycle_id}");
        let thread = self
            .derive_phase_thread(
                cycle_id,
                organ,
                source_thread,
                prompts::attention_system_prompt(),
                ContextControlReason::AttentionPhase,
                &request_id,
            )
            .await?;
        let tool_executor = Arc::new(AttentionToolExecutor::new(self.clone(), cycle_id));
        let user_prompt = prompts::attention_user_prompt();
        let tool_overrides = attention_tool_overrides(self.limits.max_waiting_ticks);
        let response = self
            .run_phase_tool_turn(
                cycle_id,
                organ,
                request_id,
                &thread,
                user_prompt,
                tool_overrides,
                Some(tool_executor.clone()),
            )
            .await?;
        let output = tool_executor.output().await?;
        helpers::log_organ_output(
            cycle_id,
            stage,
            &helpers::pretty_json(&serde_json::json!({
                "response_text": response.output_text.trim(),
                "output": output,
            })),
        );
        Ok(output)
    }

    pub(super) async fn derive_phase_thread(
        &self,
        cycle_id: u64,
        organ: CognitionOrgan,
        source_thread: &Thread,
        system_prompt: String,
        reason: ContextControlReason,
        parent_span_id: &str,
    ) -> Result<Thread, CortexError> {
        let chat = self.chat.as_ref().ok_or_else(|| {
            CortexError::new(
                crate::cortex::error::CortexErrorKind::Internal,
                "AI Gateway is not configured for this Cortex instance",
            )
        })?;
        let stage = organ.stage();
        let mut metadata = organ_thread_metadata(cycle_id, stage);
        metadata.insert("parent_span_id".to_string(), parent_span_id.to_string());
        let (thread, _) = chat
            .derive_context(
                source_thread,
                ThreadContextRequest {
                    retention: TurnRetentionPolicy::KeepAll,
                    system_prompt: SystemPromptAction::Replace {
                        prompt: system_prompt,
                    },
                    drop_unfinished_continuation: true,
                    reason,
                },
                DeriveContextOptions {
                    thread_id: Some(format!("cortex-{stage}-{cycle_id}-thread")),
                    route_ref: alias_route_ref(self.resolve_route(organ)),
                    metadata,
                },
            )
            .await
            .map_err(|err| map_organ_gateway_error(organ, err.to_string()))?;
        Ok(thread)
    }

    pub(super) async fn run_phase_tool_turn(
        &self,
        cycle_id: u64,
        organ: CognitionOrgan,
        request_id: String,
        thread: &Thread,
        user_prompt: String,
        tool_overrides: Vec<ToolOverride>,
        tool_executor: Option<Arc<dyn ToolExecutor>>,
    ) -> Result<crate::ai_gateway::chat::TurnResponse, CortexError> {
        let stage = organ.stage();
        let started_at = Instant::now();
        let route = self.resolve_route(organ);
        let route_or_organ = route.clone().unwrap_or_else(|| stage.to_string());
        let output_mode = OutputMode::Text;
        let output_mode_label = output_mode_label(&output_mode);
        observability_runtime::emit_cortex_organ_start(
            cycle_id,
            stage,
            Some(&route_or_organ),
            &request_id,
            serde_json::json!({
                "user_prompt": user_prompt,
                "tool_overrides": tool_overrides_payload(&tool_overrides),
                "max_output_tokens": self.limits.max_sub_output_tokens,
                "max_request_time_ms": self.limits.max_cycle_time_ms,
                "output_mode": output_mode_label,
            }),
        );
        let mut input = build_turn_input(
            cycle_id,
            request_id.clone(),
            self.limits.max_sub_output_tokens,
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
            tool_overrides,
            stage,
            output_mode,
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
}

fn fail_closed<T>(state: &mut AttentionToolState, message: impl Into<String>) -> Result<T, String> {
    let message = message.into();
    state.protocol_violation = Some(message.clone());
    Err(message)
}

fn attention_tool_overrides(max_waiting_ticks: u64) -> Vec<ToolOverride> {
    vec![
        ToolOverride::Remove(PRIMARY_TOOL_EXPAND_SENSES.to_string()),
        ToolOverride::Remove(PRIMARY_TOOL_BREAK_PRIMARY_PHASE.to_string()),
        ToolOverride::Set(crate::ai_gateway::chat::ChatToolDefinition {
            name: ATTENTION_TOOL_REPLACE_AFFERENT_GATING.to_string(),
            description: Some(
                concat!(
                    "Replace the complete afferent gating ruleset for future ticks. ",
                    "Omit this tool when the existing gating rules should remain unchanged."
                )
                .to_string(),
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "rules": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "rule_id": { "type": "string", "minLength": 1 },
                                "min_weight": {
                                    "type": "number",
                                    "minimum": 0,
                                    "maximum": 1
                                },
                                "fq_sense_id_pattern": {
                                    "type": "string",
                                    "minLength": 1
                                }
                            },
                            "required": ["rule_id"],
                            "additionalProperties": false
                        }
                    }
                },
                "required": ["rules"],
                "additionalProperties": false
            }),
        }),
        ToolOverride::Set(crate::ai_gateway::chat::ChatToolDefinition {
            name: ATTENTION_TOOL_SLEEP.to_string(),
            description: Some(
                "Skip future admitted ticks for this many ticks after the current tick completes."
                    .to_string(),
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "ticks": {
                        "type": "integer",
                        "minimum": 1,
                        "maximum": max_waiting_ticks
                    }
                },
                "required": ["ticks"],
                "additionalProperties": false
            }),
        }),
    ]
}
