use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use async_trait::async_trait;
use futures_util::StreamExt;
use serde_json::{Value, json};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tracing::Instrument;

use crate::ai_gateway::{
    adapters::{
        BackendAdapter,
        http_stream::{self, HttpRequestConfig},
        wire,
    },
    chat::types::{
        AdapterInvocation, BackendCompleteResponse, BackendIdentity, BackendRawEvent, FinishReason,
        OutputMode, ToolCallResult, ToolCallStatus, TurnPayload, UsageStats,
    },
    error::{GatewayError, GatewayErrorKind},
    types::{AdapterContext, BackendCapabilities, BackendDialect},
};

use super::wire as openai_wire;

// ---------------------------------------------------------------------------
// Adapter
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct OpenAiCompatibleAdapter {
    client: reqwest::Client,
}

impl Default for OpenAiCompatibleAdapter {
    fn default() -> Self {
        Self {
            client: reqwest::Client::builder()
                .pool_idle_timeout(Duration::from_secs(30))
                .build()
                .expect("reqwest client must build"),
        }
    }
}

#[async_trait]
impl BackendAdapter for OpenAiCompatibleAdapter {
    fn dialect(&self) -> BackendDialect {
        BackendDialect::OpenAiCompatible
    }

    fn static_capabilities(&self) -> BackendCapabilities {
        BackendCapabilities {
            streaming: true,
            tool_calls: true,
            json_mode: true,
            json_schema_mode: true,
            vision: false,
            resumable_streaming: false,
        }
    }

    async fn complete(
        &self,
        ctx: AdapterContext,
        payload: &TurnPayload,
    ) -> Result<BackendCompleteResponse, GatewayError> {
        let url = validated_url(&ctx)?;
        let backend_id = ctx.backend_id.clone();

        let body = build_body(&ctx.model, payload, false);
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
        ctx: AdapterContext,
        payload: &TurnPayload,
    ) -> Result<AdapterInvocation, GatewayError> {
        let url = validated_url(&ctx)?;
        let cancel_flag = Arc::new(AtomicBool::new(false));
        let cancel_flag_task = cancel_flag.clone();

        let (tx, rx) = mpsc::channel::<Result<BackendRawEvent, GatewayError>>(64);
        let backend_id = ctx.backend_id.clone();
        let model = ctx.model.clone();
        let request_id = ctx.request_id.clone();
        let dispatch_span = tracing::debug_span!(
            target: "ai_gateway.openai_compatible",
            "openai_dispatch",
            request_id = %request_id,
            backend_id = %backend_id,
            model = %model,
        );

        let body = build_body(&model, payload, true);
        let http_config = HttpRequestConfig {
            client: self.client.clone(),
            url,
            body,
            backend_id: backend_id.clone(),
            request_id,
            credential: ctx.credential.clone(),
            timeout: ctx.timeout,
        };

        tokio::spawn(
            async move {
                let response = match http_stream::send_post(&http_config).await {
                    Ok(r) => r,
                    Err(err) => {
                        let _ = tx.send(Err(err)).await;
                        return;
                    }
                };

                let mut byte_stream = response.bytes_stream();
                let mut buffer = String::new();
                let mut saw_terminal = false;

                while let Some(item) = byte_stream.next().await {
                    if cancel_flag_task.load(Ordering::SeqCst) {
                        return;
                    }

                    let chunk = match item {
                        Ok(chunk) => chunk,
                        Err(err) => {
                            let _ = tx
                                .send(Err(GatewayError::new(
                                    GatewayErrorKind::BackendTransient,
                                    format!("openai-compatible stream chunk error: {}", err),
                                )
                                .with_retryable(true)
                                .with_backend_id(backend_id.clone())))
                                .await;
                            return;
                        }
                    };

                    buffer.push_str(&String::from_utf8_lossy(&chunk));
                    let (frames, done) =
                        match http_stream::extract_sse_frames(&mut buffer, &backend_id) {
                            Ok(result) => result,
                            Err(err) => {
                                let _ = tx.send(Err(err)).await;
                                return;
                            }
                        };

                    for json in frames {
                        match parse_stream_delta(&json, &backend_id) {
                            Ok(events) => {
                                for event in events {
                                    if matches!(event, BackendRawEvent::Completed { .. }) {
                                        saw_terminal = true;
                                    }
                                    if tx.send(Ok(event)).await.is_err() {
                                        return;
                                    }
                                }
                            }
                            Err(err) => {
                                let _ = tx.send(Err(err)).await;
                                return;
                            }
                        }
                    }

                    if done {
                        if !saw_terminal {
                            let _ = tx
                                .send(Ok(BackendRawEvent::Completed {
                                    finish_reason: FinishReason::Stop,
                                }))
                                .await;
                        }
                        return;
                    }
                }

                if !saw_terminal {
                    let _ = tx
                        .send(Ok(BackendRawEvent::Completed {
                            finish_reason: FinishReason::Stop,
                        }))
                        .await;
                }
            }
            .instrument(dispatch_span),
        );

        let cancel = {
            let cancel_flag = cancel_flag.clone();
            Arc::new(move || {
                cancel_flag.store(true, Ordering::SeqCst);
            })
        };

        Ok(AdapterInvocation {
            stream: Box::pin(ReceiverStream::new(rx)),
            backend_identity: BackendIdentity {
                backend_id: ctx.backend_id,
                dialect: BackendDialect::OpenAiCompatible,
                model: ctx.model,
            },
            cancel: Some(cancel),
        })
    }
}

