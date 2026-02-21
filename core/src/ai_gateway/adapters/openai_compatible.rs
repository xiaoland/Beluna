use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

use async_trait::async_trait;
use futures_util::StreamExt;
use reqwest::{Client, header};
use serde_json::{Value, json};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tracing::Instrument;

use crate::ai_gateway::{
    adapters::{BackendAdapter, http_common},
    error::{GatewayError, GatewayErrorKind},
    types::{AdapterContext, BackendCapabilities, BackendDialect},
    types_chat::{
        AdapterInvocation, BackendIdentity, BackendRawEvent, CanonicalOutputMode, CanonicalRequest,
        CanonicalToolCall, FinishReason, ToolCallStatus, UsageStats,
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
            json_schema_mode: true,
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
        let timeout_ms = ctx.timeout.as_millis();
        let dispatch_span = tracing::debug_span!(
            target: "ai_gateway.openai_compatible",
            "openai_dispatch",
            request_id = %request_id,
            backend_id = %backend_id,
            model = %model,
            stream = req.stream,
            timeout_ms = timeout_ms as u64
        );

        tokio::spawn(
            async move {
                let request_started_at = Instant::now();
                tracing::debug!(
                    target: "ai_gateway.openai_compatible",
                    request_id = %request_id,
                    backend_id = %backend_id,
                    model = %model,
                    stream = req.stream,
                    timeout_ms = timeout_ms as u64,
                    url = %url,
                    "openai_dispatch_start"
                );
                let mut body = json!({
                    "model": model,
                    "messages": http_common::canonical_messages_to_openai(&req.messages),
                    "stream": req.stream,
                });

                if !req.tools.is_empty() {
                    body["tools"] = Value::Array(http_common::tools_to_openai(&req.tools));
                    body["tool_choice"] = http_common::tool_choice_to_openai(&req.tool_choice);
                }

                match &req.output_mode {
                    CanonicalOutputMode::JsonObject => {
                        body["response_format"] = json!({"type": "json_object"});
                    }
                    CanonicalOutputMode::JsonSchema {
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
                    CanonicalOutputMode::Text => {}
                }

                if let Some(max_tokens) = req.limits.max_output_tokens {
                    body["max_tokens"] = Value::Number(max_tokens.into());
                }

                let mut req_builder = client
                    .post(url)
                    .timeout(ctx.timeout)
                    .header(header::CONTENT_TYPE, "application/json")
                    .header("x-request-id", request_id.clone())
                    .json(&body);

                if let Some(auth_header) = credential.auth_header {
                    req_builder = req_builder.header(header::AUTHORIZATION, auth_header);
                }
                for (k, v) in credential.extra_headers {
                    req_builder = req_builder.header(k, v);
                }

                let send_started_at = Instant::now();
                tracing::debug!(
                    target: "ai_gateway.openai_compatible",
                    request_id = %request_id,
                    backend_id = %backend_id,
                    timeout_ms = timeout_ms as u64,
                    "openai_http_send"
                );
                let response = match req_builder.send().await {
                    Ok(response) => response,
                    Err(err) => {
                        tracing::debug!(
                            target: "ai_gateway.openai_compatible",
                            request_id = %request_id,
                            backend_id = %backend_id,
                            elapsed_ms = send_started_at.elapsed().as_millis() as u64,
                            error = %err,
                            "openai_http_error"
                        );
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
                tracing::debug!(
                    target: "ai_gateway.openai_compatible",
                    request_id = %request_id,
                    backend_id = %backend_id,
                    status = response.status().as_u16(),
                    elapsed_ms = send_started_at.elapsed().as_millis() as u64,
                    "openai_http_headers"
                );

                if !response.status().is_success() {
                    let status = response.status().as_u16();
                    let body = response.text().await.unwrap_or_default();
                    tracing::debug!(
                        target: "ai_gateway.openai_compatible",
                        request_id = %request_id,
                        backend_id = %backend_id,
                        status = status,
                        elapsed_ms = request_started_at.elapsed().as_millis() as u64,
                        body_bytes = body.len(),
                        "openai_http_non_success"
                    );
                    let _ = tx
                        .send(Err(http_common::map_http_error(status, &backend_id, &body)))
                        .await;
                    return;
                }

                if req.stream {
                    let mut stream = response.bytes_stream();
                    let mut buffer = String::new();
                    let mut saw_terminal = false;
                    let mut saw_first_chunk = false;

                    while let Some(item) = stream.next().await {
                        if cancel_flag_task.load(Ordering::SeqCst) {
                            tracing::debug!(
                                target: "ai_gateway.openai_compatible",
                                request_id = %request_id,
                                backend_id = %backend_id,
                                elapsed_ms = request_started_at.elapsed().as_millis() as u64,
                                "openai_stream_cancelled"
                            );
                            return;
                        }

                        let chunk = match item {
                            Ok(chunk) => chunk,
                            Err(err) => {
                                tracing::debug!(
                                    target: "ai_gateway.openai_compatible",
                                    request_id = %request_id,
                                    backend_id = %backend_id,
                                    elapsed_ms = request_started_at.elapsed().as_millis() as u64,
                                    error = %err,
                                    "openai_stream_chunk_error"
                                );
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
                        if !saw_first_chunk {
                            saw_first_chunk = true;
                            tracing::debug!(
                                target: "ai_gateway.openai_compatible",
                                request_id = %request_id,
                                backend_id = %backend_id,
                                elapsed_ms = request_started_at.elapsed().as_millis() as u64,
                                chunk_bytes = chunk.len(),
                                "openai_stream_first_chunk"
                            );
                        }

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
                                tracing::debug!(
                                    target: "ai_gateway.openai_compatible",
                                    request_id = %request_id,
                                    backend_id = %backend_id,
                                    elapsed_ms = request_started_at.elapsed().as_millis() as u64,
                                    saw_terminal = saw_terminal,
                                    "openai_stream_done"
                                );
                                if !saw_terminal {
                                    let _ = tx
                                        .send(Ok(BackendRawEvent::Completed {
                                            finish_reason: FinishReason::Stop,
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
                        tracing::debug!(
                            target: "ai_gateway.openai_compatible",
                            request_id = %request_id,
                            backend_id = %backend_id,
                            elapsed_ms = request_started_at.elapsed().as_millis() as u64,
                            "openai_stream_end_without_terminal"
                        );
                        let _ = tx
                            .send(Ok(BackendRawEvent::Completed {
                                finish_reason: FinishReason::Stop,
                            }))
                            .await;
                    }
                    tracing::debug!(
                        target: "ai_gateway.openai_compatible",
                        request_id = %request_id,
                        backend_id = %backend_id,
                        elapsed_ms = request_started_at.elapsed().as_millis() as u64,
                        saw_first_chunk = saw_first_chunk,
                        saw_terminal = saw_terminal,
                        "openai_stream_end"
                    );
                    return;
                }

                let payload = match response.json::<Value>().await {
                    Ok(payload) => payload,
                    Err(err) => {
                        tracing::debug!(
                            target: "ai_gateway.openai_compatible",
                            request_id = %request_id,
                            backend_id = %backend_id,
                            elapsed_ms = request_started_at.elapsed().as_millis() as u64,
                            error = %err,
                            "openai_body_decode_error"
                        );
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
                tracing::debug!(
                    target: "ai_gateway.openai_compatible",
                    request_id = %request_id,
                    backend_id = %backend_id,
                    elapsed_ms = request_started_at.elapsed().as_millis() as u64,
                    "openai_non_stream_payload_ready"
                );

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
