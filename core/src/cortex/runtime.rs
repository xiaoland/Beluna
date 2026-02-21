use std::{
    collections::{BTreeMap, HashMap},
    future::Future,
    pin::Pin,
    sync::Arc,
    time::Instant,
};

use serde::Deserialize;
use tokio::{
    sync::RwLock,
    time::{Duration, timeout},
};

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
        clamp::derive_act_id,
        error::{CortexError, extractor_failed, filler_failed, invalid_input, primary_failed},
        helpers_input,
        helpers_output::{
            GoalStackHelperOutput, acts_json_schema, apply_goal_stack_patch,
            empty_goal_stack_patch, goal_stack_patch_json_schema,
        },
        ir,
        testing::{
            ActDescriptorHelperRequest as TestActDescriptorHelperRequest,
            ActsHelperRequest as TestActsHelperRequest, GoalStackHelperRequest as TestGoalRequest,
            PrimaryRequest as TestPrimaryRequest, SenseHelperRequest as TestSenseHelperRequest,
            TestGoalStackPatch, TestGoalStackPatchOp, TestHooks,
        },
        types::{ActsHelperOutput, CortexOutput, GoalStackPatch, GoalStackPatchOp, ReactionLimits},
    },
    types::{Act, CognitionState, NeuralSignalDescriptor, PhysicalState, Sense},
};