// ---------------------------------------------------------------------------
// Helpers — URL / body
// ---------------------------------------------------------------------------

fn validated_url(ctx: &AdapterContext) -> Result<String, GatewayError> {
    let endpoint = ctx.profile.endpoint.clone().ok_or_else(|| {
        GatewayError::new(
            GatewayErrorKind::InvalidRequest,
            "openai-compatible backend requires endpoint",
        )
        .with_retryable(false)
        .with_backend_id(ctx.backend_id.clone())
    })?;
    Ok(format!(
        "{}/chat/completions",
        endpoint.trim_end_matches('/')
    ))
}

fn build_body(model: &str, payload: &TurnPayload, stream: bool) -> Value {
    let mut body = json!({
        "model": model,
        "messages": openai_wire::messages_to_openai(&payload.messages),
        "stream": stream,
    });

    if !payload.tools.is_empty() {
        body["tools"] = Value::Array(openai_wire::tools_to_openai(&payload.tools));
        body["tool_choice"] = Value::String("auto".to_string());
    }

    match &payload.output_mode {
        OutputMode::JsonObject => {
            body["response_format"] = json!({"type": "json_object"});
        }
        OutputMode::JsonSchema {
            name,
            schema,
            strict,
        } => {
            body["response_format"] = json!({
                "type": "json_schema",
                "json_schema": {
                    "name": name,
                    "schema": schema,
                    "strict": strict
                }
            });
        }
        OutputMode::Text => {}
    }

    if let Some(max_tokens) = payload.limits.max_output_tokens {
        body["max_tokens"] = Value::Number(max_tokens.into());
    }

    if payload.enable_thinking {
        body["thinking"] = json!({
            "type": "enabled",
            "budget_tokens": payload.limits.max_output_tokens.unwrap_or(10000)
        });
        body["extra_body"] = json!({
            "enable_thinking": "enabled",
            "thinking_budget": "payload.limits.max_output_tokens.unwrap_or(10000)"
        });
        body["enable_thinking"] = json!(true);
    } else {
        body["thinking"] = json!({
            "type": "disabled",
        });
        body["enable_thinking"] = json!(false);
    }

    body
}

// ---------------------------------------------------------------------------
// Response parsing — complete (non-stream)
// ---------------------------------------------------------------------------

