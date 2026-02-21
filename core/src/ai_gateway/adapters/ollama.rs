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
        AdapterInvocation, BackendIdentity, BackendRawEvent, CanonicalRequest, CanonicalToolCall,
        FinishReason, ToolCallStatus, UsageStats,
    },
};

#[derive(Clone)]
pub struct OllamaAdapter {
    client: Client,
}

impl Default for OllamaAdapter {
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
impl BackendAdapter for OllamaAdapter {
    fn dialect(&self) -> BackendDialect {
        BackendDialect::Ollama
    }

    fn static_capabilities(&self) -> BackendCapabilities {
        BackendCapabilities {
            streaming: true,
            tool_calls: false,
            json_mode: false,
            json_schema_mode: false,
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
                "ollama backend requires endpoint",
            )
            .with_retryable(false)
            .with_backend_id(ctx.backend_id.clone())
        })?;

        let url = format!("{}/api/chat", endpoint.trim_end_matches('/'));
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
            target: "ai_gateway.ollama",
            "ollama_dispatch",
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
                    target: "ai_gateway.ollama",
                    request_id = %request_id,
                    backend_id = %backend_id,
                    model = %model,
                    stream = req.stream,
                    timeout_ms = timeout_ms as u64,
                    url = %url,
                    "ollama_dispatch_start"
                );
                let mut body = json!({
                    "model": model,
                    "messages": http_common::canonical_messages_to_ollama(&req.messages),
                    "stream": req.stream,
                });

                if !req.tools.is_empty() {
                    body["tools"] = Value::Array(http_common::tools_to_ollama(&req.tools));
                }
                if let Some(max_tokens) = req.limits.max_output_tokens {
                    body["options"] = json!({"num_predict": max_tokens});
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

                let response = match req_builder.send().await {
                    Ok(response) => response,
                    Err(err) => {
                        tracing::debug!(
                            target: "ai_gateway.ollama",
                            request_id = %request_id,
                            backend_id = %backend_id,
                            elapsed_ms = request_started_at.elapsed().as_millis() as u64,
                            error = %err,
                            "ollama_http_error"
                        );
                        let _ = tx
                            .send(Err(GatewayError::new(
                                GatewayErrorKind::BackendTransient,
                                format!("ollama request failed: {}", err),
                            )
                            .with_retryable(true)
                            .with_backend_id(backend_id.clone())))
                            .await;
                        return;
                    }
                };
                tracing::debug!(
                    target: "ai_gateway.ollama",
                    request_id = %request_id,
                    backend_id = %backend_id,
                    status = response.status().as_u16(),
                    elapsed_ms = request_started_at.elapsed().as_millis() as u64,
                    "ollama_http_headers"
                );

                if !response.status().is_success() {
                    let status = response.status().as_u16();
                    let body = response.text().await.unwrap_or_default();
                    tracing::debug!(
                        target: "ai_gateway.ollama",
                        request_id = %request_id,
                        backend_id = %backend_id,
                        status = status,
                        body_bytes = body.len(),
                        elapsed_ms = request_started_at.elapsed().as_millis() as u64,
                        "ollama_http_non_success"
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
                                target: "ai_gateway.ollama",
                                request_id = %request_id,
                                backend_id = %backend_id,
                                elapsed_ms = request_started_at.elapsed().as_millis() as u64,
                                "ollama_stream_cancelled"
                            );
                            return;
                        }

                        let chunk = match item {
                            Ok(chunk) => chunk,
                            Err(err) => {
                                tracing::debug!(
                                    target: "ai_gateway.ollama",
                                    request_id = %request_id,
                                    backend_id = %backend_id,
                                    elapsed_ms = request_started_at.elapsed().as_millis() as u64,
                                    error = %err,
                                    "ollama_stream_chunk_error"
                                );
                                let _ = tx
                                    .send(Err(GatewayError::new(
                                        GatewayErrorKind::BackendTransient,
                                        format!("ollama stream chunk error: {}", err),
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
                                target: "ai_gateway.ollama",
                                request_id = %request_id,
                                backend_id = %backend_id,
                                elapsed_ms = request_started_at.elapsed().as_millis() as u64,
                                chunk_bytes = chunk.len(),
                                "ollama_stream_first_chunk"
                            );
                        }

                        buffer.push_str(&String::from_utf8_lossy(&chunk));
                        while let Some(idx) = buffer.find('\n') {
                            let line = buffer[..idx].trim_end_matches('\r').to_string();
                            buffer = buffer[idx + 1..].to_string();
                            if line.trim().is_empty() {
                                continue;
                            }

                            let payload = match serde_json::from_str::<Value>(&line) {
                                Ok(payload) => payload,
                                Err(err) => {
                                    let _ = tx
                                        .send(Err(GatewayError::new(
                                            GatewayErrorKind::ProtocolViolation,
                                            format!("invalid ollama ndjson payload: {}", err),
                                        )
                                        .with_retryable(false)
                                        .with_backend_id(backend_id.clone())))
                                        .await;
                                    return;
                                }
                            };

                            match parse_ollama_payload(&payload) {
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
                                    let _ =
                                        tx.send(Err(err.with_backend_id(backend_id.clone()))).await;
                                    return;
                                }
                            }
                        }
                    }

                    if !saw_terminal {
                        tracing::debug!(
                            target: "ai_gateway.ollama",
                            request_id = %request_id,
                            backend_id = %backend_id,
                            elapsed_ms = request_started_at.elapsed().as_millis() as u64,
                            "ollama_stream_end_without_terminal"
                        );
                        let _ = tx
                            .send(Ok(BackendRawEvent::Completed {
                                finish_reason: FinishReason::Stop,
                            }))
                            .await;
                    }
                    tracing::debug!(
                        target: "ai_gateway.ollama",
                        request_id = %request_id,
                        backend_id = %backend_id,
                        elapsed_ms = request_started_at.elapsed().as_millis() as u64,
                        saw_first_chunk = saw_first_chunk,
                        saw_terminal = saw_terminal,
                        "ollama_stream_end"
                    );
                    return;
                }

                let payload = match response.json::<Value>().await {
                    Ok(payload) => payload,
                    Err(err) => {
                        tracing::debug!(
                            target: "ai_gateway.ollama",
                            request_id = %request_id,
                            backend_id = %backend_id,
                            elapsed_ms = request_started_at.elapsed().as_millis() as u64,
                            error = %err,
                            "ollama_body_decode_error"
                        );
                        let _ = tx
                            .send(Err(GatewayError::new(
                                GatewayErrorKind::ProtocolViolation,
                                format!("invalid ollama response payload: {}", err),
                            )
                            .with_retryable(false)
                            .with_backend_id(backend_id.clone())))
                            .await;
                        return;
                    }
                };
                tracing::debug!(
                    target: "ai_gateway.ollama",
                    request_id = %request_id,
                    backend_id = %backend_id,
                    elapsed_ms = request_started_at.elapsed().as_millis() as u64,
                    "ollama_non_stream_payload_ready"
                );

                match parse_ollama_payload(&payload) {
                    Ok(events) => {
                        for event in events {
                            if tx.send(Ok(event)).await.is_err() {
                                return;
                            }
                        }
                    }
                    Err(err) => {
                        let _ = tx.send(Err(err.with_backend_id(backend_id.clone()))).await;
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
                dialect: BackendDialect::Ollama,
                model: ctx.model,
            },
            cancel: Some(cancel),
        })
    }
}

