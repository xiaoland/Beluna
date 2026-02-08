use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use async_trait::async_trait;
use futures_util::StreamExt;
use reqwest::{Client, header};
use serde_json::{Value, json};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

use crate::ai_gateway::{
    adapters::{BackendAdapter, http_common},
    error::{GatewayError, GatewayErrorKind},
    types::{
        AdapterContext, AdapterInvocation, BackendCapabilities, BackendDialect, BackendIdentity,
        BackendRawEvent, CanonicalOutputMode, CanonicalRequest, CanonicalToolCall, ToolCallStatus,
        UsageStats,
    },
};

#[derive(Clone)]
pub struct OpenAiCompatibleAdapter {
    client: Client,
}

impl Default for OpenAiCompatibleAdapter {
    fn default() -> Self {
        Self {
            client: Client::builder()
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
            vision: false,
            resumable_streaming: false,
        }
    }

    async fn invoke_stream(
        &self,
        ctx: AdapterContext,
        req: CanonicalRequest,
    ) -> Result<AdapterInvocation, GatewayError> {
        let endpoint = ctx.profile.endpoint.clone().ok_or_else(|| {
            GatewayError::new(
                GatewayErrorKind::InvalidRequest,
                "openai-compatible backend requires endpoint",
            )
            .with_retryable(false)
            .with_backend_id(ctx.backend_id.clone())
        })?;

        let url = format!("{}/chat/completions", endpoint.trim_end_matches('/'));
        let cancel_flag = Arc::new(AtomicBool::new(false));
        let cancel_flag_task = cancel_flag.clone();

        let (tx, rx) = mpsc::channel::<Result<BackendRawEvent, GatewayError>>(64);
        let client = self.client.clone();
        let backend_id = ctx.backend_id.clone();
        let model = ctx.model.clone();
        let request_id = ctx.request_id.clone();
        let credential = ctx.credential.clone();

        tokio::spawn(async move {
            let mut body = json!({
                "model": model,
                "messages": http_common::canonical_messages_to_openai(&req.messages),
                "stream": req.stream,
            });

            if !req.tools.is_empty() {
                body["tools"] = Value::Array(http_common::tools_to_openai(&req.tools));
                body["tool_choice"] = http_common::tool_choice_to_openai(&req.tool_choice);
            }

            if matches!(req.output_mode, CanonicalOutputMode::JsonObject) {
                body["response_format"] = json!({"type": "json_object"});
            }

            if let Some(max_tokens) = req.limits.max_output_tokens {
                body["max_tokens"] = Value::Number(max_tokens.into());
            }

            let mut req_builder = client
                .post(url)
                .timeout(ctx.timeout)
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-request-id", request_id)
                .json(&body);

            if let Some(auth_header) = credential.auth_header {
                req_builder = req_builder.header(header::AUTHORIZATION, auth_header);
            }
            for (k, v) in credential.extra_headers {
                req_builder = req_builder.header(k, v);
            }

            let response = match req_builder.send().await {
                Ok(response) => response,
                Err(err) => {
                    let _ = tx
                        .send(Err(GatewayError::new(
                            GatewayErrorKind::BackendTransient,
                            format!("openai-compatible request failed: {}", err),
                        )
                        .with_retryable(true)
                        .with_backend_id(backend_id)))
                        .await;
                    return;
                }
            };

            if !response.status().is_success() {
                let status = response.status().as_u16();
                let body = response.text().await.unwrap_or_default();
                let _ = tx
                    .send(Err(http_common::map_http_error(status, &backend_id, &body)))
                    .await;
                return;
            }

            if req.stream {
                let mut stream = response.bytes_stream();
                let mut buffer = String::new();
                let mut saw_terminal = false;

                while let Some(item) = stream.next().await {
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
                    while let Some(idx) = buffer.find('\n') {
                        let line = buffer[..idx].trim_end_matches('\r').to_string();
                        buffer = buffer[idx + 1..].to_string();

                        if !line.starts_with("data:") {
                            continue;
                        }

                        let data = line[5..].trim();
                        if data.is_empty() {
                            continue;
                        }
                        if data == "[DONE]" {
                            if !saw_terminal {
                                let _ = tx
                                    .send(Ok(BackendRawEvent::Completed {
                                        finish_reason: crate::ai_gateway::types::FinishReason::Stop,
                                    }))
                                    .await;
                            }
                            return;
                        }

                        let parsed = match serde_json::from_str::<Value>(data) {
                            Ok(parsed) => parsed,
                            Err(err) => {
                                let _ = tx
                                    .send(Err(GatewayError::new(
                                        GatewayErrorKind::ProtocolViolation,
                                        format!(
                                            "failed to parse openai-compatible SSE payload: {}",
                                            err
                                        ),
                                    )
                                    .with_retryable(false)
                                    .with_backend_id(backend_id.clone())))
                                    .await;
                                return;
                            }
                        };

                        match parse_stream_payload(&parsed, &backend_id) {
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
                }

                if !saw_terminal {
                    let _ = tx
                        .send(Ok(BackendRawEvent::Completed {
                            finish_reason: crate::ai_gateway::types::FinishReason::Stop,
                        }))
                        .await;
                }
                return;
            }

            let payload = match response.json::<Value>().await {
                Ok(payload) => payload,
                Err(err) => {
                    let _ = tx
                        .send(Err(GatewayError::new(
                            GatewayErrorKind::ProtocolViolation,
                            format!("openai-compatible body decode failed: {}", err),
                        )
                        .with_retryable(false)
                        .with_backend_id(backend_id.clone())))
                        .await;
                    return;
                }
            };

            match parse_non_stream_payload(&payload, &backend_id) {
                Ok(events) => {
                    for event in events {
                        if tx.send(Ok(event)).await.is_err() {
                            return;
                        }
                    }
                }
                Err(err) => {
                    let _ = tx.send(Err(err)).await;
                }
            }
        });

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

fn parse_stream_payload(
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
                finish_reason: http_common::parse_finish_reason(finish_reason),
            });
        }
    }

