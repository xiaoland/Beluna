use std::{
    process::Stdio,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Instant,
};

use async_trait::async_trait;
use futures_util::StreamExt;
use serde_json::{Value, json};
use tokio::{process::Command, sync::mpsc};
use tokio_stream::wrappers::ReceiverStream;
use tracing::Instrument;

use crate::ai_gateway::{
    adapters::{BackendAdapter, github_copilot::rpc::RpcIo},
    chat::types::{
        AdapterInvocation, BackendIdentity, BackendCompleteResponse, BackendRawEvent, ContentPart,
        FinishReason, TurnPayload,
    },
    error::{GatewayError, GatewayErrorKind},
    types::{AdapterContext, BackendCapabilities, BackendDialect},
};

#[derive(Default)]
pub struct GitHubCopilotAdapter;

#[async_trait]
impl BackendAdapter for GitHubCopilotAdapter {
    fn dialect(&self) -> BackendDialect {
        BackendDialect::GitHubCopilotSdk
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
        let backend_id = ctx.backend_id.clone();
        let model = ctx.model.clone();
        let invocation = self.stream(ctx, payload).await?;
        let mut stream = invocation.stream;
        let mut output_text = String::new();
        let mut finish_reason: Option<FinishReason> = None;

        while let Some(item) = stream.next().await {
            match item? {
                BackendRawEvent::OutputTextDelta { delta } => {
                    output_text.push_str(&delta);
                }
                BackendRawEvent::Completed {
                    finish_reason: reason,
                } => {
                    finish_reason = Some(reason);
                    break;
                }
                BackendRawEvent::Failed { error } => return Err(error),
                BackendRawEvent::ToolCallDelta { .. }
                | BackendRawEvent::ToolCallReady { .. }
                | BackendRawEvent::Usage { .. } => {}
            }
        }

        let finish_reason = finish_reason.ok_or_else(|| {
            GatewayError::new(
                GatewayErrorKind::ProtocolViolation,
                "copilot once response missing terminal finish reason",
            )
            .with_retryable(false)
            .with_backend_id(backend_id.clone())
        })?;

        Ok(BackendCompleteResponse {
            backend_identity: BackendIdentity {
                backend_id,
                dialect: BackendDialect::GitHubCopilotSdk,
                model,
            },
            output_text,
            tool_calls: Vec::new(),
            usage: None,
            finish_reason,
        })
    }

