use std::{collections::HashMap, sync::Arc, time::Instant};

use futures_util::StreamExt;
use tokio::{sync::mpsc, time::sleep};
use tokio_stream::wrappers::ReceiverStream;
use tracing::Instrument;

use crate::ai_gateway::{
    adapters::{BackendAdapter, build_default_adapters},
    budget::{BudgetEnforcer, BudgetLease},
    capabilities::CapabilityGuard,
    credentials::CredentialProvider,
    error::{GatewayError, GatewayErrorKind},
    reliability::ReliabilityLayer,
    request_normalizer::RequestNormalizer,
    response_normalizer::ResponseNormalizer,
    router::BackendRouter,
    telemetry::{GatewayTelemetryEvent, emit_gateway_event},
    types::{AIGatewayConfig, AdapterContext, BackendCapabilities, BackendDialect},
    types_chat::{
        CanonicalRequest, ChatEvent, ChatEventStream, ChatRequest, ChatResponse, FinishReason,
        UsageStats,
    },
};

pub struct AIGateway {
    router: BackendRouter,
    credential_provider: Arc<dyn CredentialProvider>,
    adapters: HashMap<BackendDialect, Arc<dyn BackendAdapter>>,
    request_normalizer: RequestNormalizer,
    response_normalizer: ResponseNormalizer,
    capability_guard: CapabilityGuard,
    budget_enforcer: BudgetEnforcer,
    reliability: ReliabilityLayer,
}

impl AIGateway {
    pub fn new(
        config: AIGatewayConfig,
        credential_provider: Arc<dyn CredentialProvider>,
    ) -> Result<Self, GatewayError> {
        let router = BackendRouter::new(&config)?;
        Ok(Self {
            router,
            credential_provider,
            adapters: build_default_adapters(),
            request_normalizer: RequestNormalizer,
            response_normalizer: ResponseNormalizer,
            capability_guard: CapabilityGuard,
            budget_enforcer: BudgetEnforcer::new(config.budget),
            reliability: ReliabilityLayer::new(config.reliability),
        })
    }

    pub fn with_adapters(
        mut self,
        adapters: HashMap<BackendDialect, Arc<dyn BackendAdapter>>,
    ) -> Self {
        self.adapters = adapters;
        self
    }

    pub async fn chat_stream(&self, request: ChatRequest) -> Result<ChatEventStream, GatewayError> {
        self.chat_stream_internal(request, true).await
    }

    pub async fn chat_once(&self, request: ChatRequest) -> Result<ChatResponse, GatewayError> {
        let mut stream = self.chat_stream_internal(request, false).await?;
        let mut output_text = String::new();
        let mut tool_calls = Vec::new();
        let mut usage: Option<UsageStats> = None;
        let mut finish_reason: Option<FinishReason> = None;
        let mut request_id: Option<String> = None;

        while let Some(item) = stream.next().await {
            let event = item?;
            match event {
                ChatEvent::Started { request_id: id, .. } => {
                    request_id = Some(id);
                }
                ChatEvent::TextDelta { delta, .. } => {
                    output_text.push_str(&delta);
                }
                ChatEvent::ToolCallReady { call, .. } => {
                    tool_calls.push(call);
                }
                ChatEvent::Usage { usage: current, .. } => {
                    usage = Some(current);
                }
                ChatEvent::Completed {
                    request_id: id,
                    finish_reason: reason,
                } => {
                    request_id = Some(id);
                    finish_reason = Some(reason);
                    break;
                }
                ChatEvent::Failed { error, .. } => {
                    return Err(error);
                }
                ChatEvent::ToolCallDelta { .. } => {}
            }
        }

        let request_id = request_id.ok_or_else(|| {
            GatewayError::new(
                GatewayErrorKind::ProtocolViolation,
                "missing request id in stream",
            )
            .with_retryable(false)
        })?;

        let finish_reason = finish_reason.ok_or_else(|| {
            GatewayError::new(
                GatewayErrorKind::ProtocolViolation,
                "stream ended without terminal event",
            )
            .with_retryable(false)
        })?;

        Ok(ChatResponse {
            request_id,
            output_text,
            tool_calls,
            usage,
            finish_reason,
            backend_metadata: Default::default(),
        })
    }