    Ok(events)
}

fn parse_non_stream_payload(
    payload: &Value,
    backend_id: &str,
) -> Result<Vec<BackendRawEvent>, GatewayError> {
    let mut events = Vec::new();

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

    if let Some(content) = choice
        .get("message")
        .and_then(|message| message.get("content"))
        .and_then(Value::as_str)
    {
        if !content.is_empty() {
            events.push(BackendRawEvent::OutputTextDelta {
                delta: content.to_string(),
            });
        }
    }

    if let Some(tool_calls) = choice
        .get("message")
        .and_then(|message| message.get("tool_calls"))
        .and_then(Value::as_array)
    {
        for tool_call in tool_calls {
            let id = tool_call
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or("call_0")
                .to_string();
            let name = tool_call
                .get("function")
                .and_then(|f| f.get("name"))
                .and_then(Value::as_str)
                .unwrap_or("unknown_tool")
                .to_string();
            let arguments_json = tool_call
                .get("function")
                .and_then(|f| f.get("arguments"))
                .map(|v| {
                    if let Some(arguments) = v.as_str() {
                        arguments.to_string()
                    } else {
                        v.to_string()
                    }
                })
                .unwrap_or_else(|| "{}".to_string());

            events.push(BackendRawEvent::ToolCallReady {
                call: CanonicalToolCall {
                    id,
                    name,
                    arguments_json,
                    status: ToolCallStatus::Ready,
                },
            });
        }
    }

    if let Some(usage) = payload.get("usage") {
        events.push(BackendRawEvent::Usage {
            usage: parse_usage(usage),
        });
    }

    let finish_reason = choice.get("finish_reason").and_then(Value::as_str);
    events.push(BackendRawEvent::Completed {
        finish_reason: http_common::parse_finish_reason(finish_reason),
    });

    Ok(events)
}

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
