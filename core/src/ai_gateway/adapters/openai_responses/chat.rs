use std::time::Duration;

use async_trait::async_trait;
use serde_json::Value;

use crate::ai_gateway::{
    adapters::{
        BackendAdapter,
        http_stream::{self, HttpRequestConfig},
    },
    chat::types::{
        AdapterInvocation, BackendCompleteResponse, BackendIdentity, FinishReason, ToolCallResult,
        ToolCallStatus, TurnPayload, UsageStats,
    },
    error::{GatewayError, GatewayErrorKind},
    types::{AdapterContext, BackendCapabilities, BackendDialect},
};

use super::wire as responses_wire;

#[derive(Clone)]
pub struct OpenAiResponsesAdapter {
    client: reqwest::Client,
}

impl Default for OpenAiResponsesAdapter {
    fn default() -> Self {
        Self {
            client: reqwest::Client::builder()
                .pool_idle_timeout(Duration::from_secs(30))
                .build()
                .expect("reqwest client must build"),
        }
    }
}

fn default_responses_capabilities() -> BackendCapabilities {
    BackendCapabilities {
        streaming: false,
        tool_calls: true,
        parallel_tool_calls: true,
        json_mode: true,
        json_schema_mode: true,
        vision: false,
        resumable_streaming: false,
    }
}

fn parallel_tool_calls_enabled(profile_capabilities: Option<&BackendCapabilities>) -> bool {
    profile_capabilities
        .map(|capabilities| capabilities.parallel_tool_calls)
        .unwrap_or_else(|| default_responses_capabilities().parallel_tool_calls)
}

#[async_trait]
impl BackendAdapter for OpenAiResponsesAdapter {
    fn dialect(&self) -> BackendDialect {
        BackendDialect::OpenAiResponses
    }

    fn static_capabilities(&self) -> BackendCapabilities {
        default_responses_capabilities()
    }

    async fn complete(
        &self,
        ctx: AdapterContext,
        payload: &TurnPayload,
    ) -> Result<BackendCompleteResponse, GatewayError> {
        let url = validated_url(&ctx)?;
        let backend_id = ctx.backend_id.clone();
        let allow_parallel_tool_calls =
            parallel_tool_calls_enabled(ctx.profile.capabilities.as_ref());
        let body =
            responses_wire::build_request_body(&ctx.model, payload, allow_parallel_tool_calls)
                .map_err(|err| err.with_backend_id(backend_id.clone()))?;

        let json_response = http_stream::post_json(&HttpRequestConfig {
            client: self.client.clone(),
            url,
            body,
            backend_id: backend_id.clone(),
            request_id: ctx.request_id.clone(),
            credential: ctx.credential,
            timeout: ctx.timeout,
        })
        .await?;

        parse_complete_response(&json_response, &backend_id, &ctx.model)
    }

    async fn stream(
        &self,
        _ctx: AdapterContext,
        _payload: &TurnPayload,
    ) -> Result<AdapterInvocation, GatewayError> {
        Err(GatewayError::new(
            GatewayErrorKind::UnsupportedCapability,
            "openai-responses adapter does not implement chat stream",
        )
        .with_retryable(false))
    }
}

fn validated_url(ctx: &AdapterContext) -> Result<String, GatewayError> {
    let endpoint = ctx.profile.endpoint.clone().ok_or_else(|| {
        GatewayError::new(
            GatewayErrorKind::InvalidRequest,
            "openai-responses backend requires endpoint",
        )
        .with_retryable(false)
        .with_backend_id(ctx.backend_id.clone())
    })?;
    Ok(format!("{}/responses", endpoint.trim_end_matches('/')))
}

fn parse_complete_response(
    payload: &Value,
    backend_id: &str,
    model: &str,
) -> Result<BackendCompleteResponse, GatewayError> {
    if let Some(error) = payload.get("error").filter(|value| !value.is_null()) {
        return Err(GatewayError::new(
            GatewayErrorKind::BackendPermanent,
            format!("openai responses returned error: {}", error),
        )
        .with_retryable(false)
        .with_backend_id(backend_id.to_string()));
    }

    let output_items = payload
        .get("output")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut output_text = parse_output_text(&output_items);
    if output_text.is_empty() {
        output_text = payload
            .get("output_text")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
    }
    let tool_calls = parse_tool_calls(&output_items);

    if output_items.is_empty() && output_text.is_empty() {
        return Err(GatewayError::new(
            GatewayErrorKind::ProtocolViolation,
            "openai responses payload missing output",
        )
        .with_retryable(false)
        .with_backend_id(backend_id.to_string()));
    }

    let usage = payload.get("usage").map(parse_usage);
    let finish_reason = parse_finish_reason(payload, !tool_calls.is_empty());

    Ok(BackendCompleteResponse {
        backend_identity: BackendIdentity {
            backend_id: backend_id.to_string(),
            dialect: BackendDialect::OpenAiResponses,
            model: model.to_string(),
        },
        output_text,
        tool_calls,
        usage,
        finish_reason,
    })
}

fn parse_output_text(output_items: &[Value]) -> String {
    output_items
        .iter()
        .filter(|item| item.get("type").and_then(Value::as_str) == Some("message"))
        .flat_map(|item| {
            item.get("content")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
        })
        .filter_map(|content| {
            let content_type = content.get("type").and_then(Value::as_str);
            match content_type {
                Some("output_text") | Some("text") => content
                    .get("text")
                    .and_then(Value::as_str)
                    .map(str::to_string),
                _ => None,
            }
        })
        .collect::<Vec<_>>()
        .join("")
}

fn parse_tool_calls(output_items: &[Value]) -> Vec<ToolCallResult> {
    output_items
        .iter()
        .filter(|item| item.get("type").and_then(Value::as_str) == Some("function_call"))
        .enumerate()
        .map(|(index, item)| {
            let id = item
                .get("call_id")
                .and_then(Value::as_str)
                .or_else(|| item.get("id").and_then(Value::as_str))
                .map(str::to_string)
                .unwrap_or_else(|| format!("call_{index}"));
            let name = item
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("unknown_tool")
                .to_string();
            let arguments_json = item
                .get("arguments")
                .map(|value| {
                    value
                        .as_str()
                        .map(str::to_string)
                        .unwrap_or_else(|| value.to_string())
                })
                .unwrap_or_else(|| "{}".to_string());

            ToolCallResult {
                id,
                name,
                arguments_json,
                status: ToolCallStatus::Ready,
            }
        })
        .collect()
}

fn parse_finish_reason(payload: &Value, has_tool_calls: bool) -> FinishReason {
    if has_tool_calls {
        return FinishReason::ToolCalls;
    }

    match payload.get("status").and_then(Value::as_str) {
        Some("completed") => FinishReason::Stop,
        Some("incomplete") => {
            let reason = payload
                .get("incomplete_details")
                .and_then(|details| details.get("reason"))
                .and_then(Value::as_str);
            match reason {
                Some("max_output_tokens") => FinishReason::Length,
                Some(other) => FinishReason::Other(other.to_string()),
                None => FinishReason::Other("incomplete".to_string()),
            }
        }
        Some(other) => FinishReason::Other(other.to_string()),
        None => FinishReason::Stop,
    }
}

fn parse_usage(usage: &Value) -> UsageStats {
    UsageStats {
        input_tokens: usage.get("input_tokens").and_then(Value::as_u64),
        output_tokens: usage.get("output_tokens").and_then(Value::as_u64),
        total_tokens: usage.get("total_tokens").and_then(Value::as_u64),
        provider_usage_raw: Some(usage.clone()),
    }
}