    async fn chat_stream_internal(
        &self,
        request: ChatRequest,
        stream: bool,
    ) -> Result<ChatEventStream, GatewayError> {
        let canonical_request = self.request_normalizer.normalize_chat(request, stream)?;
        self.dispatch_stream(canonical_request).await
    }

    async fn dispatch_stream(
        &self,
        canonical_request: CanonicalRequest,
    ) -> Result<ChatEventStream, GatewayError> {
        let selected = self.router.select(&canonical_request)?;
        let credential = self
            .credential_provider
            .resolve(&selected.profile.credential, &selected.profile)
            .await?;

        let adapter = self
            .adapters
            .get(&selected.profile.dialect)
            .cloned()
            .ok_or_else(|| {
                GatewayError::new(
                    GatewayErrorKind::InvalidRequest,
                    format!(
                        "adapter for dialect {:?} is not registered",
                        selected.profile.dialect
                    ),
                )
                .with_retryable(false)
                .with_backend_id(selected.backend_id.clone())
            })?;

        let capabilities = selected
            .profile
            .capabilities
            .clone()
            .unwrap_or_else(|| adapter.static_capabilities());
        self.capability_guard
            .assert_supported(&canonical_request, &capabilities)?;

        let budget_lease = self
            .budget_enforcer
            .pre_dispatch(&canonical_request, &selected.backend_id)
            .await?;

        let stage = canonical_request
            .metadata
            .get("cortex_stage")
            .map(String::as_str)
            .unwrap_or("primary");
        let request_span = tracing::info_span!(
            target: "ai_gateway",
            "gateway_request",
            request_id = %canonical_request.request_id,
            backend_id = %selected.backend_id,
            model = %selected.resolved_model,
            stream = canonical_request.stream,
            stage = stage,
            cost_attribution_id = canonical_request.cost_attribution_id.as_deref().unwrap_or("-")
        );

        tracing::debug!(
            target: "ai_gateway",
            request_id = %canonical_request.request_id,
            backend_id = %selected.backend_id,
            dialect = ?selected.profile.dialect,
            model = %selected.resolved_model,
            stream = canonical_request.stream,
            stage = stage,
            max_output_tokens = ?canonical_request.limits.max_output_tokens,
            requested_timeout_ms = ?canonical_request.limits.max_request_time_ms,
            effective_timeout_ms = budget_lease.effective_timeout.as_millis() as u64,
            "chat_stream_prepared"
        );

        emit_gateway_event(GatewayTelemetryEvent::RequestStarted {
            request_id: canonical_request.request_id.clone(),
            backend_id: selected.backend_id.clone(),
            model: selected.resolved_model.clone(),
            cost_attribution_id: canonical_request.cost_attribution_id.clone(),
        });

        let (tx, rx) = mpsc::channel::<Result<ChatEvent, GatewayError>>(128);

        let response_normalizer = self.response_normalizer;
        let reliability = self.reliability.clone();
        let budget_enforcer = self.budget_enforcer.clone();

        tokio::spawn(
            async move {
                run_stream_task(
                    tx,
                    canonical_request,
                    selected,
                    adapter,
                    credential,
                    capabilities,
                    response_normalizer,
                    reliability,
                    budget_enforcer,
                    budget_lease,
                )
                .await;
            }
            .instrument(request_span),
        );

        Ok(Box::pin(ReceiverStream::new(rx)))
    }
}