    async fn stream(
        &self,
        ctx: AdapterContext,
        payload: &TurnPayload,
    ) -> Result<AdapterInvocation, GatewayError> {
        let (tx, rx) = mpsc::channel::<Result<BackendRawEvent, GatewayError>>(16);
        let cancel_flag = Arc::new(AtomicBool::new(false));
        let cancel_flag_task = cancel_flag.clone();

        let backend_id = ctx.backend_id.clone();
        let model = ctx.model.clone();
        let model_for_task = model.clone();
        let request_id = ctx.request_id.clone();
        let copilot_config = ctx.profile.copilot.clone();
        let dispatch_span = tracing::debug_span!(
            target: "ai_gateway.github_copilot",
            "copilot_dispatch",
            request_id = %request_id,
            backend_id = %backend_id,
            model = %model_for_task,
        );

        let prompt_text = extract_text_from_payload(payload);

        tokio::spawn(
            async move {
                let _request_started_at = Instant::now();
                let copilot_config = match copilot_config {
                    Some(config) => config,
                    None => {
                        let _ = tx
                            .send(Err(GatewayError::new(
                                GatewayErrorKind::InvalidRequest,
                                "github_copilot_sdk backend requires copilot config",
                            )
                            .with_retryable(false)
                            .with_backend_id(backend_id.clone())))
                            .await;
                        return;
                    }
                };

                let mut command = Command::new(&copilot_config.command);
                command
                    .args(&copilot_config.args)
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::null());

                let mut child = match command.spawn() {
                    Ok(child) => child,
                    Err(err) => {
                        let _ = tx
                            .send(Err(GatewayError::new(
                                GatewayErrorKind::BackendTransient,
                                format!("failed to spawn copilot language server: {}", err),
                            )
                            .with_retryable(true)
                            .with_backend_id(backend_id.clone())))
                            .await;
                        return;
                    }
                };

                let stdout = match child.stdout.take() {
                    Some(stdout) => stdout,
                    None => {
                        let _ = tx
                            .send(Err(GatewayError::new(
                                GatewayErrorKind::ProtocolViolation,
                                "copilot process missing stdout pipe",
                            )
                            .with_retryable(false)
                            .with_backend_id(backend_id.clone())))
                            .await;
                        let _ = child.kill().await;
                        return;
                    }
                };
                let stdin = match child.stdin.take() {
                    Some(stdin) => stdin,
                    None => {
                        let _ = tx
                            .send(Err(GatewayError::new(
                                GatewayErrorKind::ProtocolViolation,
                                "copilot process missing stdin pipe",
                            )
                            .with_retryable(false)
                            .with_backend_id(backend_id.clone())))
                            .await;
                        let _ = child.kill().await;
                        return;
                    }
                };

                let mut rpc = RpcIo::new(stdout, stdin);

                if let Err(err) = rpc
                    .request(
                        1,
                        "initialize",
                        json!({
                            "processId": std::process::id(),
                            "clientInfo": {"name": "beluna", "version": "0.1.0"},
                            "rootUri": null,
                            "capabilities": {},
                        }),
                    )
                    .await
                {
                    let _ = tx.send(Err(err.with_backend_id(backend_id.clone()))).await;
                    let _ = child.kill().await;
                    return;
                }

                if cancel_flag_task.load(Ordering::SeqCst) {
                    let _ = child.kill().await;
                    return;
                }

                if let Err(err) = rpc.send_notification("initialized", json!({})).await {
                    let _ = tx.send(Err(err.with_backend_id(backend_id.clone()))).await;
                    let _ = child.kill().await;
                    return;
                }

                let status_result = match rpc.request(2, "checkStatus", json!({})).await {
                    Ok(result) => result,
                    Err(err) => {
                        let _ = tx.send(Err(err.with_backend_id(backend_id.clone()))).await;
                        let _ = child.kill().await;
                        return;
                    }
                };

                if !status_is_ready(&status_result) {
                    let _ = tx
                        .send(Err(GatewayError::new(
                            GatewayErrorKind::Authentication,
                            "copilot session is not authenticated/ready",
                        )
                        .with_retryable(false)
                        .with_backend_id(backend_id.clone())))
                        .await;
                    let _ = child.kill().await;
                    return;
                }

                if cancel_flag_task.load(Ordering::SeqCst) {
                    let _ = child.kill().await;
                    return;
                }

                let panel_params = build_panel_completion_params(&prompt_text);
                let completion_result = match rpc
                    .request(3, "textDocument/copilotPanelCompletion", panel_params)
                    .await
                {
                    Ok(result) => result,
                    Err(_) => match rpc
                        .request(
                            4,
                            "textDocument/inlineCompletion",
                            build_panel_completion_params(&prompt_text),
                        )
                        .await
                    {
                        Ok(result) => result,
                        Err(err) => {
                            let _ = tx.send(Err(err.with_backend_id(backend_id.clone()))).await;
                            let _ = child.kill().await;
                            return;
                        }
                    },
                };

                if let Some(text) = extract_completion_text(&completion_result) {
                    if !text.is_empty() {
                        if tx
                            .send(Ok(BackendRawEvent::OutputTextDelta {
                                delta: text.to_string(),
                            }))
                            .await
                            .is_err()
                        {
                            let _ = child.kill().await;
                            return;
                        }
                    }

                    let _ = tx
                        .send(Ok(BackendRawEvent::Completed {
                            finish_reason: FinishReason::Stop,
                        }))
                        .await;
                } else {
                    let _ = tx
                        .send(Err(GatewayError::new(
                            GatewayErrorKind::ProtocolViolation,
                            "copilot completion response missing text",
                        )
                        .with_retryable(false)
                        .with_backend_id(backend_id.clone())))
                        .await;
                }

                let _ = child.kill().await;
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
                dialect: BackendDialect::GitHubCopilotSdk,
                model,
            },
            cancel: Some(cancel),
        })
    }
}

fn status_is_ready(value: &Value) -> bool {
    if let Some(ready) = value.get("ready").and_then(Value::as_bool) {
        return ready;
    }
    if let Some(status) = value.get("status").and_then(Value::as_str) {
        return matches!(status, "ok" | "ready" | "logged_in" | "authorized");
    }
    true
}

fn extract_text_from_payload(payload: &TurnPayload) -> String {
    payload
        .messages
        .iter()
        .flat_map(|message| message.parts.iter())
        .filter_map(|part| match part {
            ContentPart::Text { text } => Some(text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn build_panel_completion_params(text: &str) -> Value {
    json!({
        "textDocument": {"uri": "file:///beluna/copilot-synthetic.txt"},
        "position": {"line": 0, "character": text.len()},
        "context": {"prefix": text},
    })
}

fn extract_completion_text(value: &Value) -> Option<&str> {
    if let Some(text) = value.get("text").and_then(Value::as_str) {
        return Some(text);
    }

    if let Some(items) = value.get("items").and_then(Value::as_array) {
        if let Some(first) = items.first() {
            if let Some(text) = first.get("insertText").and_then(Value::as_str) {
                return Some(text);
            }
            if let Some(text) = first.get("text").and_then(Value::as_str) {
                return Some(text);
            }
        }
    }

    None
}
