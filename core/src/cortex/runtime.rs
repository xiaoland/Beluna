use std::{collections::BTreeMap, future::Future, pin::Pin, sync::Arc, time::Instant};

use serde::Deserialize;
use tokio::time::{Duration, timeout};

use crate::{
    ai_gateway::{
        gateway::AIGateway,
        types_chat::{
            BelunaContentPart, BelunaMessage, BelunaRole, ChatRequest, ChatResponse, OutputMode,
            RequestLimitOverrides, ToolChoice,
        },
    },
    cortex::{
        clamp::derive_act_id,
        error::{
            CortexError, budget_exceeded, cycle_timeout, extractor_failed, invalid_input,
            primary_failed,
        },
        types::{AttemptDraft, CortexOutput, ProseIr, ReactionLimits},
    },
    types::{Act, CognitionState, GoalFrame, PhysicalState, RequestedResources, Sense},
};

#[derive(Debug, Clone)]
pub struct PrimaryReasonerRequest {
    pub cycle_id: u64,
    pub senses: Vec<Sense>,
    pub physical_state: PhysicalState,
    pub cognition_state: CognitionState,
    pub limits: ReactionLimits,
}

#[derive(Debug, Clone)]
pub struct AttemptExtractorRequest {
    pub cycle_id: u64,
    pub prose_ir: ProseIr,
    pub capability_catalog: crate::cortex::CapabilityCatalog,
    pub senses: Vec<Sense>,
    pub cognition_state: CognitionState,
    pub limits: ReactionLimits,
}

#[derive(Debug, Clone)]
pub enum CortexTelemetryEvent {
    ReactionStarted { cycle_id: u64 },
    StageFailed { cycle_id: u64, stage: &'static str },
    ReactionCompleted { cycle_id: u64, act_count: usize },
    NoopFallback { cycle_id: u64, reason: &'static str },
}

pub type CortexTelemetryHook = Arc<dyn Fn(CortexTelemetryEvent) + Send + Sync>;

type PrimaryReasonerFuture = Pin<Box<dyn Future<Output = Result<ProseIr, CortexError>> + Send>>;
type AttemptExtractorFuture =
    Pin<Box<dyn Future<Output = Result<Vec<AttemptDraft>, CortexError>> + Send>>;

pub type PrimaryReasonerHook =
    Arc<dyn Fn(PrimaryReasonerRequest) -> PrimaryReasonerFuture + Send + Sync>;
pub type AttemptExtractorHook =
    Arc<dyn Fn(AttemptExtractorRequest) -> AttemptExtractorFuture + Send + Sync>;

#[derive(Clone)]
enum CortexCollaborators {
    Gateway {
        gateway: Arc<AIGateway>,
        primary_route: Option<String>,
        sub_route: Option<String>,
    },
    Hooks {
        primary: PrimaryReasonerHook,
        extractor: AttemptExtractorHook,
    },
}

#[derive(Clone)]
pub struct Cortex {
    collaborators: CortexCollaborators,
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
            collaborators: CortexCollaborators::Gateway {
                gateway,
                primary_route: config.primary_route.clone(),
                sub_route: config.sub_route.clone(),
            },
            telemetry_hook,
            limits: config.default_limits.clone(),
        }
    }

