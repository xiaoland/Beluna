use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
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
    },
    chat::types::{
        AdapterInvocation, BackendCompleteResponse, BackendIdentity, BackendRawEvent, FinishReason,
        ToolCallResult, ToolCallStatus, TurnPayload, UsageStats,
    },
    error::{GatewayError, GatewayErrorKind},
    types::{AdapterContext, BackendCapabilities, BackendDialect},
};

use super::wire as ollama_wire;

// ---------------------------------------------------------------------------
// Adapter
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct OllamaAdapter {
    client: reqwest::Client,
}

impl Default for OllamaAdapter {
    fn default() -> Self {
        Self {
            client: reqwest::Client::builder()
                .pool_idle_timeout(std::time::Duration::from_secs(30))
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
            target: "ai_gateway.ollama",
            "ollama_dispatch",
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
                                    format!("ollama stream chunk error: {}", err),
                                )
                                .with_retryable(true)
                                .with_backend_id(backend_id.clone())))
                                .await;
                            return;
                        }
                    };

                    buffer.push_str(&String::from_utf8_lossy(&chunk));
                    let frames = match http_stream::extract_ndjson_frames(&mut buffer, &backend_id)
                    {
                        Ok(result) => result,
                        Err(err) => {
                            let _ = tx.send(Err(err)).await;
                            return;
                        }
                    };

                    for json in frames {
                        match parse_stream_delta(&json) {
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
                                let _ = tx.send(Err(err.with_backend_id(backend_id.clone()))).await;
                                return;
                            }
                        }
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
                dialect: BackendDialect::Ollama,
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
            "ollama backend requires endpoint",
        )
        .with_retryable(false)
        .with_backend_id(ctx.backend_id.clone())
    })?;
    Ok(format!("{}/api/chat", endpoint.trim_end_matches('/')))
}

fn build_body(model: &str, payload: &TurnPayload, stream: bool) -> Value {
    let mut body = json!({
        "model": model,
        "messages": ollama_wire::messages_to_ollama(&payload.messages),
        "stream": stream,
    });

    if !payload.tools.is_empty() {
        body["tools"] = Value::Array(ollama_wire::tools_to_ollama(&payload.tools));
    }
    if let Some(max_tokens) = payload.limits.max_output_tokens {
        body["options"] = json!({"num_predict": max_tokens});
    }
    if payload.enable_thinking {
        body["think"] = Value::Bool(true);
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
    let output_text = payload
        .get("message")
        .and_then(|m| m.get("content"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();

    let tool_calls = parse_tool_calls_from_message(payload.get("message"));

    let done = payload
        .get("done")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    if !done {
        return Err(GatewayError::new(
            GatewayErrorKind::ProtocolViolation,
            "ollama complete response missing done:true",
        )
        .with_retryable(false)
        .with_backend_id(backend_id.to_string()));
    }

    let usage = parse_usage(payload);

    Ok(BackendCompleteResponse {
        backend_identity: BackendIdentity {
            backend_id: backend_id.to_string(),
            dialect: BackendDialect::Ollama,
            model: model.to_string(),
        },
        output_text,
        tool_calls,
        usage: Some(usage),
        finish_reason: FinishReason::Stop,
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
                .unwrap_or("tool_call_0")
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
                .map(Value::to_string)
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

fn parse_stream_delta(payload: &Value) -> Result<Vec<BackendRawEvent>, GatewayError> {
    let mut events = Vec::new();

    if let Some(content) = payload
        .get("message")
        .and_then(|m| m.get("content"))
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
        .and_then(|m| m.get("tool_calls"))
        .and_then(Value::as_array)
    {
        for tc in tool_calls {
            let id = tc
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or("tool_call_0")
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
                .map(Value::to_string)
                .unwrap_or_else(|| "{}".to_string());

            events.push(BackendRawEvent::ToolCallReady {
                call: ToolCallResult {
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
            events.push(BackendRawEvent::Usage {
                usage: parse_usage(payload),
            });
            events.push(BackendRawEvent::Completed {
                finish_reason: FinishReason::Stop,
            });
        }
    }

    Ok(events)
}

// ---------------------------------------------------------------------------
// Usage parsing
// ---------------------------------------------------------------------------

fn parse_usage(payload: &Value) -> UsageStats {
    let input = payload.get("prompt_eval_count").and_then(Value::as_u64);
    let output = payload.get("eval_count").and_then(Value::as_u64);
    UsageStats {
        input_tokens: input,
        output_tokens: output,
        total_tokens: match (input, output) {
            (Some(a), Some(b)) => Some(a + b),
            _ => None,
        },
        provider_usage_raw: Some(payload.clone()),
    }
}