#[allow(clippy::too_many_arguments)]
#[tracing::instrument(
    name = "gateway_stream_task",
    target = "ai_gateway",
    skip(
        tx,
        canonical_request,
        selected,
        adapter,
        credential,
        capabilities,
        response_normalizer,
        reliability,
        budget_enforcer,
        budget_lease
    ),
    fields(
        request_id = %canonical_request.request_id,
        backend_id = %selected.backend_id,
        model = %selected.resolved_model,
        stream = canonical_request.stream,
        cost_attribution_id = canonical_request.cost_attribution_id.as_deref().unwrap_or("-")
    )
)]
async fn run_stream_task(
    tx: mpsc::Sender<Result<ChatEvent, GatewayError>>,
    canonical_request: CanonicalRequest,
    selected: crate::ai_gateway::router::SelectedBackend,
    adapter: Arc<dyn BackendAdapter>,
    credential: crate::ai_gateway::types::ResolvedCredential,
    capabilities: BackendCapabilities,
    response_normalizer: ResponseNormalizer,
    reliability: ReliabilityLayer,
    budget_enforcer: BudgetEnforcer,
    budget_lease: BudgetLease,
) {
    let request_id = canonical_request.request_id.clone();
    let cost_attribution_id = canonical_request.cost_attribution_id.clone();
    let request_started_at = Instant::now();
    let mut attempt = 0_u32;
    let mut usage_emitted = false;
    let mut last_usage: Option<UsageStats> = None;
    let mut lease = Some(budget_lease);

    if tx
        .send(Ok(ChatEvent::Started {
            request_id: request_id.clone(),
            backend_id: selected.backend_id.clone(),
            model_id: selected.resolved_model.clone(),
        }))
        .await
        .is_err()
    {
        release_lease(&budget_enforcer, &mut lease);
        emit_gateway_event(GatewayTelemetryEvent::RequestCancelled {
            request_id,
            cost_attribution_id,
        });
        return;
    }
    tracing::debug!(
        target: "ai_gateway",
        request_id = %request_id,
        backend_id = %selected.backend_id,
        model = %selected.resolved_model,
        "run_stream_task_started"
    );

    loop {
        if tx.is_closed() {
            release_lease(&budget_enforcer, &mut lease);
            emit_gateway_event(GatewayTelemetryEvent::RequestCancelled {
                request_id: request_id.clone(),
                cost_attribution_id: cost_attribution_id.clone(),
            });
            return;
        }

        let attempt_started_at = Instant::now();
        emit_gateway_event(GatewayTelemetryEvent::AttemptStarted {
            request_id: request_id.clone(),
            attempt,
            cost_attribution_id: cost_attribution_id.clone(),
        });

        if let Err(err) = reliability
            .ensure_backend_allowed(&selected.backend_id)
            .await
        {
            let _ = tx
                .send(Ok(ChatEvent::Failed {
                    request_id: request_id.clone(),
                    error: err.clone(),
                }))
                .await;
            release_lease(&budget_enforcer, &mut lease);
            emit_gateway_event(GatewayTelemetryEvent::RequestFailed {
                request_id,
                attempts: attempt + 1,
                error_kind: err.kind,
                cost_attribution_id,
            });
            return;
        }

        let adapter_ctx = AdapterContext {
            backend_id: selected.backend_id.clone(),
            model: selected.resolved_model.clone(),
            profile: selected.profile.clone(),
            credential: credential.clone(),
            timeout: lease
                .as_ref()
                .map(|l| l.effective_timeout)
                .unwrap_or_else(|| {
                    std::time::Duration::from_millis(reliability.config().request_timeout_ms)
                }),
            request_id: request_id.clone(),
        };

        let timeout_ms = adapter_ctx.timeout.as_millis();
        tracing::debug!(
            target: "ai_gateway",
            request_id = %request_id,
            attempt = attempt,
            backend_id = %selected.backend_id,
            timeout_ms = timeout_ms as u64,
            "attempt_invoke_start"
        );
        let invoke_started_at = Instant::now();

        let invocation = adapter
            .invoke_stream(adapter_ctx, canonical_request.clone())
            .await;
        tracing::debug!(
            target: "ai_gateway",
            request_id = %request_id,
            attempt = attempt,
            backend_id = %selected.backend_id,
            elapsed_ms = invoke_started_at.elapsed().as_millis() as u64,
            success = invocation.is_ok(),
            "attempt_invoke_result"
        );

        let mut emitted_output = false;
        let mut emitted_tool = false;

        let invocation = match invocation {
            Ok(invocation) => invocation,
            Err(err) => {
                let can_retry = reliability.can_retry(
                    &err,
                    attempt,
                    emitted_output,
                    emitted_tool,
                    &capabilities,
                    adapter.supports_tool_retry(),
                );
                tracing::debug!(
                    target: "ai_gateway",
                    request_id = %request_id,
                    attempt = attempt,
                    backend_id = %selected.backend_id,
                    elapsed_ms = attempt_started_at.elapsed().as_millis() as u64,
                    kind = ?err.kind,
                    retryable = err.retryable,
                    can_retry = can_retry,
                    error = %err.message,
                    "attempt_invoke_failed"
                );

                emit_gateway_event(GatewayTelemetryEvent::AttemptFailed {
                    request_id: request_id.clone(),
                    attempt,
                    kind: err.kind,
                    retryable: err.retryable,
                    cost_attribution_id: cost_attribution_id.clone(),
                });

                reliability
                    .record_failure(
                        &selected.backend_id,
                        ReliabilityLayer::counts_toward_breaker(&err),
                    )
                    .await;

                if can_retry {
                    attempt += 1;
                    let delay = reliability.backoff_delay(attempt);
                    tokio::select! {
                        _ = tx.closed() => {
                            release_lease(&budget_enforcer, &mut lease);
                            emit_gateway_event(GatewayTelemetryEvent::RequestCancelled {
                                request_id: request_id.clone(),
                                cost_attribution_id: cost_attribution_id.clone(),
                            });
                            return;
                        }
                        _ = sleep(delay) => {}
                    }
                    continue;
                }

                let _ = tx
                    .send(Ok(ChatEvent::Failed {
                        request_id: request_id.clone(),
                        error: err.clone(),
                    }))
                    .await;
                release_lease(&budget_enforcer, &mut lease);
                emit_gateway_event(GatewayTelemetryEvent::RequestFailed {
                    request_id,
                    attempts: attempt + 1,
                    error_kind: err.kind,
                    cost_attribution_id,
                });
                return;
            }
        };
        tracing::debug!(
            target: "ai_gateway",
            request_id = %request_id,
            attempt = attempt,
            backend_id = %invocation.backend_identity.backend_id,
            dialect = ?invocation.backend_identity.dialect,
            model = %invocation.backend_identity.model,
            "attempt_stream_started"
        );

        let mut terminal_error: Option<GatewayError> = None;
        let mut terminal_success: Option<FinishReason> = None;
        let mut stream = invocation.stream;
        let cancel = invocation.cancel.clone();
        let mut saw_first_stream_event = false;

        loop {
            tokio::select! {
                _ = tx.closed() => {
                    if let Some(cancel) = cancel.clone() {
                        cancel();
                    }
                    release_lease(&budget_enforcer, &mut lease);
                    emit_gateway_event(GatewayTelemetryEvent::RequestCancelled {
                        request_id: request_id.clone(),
                        cost_attribution_id: cost_attribution_id.clone(),
                    });
                    return;
                }
                _ = sleep(std::time::Duration::from_secs(5)), if !saw_first_stream_event => {
                    tracing::debug!(
                        target: "ai_gateway",
                        request_id = %request_id,
                        attempt = attempt,
                        backend_id = %selected.backend_id,
                        elapsed_ms = attempt_started_at.elapsed().as_millis() as u64,
                        "attempt_waiting_first_event"
                    );
                }
                next_item = stream.next() => {
                    let Some(item) = next_item else {
                        if terminal_success.is_none() && terminal_error.is_none() {
                            terminal_error = Some(
                                GatewayError::new(
                                    GatewayErrorKind::ProtocolViolation,
                                    "adapter stream ended without terminal event",
                                )
                                .with_retryable(false)
                                .with_backend_id(selected.backend_id.clone())
                            );
                            tracing::debug!(
                                target: "ai_gateway",
                                request_id = %request_id,
                                attempt = attempt,
                                backend_id = %selected.backend_id,
                                elapsed_ms = attempt_started_at.elapsed().as_millis() as u64,
                                "attempt_stream_ended_without_terminal"
                            );
                        }
                        break;
                    };

                    let raw_event = match item {
                        Ok(raw_event) => raw_event,
                        Err(err) => {
                            terminal_error = Some(err.with_backend_id(selected.backend_id.clone()));
                            break;
                        }
                    };

                    let event = match response_normalizer.map_raw(&request_id, raw_event) {
                        Ok(event) => event,
                        Err(err) => {
                            terminal_error = Some(err.with_backend_id(selected.backend_id.clone()));
                            break;
                        }
                    };

                    if !saw_first_stream_event {
                        saw_first_stream_event = true;
                        emit_gateway_event(GatewayTelemetryEvent::StreamFirstEvent {
                            request_id: request_id.clone(),
                        });
                        tracing::debug!(
                            target: "ai_gateway",
                            request_id = %request_id,
                            attempt = attempt,
                            backend_id = %selected.backend_id,
                            event = gateway_event_name(&event),
                            elapsed_ms = attempt_started_at.elapsed().as_millis() as u64,
                            "attempt_first_event"
                        );
                    }

                    if ResponseNormalizer::is_output_event(&event) {
                        emitted_output = true;
                    }
                    if ResponseNormalizer::is_tool_event(&event) {
                        emitted_tool = true;
                    }

                    match event {
                        ChatEvent::Usage { usage, .. } => {
                            if usage_emitted {
                                terminal_error = Some(
                                    GatewayError::new(
                                        GatewayErrorKind::ProtocolViolation,
                                        "duplicate usage event received",
                                    )
                                    .with_retryable(false)
                                    .with_backend_id(selected.backend_id.clone())
                                );
                                break;
                            }
                            usage_emitted = true;
                            last_usage = Some(usage.clone());
                            let usage_event = ChatEvent::Usage {
                                request_id: request_id.clone(),
                                usage,
                            };
                            budget_enforcer.observe_event(&selected.backend_id, &usage_event).await;
                            if tx.send(Ok(usage_event)).await.is_err() {
                                if let Some(cancel) = cancel.clone() {
                                    cancel();
                                }
                                release_lease(&budget_enforcer, &mut lease);
                                emit_gateway_event(GatewayTelemetryEvent::RequestCancelled {
                                    request_id: request_id.clone(),
                                    cost_attribution_id: cost_attribution_id.clone(),
                                });
                                return;
                            }
                        }
                        ChatEvent::Completed { finish_reason, .. } => {
                            terminal_success = Some(finish_reason);
                            break;
                        }
                        ChatEvent::Failed { error, .. } => {
                            terminal_error = Some(error);
                            break;
                        }
                        non_terminal => {
                            if tx.send(Ok(non_terminal)).await.is_err() {
                                if let Some(cancel) = cancel.clone() {
                                    cancel();
                                }
                                release_lease(&budget_enforcer, &mut lease);
                                emit_gateway_event(GatewayTelemetryEvent::RequestCancelled {
                                    request_id: request_id.clone(),
                                    cost_attribution_id: cost_attribution_id.clone(),
                                });
                                return;
                            }
                        }
                    }
                }
            }
        }

        if let Some(finish_reason) = terminal_success {
            tracing::debug!(
                target: "ai_gateway",
                request_id = %request_id,
                attempt = attempt,
                backend_id = %selected.backend_id,
                finish_reason = ?finish_reason,
                elapsed_ms = attempt_started_at.elapsed().as_millis() as u64,
                "attempt_completed"
            );
            reliability.record_success(&selected.backend_id).await;
            let _ = tx
                .send(Ok(ChatEvent::Completed {
                    request_id: request_id.clone(),
                    finish_reason,
                }))
                .await;
            release_lease(&budget_enforcer, &mut lease);
            emit_gateway_event(GatewayTelemetryEvent::RequestCompleted {
                request_id,
                attempts: attempt + 1,
                usage: last_usage,
                cost_attribution_id,
            });
            tracing::debug!(
                target: "ai_gateway",
                request_id = %canonical_request.request_id,
                backend_id = %selected.backend_id,
                attempts = attempt + 1,
                total_elapsed_ms = request_started_at.elapsed().as_millis() as u64,
                "request_completed"
            );
            return;
        }

        let err = terminal_error.unwrap_or_else(|| {
            GatewayError::new(
                GatewayErrorKind::ProtocolViolation,
                "missing terminal event",
            )
            .with_retryable(false)
            .with_backend_id(selected.backend_id.clone())
        });

        let can_retry = reliability.can_retry(
            &err,
            attempt,
            emitted_output,
            emitted_tool,
            &capabilities,
            adapter.supports_tool_retry(),
        );
        tracing::debug!(
            target: "ai_gateway",
            request_id = %request_id,
            attempt = attempt,
            backend_id = %selected.backend_id,
            kind = ?err.kind,
            retryable = err.retryable,
            can_retry = can_retry,
            emitted_output = emitted_output,
            emitted_tool = emitted_tool,
            elapsed_ms = attempt_started_at.elapsed().as_millis() as u64,
            error = %err.message,
            "attempt_terminal_error"
        );

        emit_gateway_event(GatewayTelemetryEvent::AttemptFailed {
            request_id: request_id.clone(),
            attempt,
            kind: err.kind,
            retryable: err.retryable,
            cost_attribution_id: cost_attribution_id.clone(),
        });

        reliability
            .record_failure(
                &selected.backend_id,
                ReliabilityLayer::counts_toward_breaker(&err),
            )
            .await;

        if can_retry {
            attempt += 1;
            let delay = reliability.backoff_delay(attempt);
            tokio::select! {
                _ = tx.closed() => {
                    if let Some(cancel) = invocation.cancel {
                        cancel();
                    }
                    release_lease(&budget_enforcer, &mut lease);
                    emit_gateway_event(GatewayTelemetryEvent::RequestCancelled {
                        request_id: request_id.clone(),
                        cost_attribution_id: cost_attribution_id.clone(),
                    });
                    return;
                }
                _ = sleep(delay) => {}
            }
            continue;
        }

        let _ = tx
            .send(Ok(ChatEvent::Failed {
                request_id: request_id.clone(),
                error: err.clone(),
            }))
            .await;
        release_lease(&budget_enforcer, &mut lease);
        emit_gateway_event(GatewayTelemetryEvent::RequestFailed {
            request_id,
            attempts: attempt + 1,
            error_kind: err.kind,
            cost_attribution_id,
        });
        tracing::debug!(
            target: "ai_gateway",
            request_id = %canonical_request.request_id,
            backend_id = %selected.backend_id,
            attempts = attempt + 1,
            total_elapsed_ms = request_started_at.elapsed().as_millis() as u64,
            error_kind = ?err.kind,
            "request_failed"
        );
        return;
    }
}

fn gateway_event_name(event: &ChatEvent) -> &'static str {
    match event {
        ChatEvent::Started { .. } => "started",
        ChatEvent::TextDelta { .. } => "text_delta",
        ChatEvent::ToolCallDelta { .. } => "tool_call_delta",
        ChatEvent::ToolCallReady { .. } => "tool_call_ready",
        ChatEvent::Usage { .. } => "usage",
        ChatEvent::Completed { .. } => "completed",
        ChatEvent::Failed { .. } => "failed",
    }
}

fn release_lease(budget_enforcer: &BudgetEnforcer, lease: &mut Option<BudgetLease>) {
    if let Some(current) = lease.take() {
        budget_enforcer.release(current);
    }
}
