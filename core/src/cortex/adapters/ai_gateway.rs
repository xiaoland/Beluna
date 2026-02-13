use std::{collections::BTreeMap, sync::Arc};

use async_trait::async_trait;
use serde::Deserialize;

use crate::{
    ai_gateway::{
        gateway::AIGateway,
        types::{
            BelunaContentPart, BelunaInferenceRequest, BelunaMessage, BelunaRole, OutputMode,
            RequestLimitOverrides, ToolChoice,
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
        let request = build_text_request(
            format!("cortex-primary-{}", req.reaction_id),
            self.backend_id.clone(),
            self.model.clone(),
            req.limits.max_primary_output_tokens,
            req.limits.max_cycle_time_ms,
            build_primary_prompt(&req),
            None,
            OutputMode::Text,
        );
        let response = self
            .gateway
            .infer_once(request)
            .await
            .map_err(|err| primary_failed(err.to_string()))?;
        let text = response.output_text.trim().to_string();
        if text.is_empty() {
            return Err(primary_failed("primary reasoner produced empty IR"));
        }
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
                    "commitment_hint": "optional string",
                    "goal_hint": "optional string"
                }
            ]
        });
        let prompt = format!(
            "Compile this prose IR into attempt drafts JSON.\nReturn strictly one JSON object matching this shape: {}\nAllowed endpoint capabilities: {}\nIR: {}",
            schema_hint,
            serde_json::to_string(&req.capability_catalog.affordances)
                .unwrap_or_else(|_| "[]".to_string()),
            req.prose_ir.text
        );

        let request = build_text_request(
            format!("cortex-extractor-{}", req.reaction_id),
            self.backend_id.clone(),
            self.model.clone(),
            req.limits.max_sub_output_tokens,
            req.limits.max_cycle_time_ms,
            prompt,
            Some("extractor".to_string()),
            OutputMode::JsonObject,
        );
        let response = self
            .gateway
            .infer_once(request)
            .await
            .map_err(|err| extractor_failed(err.to_string()))?;

        let parsed = parse_json_output::<AttemptDraftEnvelope>(&response.output_text)
            .map_err(|err| extractor_failed(err.to_string()))?;
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
        let request = build_text_request(
            format!("cortex-filler-{}", req.reaction_id),
            self.backend_id.clone(),
            self.model.clone(),
            req.limits.max_sub_output_tokens,
            req.limits.max_cycle_time_ms,
            prompt,
            Some("filler".to_string()),
            OutputMode::JsonObject,
        );
        let response = self
            .gateway
            .infer_once(request)
            .await
            .map_err(|err| filler_failed(err.to_string()))?;
        let parsed = parse_json_output::<AttemptDraftEnvelope>(&response.output_text)
            .map_err(|err| filler_failed(err.to_string()))?;
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

fn build_primary_prompt(req: &PrimaryReasonerRequest) -> String {
    format!(
        "Generate prose IR describing intent, attention, and action sketches.\nContext: {}\nSense IDs: {}",
        req.prompt_context,
        req.sense_window
            .iter()
            .map(|sense| sense.sense_id.as_str())
            .collect::<Vec<_>>()
            .join(",")
    )
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
