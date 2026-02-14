use std::{collections::BTreeMap, sync::Arc, time::Instant};

use async_trait::async_trait;
use serde::Deserialize;

use crate::{
    ai_gateway::{
        gateway::AIGateway,
        telemetry::debug_log,
        types::{
            BelunaContentPart, BelunaInferenceRequest, BelunaMessage, BelunaRole,
            CanonicalFinalResponse, OutputMode, RequestLimitOverrides, ToolChoice,
        },
    },
    cortex::{
        error::{CortexError, extractor_failed, filler_failed, primary_failed},
        ports::{
            AttemptExtractorPort, AttemptExtractorRequest, PayloadFillerPort, PayloadFillerRequest,
            PrimaryReasonerPort, PrimaryReasonerRequest,
        },
        types::{AttemptDraft, ProseIr},
    },
};

#[derive(Clone)]
pub struct AIGatewayPrimaryReasoner {
    gateway: Arc<AIGateway>,
    backend_id: Option<String>,
    model: Option<String>,
}

#[derive(Clone)]
pub struct AIGatewayAttemptExtractor {
    gateway: Arc<AIGateway>,
    backend_id: Option<String>,
    model: Option<String>,
}

#[derive(Clone)]
pub struct AIGatewayPayloadFiller {
    gateway: Arc<AIGateway>,
    backend_id: Option<String>,
    model: Option<String>,
}

impl AIGatewayPrimaryReasoner {
    pub fn new(gateway: Arc<AIGateway>, backend_id: Option<String>, model: Option<String>) -> Self {
        Self {
            gateway,
            backend_id,
            model,
        }
    }
}

impl AIGatewayAttemptExtractor {
    pub fn new(gateway: Arc<AIGateway>, backend_id: Option<String>, model: Option<String>) -> Self {
        Self {
            gateway,
            backend_id,
            model,
        }
    }
}

impl AIGatewayPayloadFiller {
    pub fn new(gateway: Arc<AIGateway>, backend_id: Option<String>, model: Option<String>) -> Self {
        Self {
            gateway,
            backend_id,
            model,
        }
    }
}

#[async_trait]
impl PrimaryReasonerPort for AIGatewayPrimaryReasoner {
    async fn infer_ir(&self, req: PrimaryReasonerRequest) -> Result<ProseIr, CortexError> {
        let helper_request_id = format!("cortex-helper-{}", req.cycle_id);
        let request_id = format!("cortex-primary-{}", req.cycle_id);
        let started_at = Instant::now();
        let helper_timeout_ms = (req.limits.max_cycle_time_ms / 2).max(1);
        let primary_timeout_ms = req
            .limits
            .max_cycle_time_ms
            .saturating_sub(helper_timeout_ms)
            .max(1);
        debug_log(format!(
            "cortex_primary_start cycle_id={} helper_request_id={} request_id={} backend_hint={} model_hint={} helper_timeout_ms={} primary_timeout_ms={} max_output_tokens={}",
            req.cycle_id,
            helper_request_id,
            request_id,
            self.backend_id.as_deref().unwrap_or("-"),
            self.model.as_deref().unwrap_or("-"),
            helper_timeout_ms,
            primary_timeout_ms,
            req.limits.max_primary_output_tokens,
        ));

        let helper_request = build_text_request(
            helper_request_id.clone(),
            self.backend_id.clone(),
            self.model.clone(),
            req.limits.max_primary_output_tokens,
            helper_timeout_ms,
            build_helper_prompt(&req),
            Some("helper".to_string()),
            OutputMode::Text,
        );
        log_llm_input("helper", req.cycle_id, &helper_request);
        let helper_response = self
            .gateway
            .infer_once(helper_request)
            .await
            .map_err(|err| {
                debug_log(format!(
                    "cortex_helper_failed cycle_id={} request_id={} elapsed_ms={} error_kind={:?} error={}",
                    req.cycle_id,
                    helper_request_id,
                    started_at.elapsed().as_millis(),
                    err.kind,
                    err.message,
                ));
                primary_failed(err.to_string())
            })?;
        log_llm_output("helper", req.cycle_id, &helper_request_id, &helper_response);
        let helper_text = helper_response.output_text.trim().to_string();
        if helper_text.is_empty() {
            return Err(primary_failed("helper produced empty context"));
        }

        let request = build_text_request(
            request_id.clone(),
            self.backend_id.clone(),
            self.model.clone(),
            req.limits.max_primary_output_tokens,
            primary_timeout_ms,
            build_primary_prompt(&helper_text),
            Some("primary".to_string()),
            OutputMode::Text,
        );
        log_llm_input("primary", req.cycle_id, &request);
        let response = self
            .gateway
            .infer_once(request)
            .await
            .map_err(|err| {
                debug_log(format!(
                    "cortex_primary_failed cycle_id={} request_id={} elapsed_ms={} error_kind={:?} error={}",
                    req.cycle_id,
                    request_id,
                    started_at.elapsed().as_millis(),
                    err.kind,
                    err.message,
                ));
                primary_failed(err.to_string())
            })?;
        log_llm_output("primary", req.cycle_id, &request_id, &response);
        let finish_reason = response.finish_reason;
        let text = response.output_text.trim().to_string();
        if text.is_empty() {
            debug_log(format!(
                "cortex_primary_empty cycle_id={} request_id={} elapsed_ms={}",
                req.cycle_id,
                request_id,
                started_at.elapsed().as_millis(),
            ));
            return Err(primary_failed("primary reasoner produced empty IR"));
        }
        debug_log(format!(
            "cortex_primary_completed cycle_id={} request_id={} elapsed_ms={} finish_reason={:?} output_chars={}",
            req.cycle_id,
            request_id,
            started_at.elapsed().as_millis(),
            finish_reason,
            text.len(),
        ));
        Ok(ProseIr { text })
    }
}