fn parse_complete_response(
    payload: &Value,
    backend_id: &str,
    model: &str,
) -> Result<BackendCompleteResponse, GatewayError> {
    let choice = payload
        .get("choices")
        .and_then(Value::as_array)
        .and_then(|choices| choices.first())
        .ok_or_else(|| {
            GatewayError::new(
                GatewayErrorKind::ProtocolViolation,
                "openai-compatible response missing choices",
            )
            .with_retryable(false)
            .with_backend_id(backend_id.to_string())
        })?;

    let output_text = choice
        .get("message")
        .and_then(|m| m.get("content"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();

    let tool_calls = parse_tool_calls_from_message(choice.get("message"));

    let usage = payload.get("usage").map(parse_usage);

    let finish_reason =
        wire::parse_finish_reason(choice.get("finish_reason").and_then(Value::as_str));

    Ok(BackendCompleteResponse {
        backend_identity: BackendIdentity {
            backend_id: backend_id.to_string(),
            dialect: BackendDialect::OpenAiCompatible,
            model: model.to_string(),
        },
        output_text,
        tool_calls,
        usage,
        finish_reason,
    })
}

fn parse_tool_calls_from_message(message: Option<&Value>) -> Vec<ToolCallResult> {
    let Some(tool_calls) = message
        .and_then(|m| m.get("tool_calls"))
        .and_then(Value::as_array)
    else {
        return Vec::new();
    };

    tool_calls
        .iter()
        .map(|tc| {
            let id = tc
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or("call_0")
                .to_string();
            let name = tc
                .get("function")
                .and_then(|f| f.get("name"))
                .and_then(Value::as_str)
                .unwrap_or("unknown_tool")
                .to_string();
            let arguments_json = tc
                .get("function")
                .and_then(|f| f.get("arguments"))
                .map(|v| {
                    if let Some(s) = v.as_str() {
                        s.to_string()
                    } else {
                        v.to_string()
                    }
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

// ---------------------------------------------------------------------------
// Response parsing — stream
// ---------------------------------------------------------------------------

fn parse_stream_delta(
    payload: &Value,
    backend_id: &str,
) -> Result<Vec<BackendRawEvent>, GatewayError> {
    let mut events = Vec::new();

    if let Some(usage) = payload.get("usage") {
        events.push(BackendRawEvent::Usage {
            usage: parse_usage(usage),
        });
    }

    let choices = payload
        .get("choices")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            GatewayError::new(
                GatewayErrorKind::ProtocolViolation,
                "openai-compatible stream payload missing choices",
            )
            .with_retryable(false)
            .with_backend_id(backend_id.to_string())
        })?;

    for choice in choices {
        if let Some(delta) = choice.get("delta") {
            if let Some(content) = delta.get("content").and_then(Value::as_str) {
                if !content.is_empty() {
                    events.push(BackendRawEvent::OutputTextDelta {
                        delta: content.to_string(),
                    });
                }
            }

            if let Some(tool_calls) = delta.get("tool_calls").and_then(Value::as_array) {
                for (index, tool_call) in tool_calls.iter().enumerate() {
                    let call_id = tool_call
                        .get("id")
                        .and_then(Value::as_str)
                        .map(str::to_string)
                        .unwrap_or_else(|| format!("call_{}", index));
                    let name = tool_call
                        .get("function")
                        .and_then(|f| f.get("name"))
                        .and_then(Value::as_str)
                        .map(str::to_string);
                    let arguments_delta = tool_call
                        .get("function")
                        .and_then(|f| f.get("arguments"))
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_string();

                    events.push(BackendRawEvent::ToolCallDelta {
                        call_id,
                        name,
                        arguments_delta,
                    });
                }
            }
        }

        let finish_reason = choice.get("finish_reason").and_then(Value::as_str);
        if finish_reason.is_some() {
            events.push(BackendRawEvent::Completed {
                finish_reason: wire::parse_finish_reason(finish_reason),
            });
        }
    }

    Ok(events)
}

// ---------------------------------------------------------------------------
// Usage parsing
// ---------------------------------------------------------------------------

fn parse_usage(usage: &Value) -> UsageStats {
    UsageStats {
        input_tokens: usage
            .get("prompt_tokens")
            .and_then(Value::as_u64)
            .or_else(|| usage.get("input_tokens").and_then(Value::as_u64)),
        output_tokens: usage
            .get("completion_tokens")
            .and_then(Value::as_u64)
            .or_else(|| usage.get("output_tokens").and_then(Value::as_u64)),
        total_tokens: usage.get("total_tokens").and_then(Value::as_u64),
        provider_usage_raw: Some(usage.clone()),
    }
}