#[derive(Debug, Clone)]
pub enum CortexTelemetryEvent {
    ReactionStarted { cycle_id: u64 },
    StageFailed { cycle_id: u64, stage: &'static str },
    ReactionCompleted { cycle_id: u64, act_count: usize },
    NoopFallback { cycle_id: u64, reason: &'static str },
}

pub type CortexTelemetryHook = Arc<dyn Fn(CortexTelemetryEvent) + Send + Sync>;

type SenseSectionFuture = Pin<Box<dyn Future<Output = Result<String, CortexError>> + Send>>;
type ActDescriptorSectionFuture = Pin<Box<dyn Future<Output = Result<String, CortexError>> + Send>>;
type PrimaryFuture = Pin<Box<dyn Future<Output = Result<String, CortexError>> + Send>>;
type ActsHelperFuture = Pin<Box<dyn Future<Output = Result<ActsHelperOutput, CortexError>> + Send>>;
type GoalHelperFuture = Pin<Box<dyn Future<Output = Result<GoalStackPatch, CortexError>> + Send>>;

#[derive(Clone)]
enum CortexCollaborators {
    Gateway {
        gateway: Arc<AIGateway>,
        helper_routes: CortexHelperRoutesConfig,
        act_descriptor_cache: Arc<RwLock<HashMap<String, String>>>,
    },
    Hooks {
        hooks: TestHooks,
        act_descriptor_cache: Arc<RwLock<HashMap<String, String>>>,
    },
}

#[derive(Clone)]
pub struct Cortex {
    collaborators: CortexCollaborators,
    telemetry_hook: Option<CortexTelemetryHook>,
    limits: ReactionLimits,
}

#[derive(Clone, Copy)]
enum HelperStage {
    Primary,
    SenseHelper,
    ActDescriptorHelper,
    ActsHelper,
    GoalStackHelper,
}

impl Cortex {
    pub fn from_config(
        config: &crate::config::CortexRuntimeConfig,
        gateway: Arc<AIGateway>,
        telemetry_hook: Option<CortexTelemetryHook>,
    ) -> Self {
        Self {
            collaborators: CortexCollaborators::Gateway {
                gateway,
                helper_routes: config.helper_routes.clone(),
                act_descriptor_cache: Arc::new(RwLock::new(HashMap::new())),
            },
            telemetry_hook,
            limits: config.default_limits.clone(),
        }
    }

    pub(crate) fn for_test_with_hooks(hooks: TestHooks, limits: ReactionLimits) -> Self {
        Self {
            collaborators: CortexCollaborators::Hooks {
                hooks,
                act_descriptor_cache: Arc::new(RwLock::new(HashMap::new())),
            },
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
        if senses.is_empty() {
            return Err(invalid_input("sense batch cannot be empty"));
        }
        if senses.iter().any(|sense| matches!(sense, Sense::Sleep)) {
            return Err(invalid_input("sleep sense should not be sent to cortex"));
        }

        self.emit(CortexTelemetryEvent::ReactionStarted {
            cycle_id: physical_state.cycle_id,
        });

        let deadline = Duration::from_millis(self.limits.max_cycle_time_ms.max(1));
        let sense_descriptors = helpers_input::sense_descriptors(physical_state);
        let act_descriptors = helpers_input::act_descriptors(physical_state);

        let (sense_section_result, act_catalog_result) = tokio::join!(
            timeout(
                deadline,
                self.build_senses_section(
                    physical_state.cycle_id,
                    senses.to_vec(),
                    sense_descriptors.clone()
                )
            ),
            timeout(
                deadline,
                self.build_act_descriptor_catalog_section(
                    physical_state.cycle_id,
                    act_descriptors.clone()
                )
            )
        );

        let senses_section = self.resolve_input_helper_fallback(
            physical_state.cycle_id,
            "sense_helper",
            sense_section_result,
            helpers_input::fallback_senses_section(senses, &sense_descriptors),
        );
        let act_descriptor_catalog_section = self.resolve_input_helper_fallback(
            physical_state.cycle_id,
            "act_descriptor_helper",
            act_catalog_result,
            helpers_input::fallback_act_descriptor_catalog_section(&act_descriptors),
        );
        let goal_stack_section = helpers_input::goal_stack_section(cognition_state);
        let context_section = helpers_input::context_section(physical_state, cognition_state);

        let input_ir = ir::build_input_ir(
            &senses_section,
            &act_descriptor_catalog_section,
            &goal_stack_section,
            &context_section,
        );

        let primary_result = timeout(
            deadline,
            self.run_primary_helper(
                physical_state.cycle_id,
                senses.to_vec(),
                physical_state.clone(),
                cognition_state.clone(),
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

        let (output_ir, output_sections) = match ir::parse_output_ir(&primary_output) {
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

        let (acts_result, goal_patch_result) = tokio::join!(
            timeout(
                deadline,
                self.run_acts_helper(
                    physical_state.cycle_id,
                    output_ir.text.clone(),
                    output_sections.acts_section.clone(),
                    act_descriptors.clone()
                )
            ),
            timeout(
                deadline,
                self.run_goal_stack_helper(
                    physical_state.cycle_id,
                    output_ir.text.clone(),
                    output_sections.goal_stack_patch_section.clone(),
                    cognition_state.clone()
                )
            )
        );

        let acts = match acts_result {
            Ok(Ok(acts_helper_output)) => {
                self.make_acts(physical_state.cycle_id, acts_helper_output.acts)
            }
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

        let goal_stack_patch = match goal_patch_result {
            Ok(Ok(patch)) => patch,
            Ok(Err(err)) => {
                self.emit(CortexTelemetryEvent::StageFailed {
                    cycle_id: physical_state.cycle_id,
                    stage: "goal_stack_helper",
                });
                tracing::warn!(
                    target: "cortex",
                    cycle_id = physical_state.cycle_id,
                    error = %err,
                    "goal_stack_helper_failed_fallback_empty"
                );
                empty_goal_stack_patch()
            }
            Err(_) => {
                self.emit(CortexTelemetryEvent::StageFailed {
                    cycle_id: physical_state.cycle_id,
                    stage: "goal_stack_helper_timeout",
                });
                tracing::warn!(
                    target: "cortex",
                    cycle_id = physical_state.cycle_id,
                    deadline_ms = deadline.as_millis() as u64,
                    "goal_stack_helper_timeout_fallback_empty"
                );
                empty_goal_stack_patch()
            }
        };

        let new_cognition_state = apply_goal_stack_patch(cognition_state, &goal_stack_patch);
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
        })
    }

    async fn build_senses_section(
        &self,
        cycle_id: u64,
        senses: Vec<Sense>,
        sense_descriptors: Vec<NeuralSignalDescriptor>,
    ) -> Result<String, CortexError> {
        match &self.collaborators {
            CortexCollaborators::Gateway { gateway, .. } => {
                let prompt = format!(
                    "Generate the body for <senses> section in <input-ir>.\nReturn plain text body only.\nSenses:\n{}\nSense descriptors:\n{}",
                    serde_json::to_string_pretty(&senses).unwrap_or_else(|_| "[]".to_string()),
                    serde_json::to_string_pretty(&sense_descriptors)
                        .unwrap_or_else(|_| "[]".to_string()),
                );
                call_text_helper_via_gateway(
                    gateway,
                    self.resolve_route(HelperStage::SenseHelper),
                    cycle_id,
                    "sense_helper",
                    self.limits.max_sub_output_tokens,
                    self.limits.max_cycle_time_ms,
                    prompt,
                )
                .await
            }
            CortexCollaborators::Hooks { hooks, .. } => {
                (hooks.sense_helper)(TestSenseHelperRequest {
                    cycle_id,
                    senses,
                    sense_descriptors,
                })
                .await
            }
        }
    }

    async fn build_act_descriptor_catalog_section(
        &self,
        cycle_id: u64,
        act_descriptors: Vec<NeuralSignalDescriptor>,
    ) -> Result<String, CortexError> {
        let cache_key = helpers_input::act_descriptor_cache_key(&act_descriptors);
        if let Some(cached) = self.get_cached_act_descriptor_section(&cache_key).await {
            tracing::debug!(
                target: "cortex",
                cycle_id = cycle_id,
                cache_key = %cache_key,
                "act_descriptor_helper_cache_hit"
            );
            return Ok(cached);
        }

        let generated = match &self.collaborators {
            CortexCollaborators::Gateway { gateway, .. } => {
                let prompt = format!(
                    "Generate the body for <act-descriptor-catalog> section in <input-ir>.\nReturn plain text body only.\nAct descriptors:\n{}",
                    serde_json::to_string_pretty(&act_descriptors)
                        .unwrap_or_else(|_| "[]".to_string()),
                );
                call_text_helper_via_gateway(
                    gateway,
                    self.resolve_route(HelperStage::ActDescriptorHelper),
                    cycle_id,
                    "act_descriptor_helper",
                    self.limits.max_sub_output_tokens,
                    self.limits.max_cycle_time_ms,
                    prompt,
                )
                .await?
            }
            CortexCollaborators::Hooks { hooks, .. } => {
                (hooks.act_descriptor_helper)(TestActDescriptorHelperRequest {
                    cycle_id,
                    act_descriptors,
                })
                .await?
            }
        };

        if !generated.trim().is_empty() {
            self.cache_act_descriptor_section(cache_key, generated.clone())
                .await;
        }
        Ok(generated)
    }

    async fn run_primary_helper(
        &self,
        cycle_id: u64,
        senses: Vec<Sense>,
        physical_state: PhysicalState,
        cognition_state: CognitionState,
        input_ir: String,
    ) -> Result<String, CortexError> {
        match &self.collaborators {
            CortexCollaborators::Gateway { gateway, .. } => {
                let prompt = format!(
                    "Transform <input-ir> into <output-ir>.\nRules:\n1) Return valid XML-like text with exact root <output-ir>.\n2) Must include first-level sections <acts> and <goal-stack-patch>.\n3) Section body may contain XML or Markdown.\n4) Return only IR.\n\nInput IR:\n{}",
                    input_ir
                );
                call_text_helper_via_gateway(
                    gateway,
                    self.resolve_route(HelperStage::Primary),
                    cycle_id,
                    "primary",
                    self.limits.max_primary_output_tokens,
                    self.limits.max_cycle_time_ms,
                    prompt,
                )
                .await
            }
            CortexCollaborators::Hooks { hooks, .. } => {
                (hooks.primary)(TestPrimaryRequest {
                    cycle_id,
                    senses,
                    physical_state,
                    cognition_state,
                    input_ir,
                })
                .await
            }
        }
    }

    async fn run_acts_helper(
        &self,
        cycle_id: u64,
        output_ir: String,
        acts_section: String,
        act_descriptors: Vec<NeuralSignalDescriptor>,
    ) -> Result<ActsHelperOutput, CortexError> {
        match &self.collaborators {
            CortexCollaborators::Gateway { gateway, .. } => {
                let prompt = format!(
                    "Parse <acts> section from output IR and produce structured acts.\nAllowed act descriptors:\n{}\nOutput IR:\n{}\nActs section:\n{}",
                    serde_json::to_string_pretty(&act_descriptors)
                        .unwrap_or_else(|_| "[]".to_string()),
                    output_ir,
                    acts_section
                );
                let response = call_helper_via_gateway(
                    gateway,
                    self.resolve_route(HelperStage::ActsHelper),
                    cycle_id,
                    "acts_helper",
                    self.limits.max_sub_output_tokens,
                    self.limits.max_cycle_time_ms,
                    prompt,
                    OutputMode::JsonSchema {
                        name: "acts_helper_output".to_string(),
                        schema: acts_json_schema(),
                        strict: true,
                    },
                )
                .await?;
                let parsed = parse_json_output::<ActsHelperOutput>(&response.output_text)
                    .map_err(|err| extractor_failed(err.to_string()))?;
                Ok(parsed)
            }
            CortexCollaborators::Hooks { hooks, .. } => {
                let raw = (hooks.acts_helper)(TestActsHelperRequest {
                    cycle_id,
                    output_ir,
                    acts_section,
                })
                .await?;
                Ok(ActsHelperOutput {
                    acts: raw
                        .acts
                        .into_iter()
                        .map(|act| crate::cortex::types::ActDraft {
                            endpoint_id: act.endpoint_id,
                            neural_signal_descriptor_id: act.neural_signal_descriptor_id,
                            payload: act.payload,
                        })
                        .collect(),
                })
            }
        }
    }

    async fn run_goal_stack_helper(
        &self,
        cycle_id: u64,
        output_ir: String,
        goal_stack_patch_section: String,
        cognition_state: CognitionState,
    ) -> Result<GoalStackPatch, CortexError> {
        match &self.collaborators {
            CortexCollaborators::Gateway { gateway, .. } => {
                let prompt = format!(
                    "Parse <goal-stack-patch> section from output IR and produce goal stack patch ops.\nCurrent goal stack:\n{}\nOutput IR:\n{}\nGoal stack patch section:\n{}",
                    serde_json::to_string_pretty(&cognition_state.goal_stack)
                        .unwrap_or_else(|_| "[]".to_string()),
                    output_ir,
                    goal_stack_patch_section
                );
                let response = call_helper_via_gateway(
                    gateway,
                    self.resolve_route(HelperStage::GoalStackHelper),
                    cycle_id,
                    "goal_stack_helper",
                    self.limits.max_sub_output_tokens,
                    self.limits.max_cycle_time_ms,
                    prompt,
                    OutputMode::JsonSchema {
                        name: "goal_stack_patch_output".to_string(),
                        schema: goal_stack_patch_json_schema(),
                        strict: true,
                    },
                )
                .await?;
                let parsed = parse_json_output::<GoalStackHelperOutput>(&response.output_text)
                    .map_err(|err| filler_failed(err.to_string()))?;
                Ok(parsed.patch)
            }
            CortexCollaborators::Hooks { hooks, .. } => {
                let raw = (hooks.goal_stack_helper)(TestGoalRequest {
                    cycle_id,
                    output_ir,
                    goal_stack_patch_section,
                    cognition_state,
                })
                .await?;
                Ok(convert_test_patch(raw))
            }
        }
    }

    fn make_acts(&self, cycle_id: u64, drafts: Vec<crate::cortex::types::ActDraft>) -> Vec<Act> {
        let mut acts = Vec::with_capacity(drafts.len());
        for draft in drafts {
            let act_id = derive_act_id(
                cycle_id,
                &[],
                &draft.endpoint_id,
                &draft.neural_signal_descriptor_id,
                &draft.payload,
            );
            acts.push(Act {
                act_id,
                endpoint_id: draft.endpoint_id,
                neural_signal_descriptor_id: draft.neural_signal_descriptor_id,
                payload: draft.payload,
            });
        }
        acts
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
        }
    }

    fn resolve_route(&self, stage: HelperStage) -> Option<String> {
        let routes = match &self.collaborators {
            CortexCollaborators::Gateway { helper_routes, .. } => helper_routes,
            CortexCollaborators::Hooks { .. } => return None,
        };

        let stage_route = match stage {
            HelperStage::Primary => routes.primary.clone(),
            HelperStage::SenseHelper => routes.sense_helper.clone(),
            HelperStage::ActDescriptorHelper => routes.act_descriptor_helper.clone(),
            HelperStage::ActsHelper => routes.acts_helper.clone(),
            HelperStage::GoalStackHelper => routes.goal_stack_helper.clone(),
        };
        stage_route.or_else(|| routes.default.clone())
    }

    async fn get_cached_act_descriptor_section(&self, cache_key: &str) -> Option<String> {
        match &self.collaborators {
            CortexCollaborators::Gateway {
                act_descriptor_cache,
                ..
            }
            | CortexCollaborators::Hooks {
                act_descriptor_cache,
                ..
            } => act_descriptor_cache.read().await.get(cache_key).cloned(),
        }
    }

    async fn cache_act_descriptor_section(&self, cache_key: String, value: String) {
        match &self.collaborators {
            CortexCollaborators::Gateway {
                act_descriptor_cache,
                ..
            }
            | CortexCollaborators::Hooks {
                act_descriptor_cache,
                ..
            } => {
                act_descriptor_cache.write().await.insert(cache_key, value);
            }
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

fn convert_test_patch(raw: TestGoalStackPatch) -> GoalStackPatch {
    GoalStackPatch {
        ops: raw
            .ops
            .into_iter()
            .map(|op| match op {
                TestGoalStackPatchOp::Push { goal_id, summary } => {
                    GoalStackPatchOp::Push { goal_id, summary }
                }
                TestGoalStackPatchOp::Pop => GoalStackPatchOp::Pop,
                TestGoalStackPatchOp::ReplaceTop { goal_id, summary } => {
                    GoalStackPatchOp::ReplaceTop { goal_id, summary }
                }
                TestGoalStackPatchOp::Clear => GoalStackPatchOp::Clear,
            })
            .collect(),
    }
}

async fn call_text_helper_via_gateway(
    gateway: &AIGateway,
    route: Option<String>,
    cycle_id: u64,
    stage: &'static str,
    max_output_tokens: u64,
    max_request_time_ms: u64,
    prompt: String,
) -> Result<String, CortexError> {
    let response = call_helper_via_gateway(
        gateway,
        route,
        cycle_id,
        stage,
        max_output_tokens,
        max_request_time_ms,
        prompt,
        OutputMode::Text,
    )
    .await?;

    let text = response.output_text.trim().to_string();
    if text.is_empty() {
        return Err(extractor_failed(format!("{stage} produced empty output")));
    }
    Ok(text)
}

async fn call_helper_via_gateway(
    gateway: &AIGateway,
    route: Option<String>,
    cycle_id: u64,
    stage: &'static str,
    max_output_tokens: u64,
    max_request_time_ms: u64,
    prompt: String,
    output_mode: OutputMode,
) -> Result<ChatResponse, CortexError> {
    let request_id = format!("cortex-{stage}-{cycle_id}");
    let started_at = Instant::now();
    let request = build_request(
        request_id.clone(),
        route,
        max_output_tokens,
        max_request_time_ms,
        prompt,
        stage,
        output_mode,
    );
    log_llm_input(stage, cycle_id, &request);
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
        match stage {
            "primary" => primary_failed(err.to_string()),
            "goal_stack_helper" => filler_failed(err.to_string()),
            _ => extractor_failed(err.to_string()),
        }
    })?;
    log_llm_output(stage, cycle_id, &request_id, &response);
    Ok(response)
}

fn build_request(
    request_id: String,
    route: Option<String>,
    max_output_tokens: u64,
    max_request_time_ms: u64,
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
                    text: "You are a Cortex helper. Return only what is asked.".to_string(),
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

fn log_llm_input(stage: &str, cycle_id: u64, request: &ChatRequest) {
    let request_json = serde_json::to_string_pretty(request)
        .unwrap_or_else(|err| format!("{{\"serialization_error\":\"{}\"}}", err));
    tracing::debug!(
        target: "cortex",
        stage = stage,
        cycle_id = cycle_id,
        request_id = request.request_id.as_deref().unwrap_or("-"),
        route = request.route.as_deref().unwrap_or("-"),
        output_mode = ?request.output_mode,
        max_output_tokens = ?request.limits.max_output_tokens,
        max_request_time_ms = ?request.limits.max_request_time_ms,
        request_json = %request_json,
        "llm_input"
    );
}

fn log_llm_output(stage: &str, cycle_id: u64, request_id: &str, response: &ChatResponse) {
    let (input_tokens, output_tokens, total_tokens) = response
        .usage
        .as_ref()
        .map(|u| (u.input_tokens, u.output_tokens, u.total_tokens))
        .unwrap_or((None, None, None));
    tracing::debug!(
        target: "cortex",
        stage = stage,
        cycle_id = cycle_id,
        request_id = request_id,
        finish_reason = ?response.finish_reason,
        usage_input_tokens = ?input_tokens,
        usage_output_tokens = ?output_tokens,
        usage_total_tokens = ?total_tokens,
        tool_calls = response.tool_calls.len(),
        output_chars = response.output_text.len(),
        output_text = %response.output_text,
        "llm_output"
    );
}

fn parse_json_output<T: for<'a> Deserialize<'a>>(text: &str) -> Result<T, CortexError> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err(CortexError::new(
            crate::cortex::error::CortexErrorKind::Internal,
            "empty JSON output",
        ));
    }

    if let Ok(parsed) = serde_json::from_str::<T>(trimmed) {
        return Ok(parsed);
    }

    if let Some(stripped) = strip_code_fence(trimmed)
        && let Ok(parsed) = serde_json::from_str::<T>(&stripped)
    {
        return Ok(parsed);
    }

    Err(CortexError::new(
        crate::cortex::error::CortexErrorKind::Internal,
        "failed to parse JSON output",
    ))
}

fn strip_code_fence(text: &str) -> Option<String> {
    let text = text.trim();
    if !text.starts_with("```") {
        return None;
    }

    let mut lines = text.lines();
    let _first = lines.next()?;
    let mut body = Vec::new();
    for line in lines {
        if line.trim_start().starts_with("```") {
            break;
        }
        body.push(line);
    }
    Some(body.join("\n"))
}