#[derive(Debug, Deserialize)]
struct AttemptDraftEnvelope {
    attempts: Vec<AttemptDraft>,
}

#[async_trait]
impl AttemptExtractorPort for AIGatewayAttemptExtractor {
    async fn extract(
        &self,
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
            self.backend_id.clone(),
            self.model.clone(),
            req.limits.max_sub_output_tokens,
            req.limits.max_cycle_time_ms,
            prompt,
            Some("extractor".to_string()),
            OutputMode::JsonObject,
        );
        log_llm_input("extractor", req.cycle_id, &request);
        let response = self
            .gateway
            .infer_once(request)
            .await
            .map_err(|err| {
                debug_log(format!(
                    "llm_call_failed stage=extractor cycle_id={} request_id={} error_kind={:?} error={}",
                    req.cycle_id, request_id, err.kind, err.message
                ));
                extractor_failed(err.to_string())
            })?;
        log_llm_output("extractor", req.cycle_id, &request_id, &response);

        let parsed = parse_json_output::<AttemptDraftEnvelope>(&response.output_text)
            .map_err(|err| {
                debug_log(format!(
                    "llm_output_parse_failed stage=extractor cycle_id={} request_id={} error={} raw_output=\n{}",
                    req.cycle_id, request_id, err, response.output_text
                ));
                extractor_failed(err.to_string())
            })?;
        Ok(parsed.attempts)
    }
}

#[async_trait]
impl PayloadFillerPort for AIGatewayPayloadFiller {
    async fn fill(&self, req: PayloadFillerRequest) -> Result<Vec<AttemptDraft>, CortexError> {
        let prompt = format!(
            "Repair the attempt drafts to satisfy violations.\nReturn JSON object {{\"attempts\": [...]}} with same count as input.\nDrafts: {}\nViolations: {}",
            serde_json::to_string(&req.drafts).unwrap_or_else(|_| "[]".to_string()),
            serde_json::to_string(&req.clamp_violations).unwrap_or_else(|_| "[]".to_string()),
        );
        let request_id = format!("cortex-filler-{}", req.cycle_id);
        let request = build_text_request(
            request_id.clone(),
            self.backend_id.clone(),
            self.model.clone(),
            req.limits.max_sub_output_tokens,
            req.limits.max_cycle_time_ms,
            prompt,
            Some("filler".to_string()),
            OutputMode::JsonObject,
        );
        log_llm_input("filler", req.cycle_id, &request);
        let response = self.gateway.infer_once(request).await.map_err(|err| {
            debug_log(format!(
                "llm_call_failed stage=filler cycle_id={} request_id={} error_kind={:?} error={}",
                req.cycle_id, request_id, err.kind, err.message
            ));
            filler_failed(err.to_string())
        })?;
        log_llm_output("filler", req.cycle_id, &request_id, &response);
        let parsed = parse_json_output::<AttemptDraftEnvelope>(&response.output_text)
            .map_err(|err| {
                debug_log(format!(
                    "llm_output_parse_failed stage=filler cycle_id={} request_id={} error={} raw_output=\n{}",
                    req.cycle_id, request_id, err, response.output_text
                ));
                filler_failed(err.to_string())
            })?;
        Ok(parsed.attempts)
    }
}

fn build_text_request(
    request_id: String,
    backend_id: Option<String>,
    model: Option<String>,
    max_output_tokens: u64,
    max_request_time_ms: u64,
    user_prompt: String,
    stage: Option<String>,
    output_mode: OutputMode,
) -> BelunaInferenceRequest {
    let mut metadata = BTreeMap::new();
    if let Some(stage) = stage {
        metadata.insert("cortex_stage".to_string(), stage);
    }
    BelunaInferenceRequest {
        request_id: Some(request_id),
        backend_id,
        model,
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
        stream: false,
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

fn log_llm_input(stage: &str, cycle_id: u64, request: &BelunaInferenceRequest) {
    let request_json = serde_json::to_string_pretty(request)
        .unwrap_or_else(|err| format!("{{\"serialization_error\":\"{}\"}}", err));
    debug_log(format!(
        "llm_input stage={} cycle_id={} request_id={} backend_id={} model={} output_mode={:?} limits={{max_output_tokens:{:?},max_request_time_ms:{:?}}}\n{}",
        stage,
        cycle_id,
        request.request_id.as_deref().unwrap_or("-"),
        request.backend_id.as_deref().unwrap_or("-"),
        request.model.as_deref().unwrap_or("-"),
        request.output_mode,
        request.limits.max_output_tokens,
        request.limits.max_request_time_ms,
        request_json
    ));
}

fn log_llm_output(stage: &str, cycle_id: u64, request_id: &str, response: &CanonicalFinalResponse) {
    let usage = response
        .usage
        .as_ref()
        .map(|u| {
            format!(
                "input={:?},output={:?},total={:?}",
                u.input_tokens, u.output_tokens, u.total_tokens
            )
        })
        .unwrap_or_else(|| "none".to_string());
    debug_log(format!(
        "llm_output stage={} cycle_id={} request_id={} finish_reason={:?} usage={} tool_calls={} output_chars={}\n{}",
        stage,
        cycle_id,
        request_id,
        response.finish_reason,
        usage,
        response.tool_calls.len(),
        response.output_text.len(),
        response.output_text
    ));
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

    if let Some(stripped) = strip_code_fence(trimmed) {
        if let Ok(parsed) = serde_json::from_str::<T>(&stripped) {
            return Ok(parsed);
        }
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