fn parse_ollama_payload(payload: &Value) -> Result<Vec<BackendRawEvent>, GatewayError> {
    let mut events = Vec::new();

    if let Some(content) = payload
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

    if let Some(tool_calls) = payload
        .get("message")
        .and_then(|message| message.get("tool_calls"))
        .and_then(Value::as_array)
    {
        for tool_call in tool_calls {
            let name = tool_call
                .get("function")
                .and_then(|f| f.get("name"))
                .and_then(Value::as_str)
                .unwrap_or("unknown_tool")
                .to_string();
            let arguments_json = tool_call
                .get("function")
                .and_then(|f| f.get("arguments"))
                .map(Value::to_string)
                .unwrap_or_else(|| "{}".to_string());
            let id = tool_call
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or("tool_call_0")
                .to_string();

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

    if let Some(done) = payload.get("done").and_then(Value::as_bool) {
        if done {
            let usage = UsageStats {
                input_tokens: payload.get("prompt_eval_count").and_then(Value::as_u64),
                output_tokens: payload.get("eval_count").and_then(Value::as_u64),
                total_tokens: match (
                    payload.get("prompt_eval_count").and_then(Value::as_u64),
                    payload.get("eval_count").and_then(Value::as_u64),
                ) {
                    (Some(a), Some(b)) => Some(a + b),
                    _ => None,
                },
                provider_usage_raw: Some(payload.clone()),
            };
            events.push(BackendRawEvent::Usage { usage });
            events.push(BackendRawEvent::Completed {
                finish_reason: FinishReason::Stop,
            });
        }
    }

    Ok(events)
}