    pub fn for_test_with_hooks(
        primary: PrimaryReasonerHook,
        extractor: AttemptExtractorHook,
        limits: ReactionLimits,
    ) -> Self {
        Self {
            collaborators: CortexCollaborators::Hooks { primary, extractor },
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

        if let Err(err) = validate_input_bounds(&self.limits) {
            self.emit(CortexTelemetryEvent::StageFailed {
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
        let ir = match timeout(deadline, self.infer_ir(primary_req)).await {
            Ok(Ok(ir)) => ir,
            Ok(Err(err)) => {
                self.emit(CortexTelemetryEvent::StageFailed {
                    cycle_id: physical_state.cycle_id,
                    stage: "primary",
                });
                return Err(err);
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
                    max_cycle_time_ms = self.limits.max_cycle_time_ms,
                    "primary_timeout"
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
        let drafts = match timeout(deadline, self.extract_attempts(extract_req)).await {
            Ok(Ok(drafts)) => drafts,
            Ok(Err(err)) => {
                self.emit(CortexTelemetryEvent::StageFailed {
                    cycle_id: physical_state.cycle_id,
                    stage: "extractor",
                });
                return Err(err);
            }
            Err(_) => {
                self.emit(CortexTelemetryEvent::StageFailed {
                    cycle_id: physical_state.cycle_id,
                    stage: "extractor_timeout",
                });
                tracing::warn!(
                    target: "cortex",
                    cycle_id = physical_state.cycle_id,
                    deadline_ms = deadline.as_millis() as u64,
                    max_cycle_time_ms = self.limits.max_cycle_time_ms,
                    "extractor_timeout"
                );
                return Err(cycle_timeout("extractor call timed out"));
            }
        };

        let acts = drafts_to_acts(physical_state.cycle_id, drafts, &self.limits);
        tracing::debug!(
            target: "cortex",
            cycle_id = physical_state.cycle_id,
            act_count = acts.len(),
            "drafts_to_acts"
        );
        if acts.is_empty() {
            self.emit(CortexTelemetryEvent::NoopFallback {
                cycle_id: physical_state.cycle_id,
                reason: "extractor_no_drafts",
            });
        }

        let new_cognition_state = evolve_cognition_state(cognition_state, senses);
        self.emit(CortexTelemetryEvent::ReactionCompleted {
            cycle_id: physical_state.cycle_id,
            act_count: acts.len(),
        });

        Ok(CortexOutput {
            acts,
            new_cognition_state,
        })
    }

    async fn infer_ir(&self, req: PrimaryReasonerRequest) -> Result<ProseIr, CortexError> {
        match &self.collaborators {
            CortexCollaborators::Gateway {
                gateway,
                primary_route,
                ..
            } => infer_ir_via_gateway(gateway, primary_route.clone(), req).await,
            CortexCollaborators::Hooks { primary, .. } => (primary)(req).await,
        }
    }

    async fn extract_attempts(
        &self,
        req: AttemptExtractorRequest,
    ) -> Result<Vec<AttemptDraft>, CortexError> {
        match &self.collaborators {
            CortexCollaborators::Gateway {
                gateway, sub_route, ..
            } => extract_attempts_via_gateway(gateway, sub_route.clone(), req).await,
            CortexCollaborators::Hooks { extractor, .. } => (extractor)(req).await,
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

fn drafts_to_acts(cycle_id: u64, drafts: Vec<AttemptDraft>, limits: &ReactionLimits) -> Vec<Act> {
    let mut acts = Vec::with_capacity(drafts.len());
    for draft in drafts {
        let AttemptDraft {
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

#[derive(Debug, Deserialize)]
struct AttemptDraftEnvelope {
    attempts: Vec<AttemptDraft>,
}

async fn infer_ir_via_gateway(
    gateway: &AIGateway,
    route: Option<String>,
    req: PrimaryReasonerRequest,
) -> Result<ProseIr, CortexError> {
    let helper_request_id = format!("cortex-helper-{}", req.cycle_id);
    let request_id = format!("cortex-primary-{}", req.cycle_id);
    let started_at = Instant::now();
    let helper_timeout_ms = (req.limits.max_cycle_time_ms / 2).max(1);
    let primary_timeout_ms = req
        .limits
        .max_cycle_time_ms
        .saturating_sub(helper_timeout_ms)
        .max(1);
    tracing::debug!(
        target: "cortex",
        cycle_id = req.cycle_id,
        helper_request_id = %helper_request_id,
        request_id = %request_id,
        route_hint = route.as_deref().unwrap_or("-"),
        helper_timeout_ms = helper_timeout_ms,
        primary_timeout_ms = primary_timeout_ms,
        max_output_tokens = req.limits.max_primary_output_tokens,
        "cortex_primary_start"
    );

    let helper_request = build_text_request(
        helper_request_id.clone(),
        route.clone(),
        req.limits.max_primary_output_tokens,
        helper_timeout_ms,
        build_helper_prompt(&req),
        Some("helper".to_string()),
        OutputMode::Text,
    );
    log_llm_input("helper", req.cycle_id, &helper_request);
    let helper_response = gateway.chat_once(helper_request).await.map_err(|err| {
        tracing::debug!(
            target: "cortex",
            cycle_id = req.cycle_id,
            request_id = %helper_request_id,
            elapsed_ms = started_at.elapsed().as_millis() as u64,
            error_kind = ?err.kind,
            error = %err.message,
            "cortex_helper_failed"
        );
        primary_failed(err.to_string())
    })?;
    log_llm_output("helper", req.cycle_id, &helper_request_id, &helper_response);
    let helper_text = helper_response.output_text.trim().to_string();
    if helper_text.is_empty() {
        return Err(primary_failed("helper produced empty context"));
    }

    let request = build_text_request(
        request_id.clone(),
        route,
        req.limits.max_primary_output_tokens,
        primary_timeout_ms,
        build_primary_prompt(&helper_text),
        Some("primary".to_string()),
        OutputMode::Text,
    );
    log_llm_input("primary", req.cycle_id, &request);
    let response = gateway.chat_once(request).await.map_err(|err| {
        tracing::debug!(
            target: "cortex",
            cycle_id = req.cycle_id,
            request_id = %request_id,
            elapsed_ms = started_at.elapsed().as_millis() as u64,
            error_kind = ?err.kind,
            error = %err.message,
            "cortex_primary_failed"
        );
        primary_failed(err.to_string())
    })?;
    log_llm_output("primary", req.cycle_id, &request_id, &response);
    let finish_reason = response.finish_reason;
    let text = response.output_text.trim().to_string();
    if text.is_empty() {
        tracing::debug!(
            target: "cortex",
            cycle_id = req.cycle_id,
            request_id = %request_id,
            elapsed_ms = started_at.elapsed().as_millis() as u64,
            "cortex_primary_empty"
        );
        return Err(primary_failed("primary reasoner produced empty IR"));
    }
    tracing::debug!(
        target: "cortex",
        cycle_id = req.cycle_id,
        request_id = %request_id,
        elapsed_ms = started_at.elapsed().as_millis() as u64,
        finish_reason = ?finish_reason,
        output_chars = text.len(),
        "cortex_primary_completed"
    );
    Ok(ProseIr { text })
}

async fn extract_attempts_via_gateway(
    gateway: &AIGateway,
    route: Option<String>,
    req: AttemptExtractorRequest,
) -> Result<Vec<AttemptDraft>, CortexError> {
    let schema_hint = serde_json::json!({
        "attempts": [
            {
                "intent_span": "string",
                "based_on": ["sense_id"],
                "attention_tags": ["tag"],
                "endpoint_id": "string",
                "capability_id": "string",
                "capability_instance_id": "optional string marker",
                "payload_draft": {},
                "requested_resources": {
                    "survival_micro": 0,
                    "time_ms": 0,
                    "io_units": 0,
                    "token_units": 0
                },
                "goal_hint": "optional string"
            }
        ]
    });
    let prompt = format!(
        "Compile this prose IR into attempt drafts JSON.\nReturn strictly one JSON object matching this shape: {}\nAllowed endpoint capabilities: {}\nSenses: {}\nCognition: {}\nIR: {}",
        schema_hint,
        serde_json::to_string(&req.capability_catalog.affordances)
            .unwrap_or_else(|_| "[]".to_string()),
        serde_json::to_string(&req.senses).unwrap_or_else(|_| "[]".to_string()),
        serde_json::to_string(&req.cognition_state).unwrap_or_else(|_| "{}".to_string()),
        req.prose_ir.text
    );

    let request_id = format!("cortex-extractor-{}", req.cycle_id);
    let request = build_text_request(
        request_id.clone(),
        route,
        req.limits.max_sub_output_tokens,
        req.limits.max_cycle_time_ms,
        prompt,
        Some("extractor".to_string()),
        OutputMode::JsonObject,
    );
    log_llm_input("extractor", req.cycle_id, &request);
    let response = gateway.chat_once(request).await.map_err(|err| {
        tracing::debug!(
            target: "cortex",
            stage = "extractor",
            cycle_id = req.cycle_id,
            request_id = %request_id,
            error_kind = ?err.kind,
            error = %err.message,
            "llm_call_failed"
        );
        extractor_failed(err.to_string())
    })?;
    log_llm_output("extractor", req.cycle_id, &request_id, &response);

    let parsed =
        parse_json_output::<AttemptDraftEnvelope>(&response.output_text).map_err(|err| {
            tracing::debug!(
                target: "cortex",
                stage = "extractor",
                cycle_id = req.cycle_id,
                request_id = %request_id,
                error = %err,
                raw_output = %response.output_text,
                "llm_output_parse_failed"
            );
            extractor_failed(err.to_string())
        })?;
    Ok(parsed.attempts)
}

fn build_text_request(
    request_id: String,
    route: Option<String>,
    max_output_tokens: u64,
    max_request_time_ms: u64,
    user_prompt: String,
    stage: Option<String>,
    output_mode: OutputMode,
) -> ChatRequest {
    let mut metadata = BTreeMap::new();
    if let Some(stage) = stage {
        metadata.insert("cortex_stage".to_string(), stage);
    }
    ChatRequest {
        request_id: Some(request_id),
        route,
        messages: vec![
            BelunaMessage {
                role: BelunaRole::System,
                parts: vec![BelunaContentPart::Text {
                    text: "You are a Cortex cognition organ. Return only what is asked."
                        .to_string(),
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

fn build_primary_prompt(helper_context: &str) -> String {
    format!(
        "Given helper context, produce prose IR only.\nThe prose IR must describe intent, attention, and action sketches.\nDo not include JSON, markdown, or explanations.\nHelper context:\n{}",
        helper_context
    )
}

fn build_helper_prompt(req: &PrimaryReasonerRequest) -> String {
    format!(
        "Translate runtime context into concise natural-language cognition notes for the primary reasoner.\nPreserve all sense_id values exactly as provided.\nOutput plain text only.\nSenses: {}\nPhysical state: {}\nCognition state: {}",
        serde_json::to_string(&req.senses).unwrap_or_else(|_| "[]".to_string()),
        serde_json::to_string(&req.physical_state).unwrap_or_else(|_| "{}".to_string()),
        serde_json::to_string(&req.cognition_state).unwrap_or_else(|_| "{}".to_string()),
    )
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
