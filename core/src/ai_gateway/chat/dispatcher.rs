use std::{collections::HashMap, sync::Arc, time::Instant};

use futures_util::StreamExt;
use tokio::{sync::mpsc, time::sleep};
use tokio_stream::wrappers::ReceiverStream;
use tracing::Instrument;

use crate::observability::metrics as observability_metrics;

use crate::ai_gateway::{
    adapters::{BackendAdapter, build_default_adapters},
    budget::{BudgetEnforcer, BudgetLease},
    credentials::CredentialProvider,
    error::{GatewayError, GatewayErrorKind},
    reliability::ReliabilityLayer,
    router::BackendRouter,
    telemetry::{GatewayTelemetryEvent, emit_gateway_event},
    types::{AIGatewayConfig, AdapterContext, BackendCapabilities, BackendDialect},
};

use super::{
    capabilities::CapabilityGuard,
    types::{
        BackendRawEvent, ChatEvent, ChatEventStream, FinishReason, TurnPayload, TurnResponse,
        UsageStats,
    },
};

/// Internal dispatcher that owns the request pipeline: routing, credentials,
/// capability guard, budget, reliability, retry loop and adapter invocation.
///
/// This is NOT a public API. Callers use [`Chat`] / [`Thread`].
pub(crate) struct ChatDispatcher {
    router: BackendRouter,
    credential_provider: Arc<dyn CredentialProvider>,
    adapters: HashMap<BackendDialect, Arc<dyn BackendAdapter>>,
    capability_guard: CapabilityGuard,
    budget_enforcer: BudgetEnforcer,
    reliability: ReliabilityLayer,
}

impl ChatDispatcher {
    pub(crate) fn new(
        config: &AIGatewayConfig,
        credential_provider: Arc<dyn CredentialProvider>,
    ) -> Result<Self, GatewayError> {
        let router = BackendRouter::new(config)?;
        Ok(Self {
            router,
            credential_provider,
            adapters: build_default_adapters(),
            capability_guard: CapabilityGuard,
            budget_enforcer: BudgetEnforcer::new(config.budget.clone()),
            reliability: ReliabilityLayer::new(config.reliability.clone()),
        })
    }

    pub(crate) async fn complete(
        &self,
        payload: &TurnPayload,
        route: Option<&str>,
    ) -> Result<DispatchResult, GatewayError> {
        let selected = self.router.select(route)?;
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
            .assert_supported(payload, &capabilities)?;

        let budget_lease = self
            .budget_enforcer
            .pre_dispatch(&payload.limits, &selected.backend_id)
            .await?;

        let correlation_id = format!("{}-{}", selected.backend_id, selected.resolved_model);
        tracing::debug!(
            target: "ai_gateway",
            backend_id = %selected.backend_id,
            dialect = ?selected.profile.dialect,
            model = %selected.resolved_model,
            max_output_tokens = ?payload.limits.max_output_tokens,
            effective_timeout_ms = budget_lease.effective_timeout.as_millis() as u64,
            "dispatch_complete_prepared"
        );

        emit_gateway_event(GatewayTelemetryEvent::RequestStarted {
            request_id: correlation_id.clone(),
            backend_id: selected.backend_id.clone(),
            model: selected.resolved_model.clone(),
            cost_attribution_id: None,
        });

        let mut attempt = 0_u32;
        let mut lease = Some(budget_lease);
        loop {
            emit_gateway_event(GatewayTelemetryEvent::AttemptStarted {
                request_id: correlation_id.clone(),
                attempt,
                cost_attribution_id: None,
            });

            if let Err(err) = self
                .reliability
                .ensure_backend_allowed(&selected.backend_id)
                .await
            {
                release_lease(&self.budget_enforcer, &mut lease);
                emit_gateway_event(GatewayTelemetryEvent::RequestFailed {
                    request_id: correlation_id,
                    attempts: attempt + 1,
                    error_kind: err.kind,
                    cost_attribution_id: None,
                });
                return Err(err);
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
                        std::time::Duration::from_millis(
                            self.reliability.config().request_timeout_ms,
                        )
                    }),
                request_id: correlation_id.clone(),
            };

            let invoke_started_at = Instant::now();
            let complete_result = adapter.complete(adapter_ctx, payload).await;
            tracing::debug!(
                target: "ai_gateway",
                backend_id = %selected.backend_id,
                attempt = attempt,
                elapsed_ms = invoke_started_at.elapsed().as_millis() as u64,
                success = complete_result.is_ok(),
                "dispatch_complete_result"
            );

            match complete_result {
                Ok(complete_response) => {
                    self.reliability.record_success(&selected.backend_id).await;
                    release_lease(&self.budget_enforcer, &mut lease);
                    emit_gateway_event(GatewayTelemetryEvent::RequestCompleted {
                        request_id: correlation_id.clone(),
                        attempts: attempt + 1,
                        usage: complete_response.usage.clone(),
                        cost_attribution_id: None,
                    });

                    let backend_id = complete_response.backend_identity.backend_id.clone();
                    let model_id = complete_response.backend_identity.model.clone();
                    let mut backend_metadata = std::collections::BTreeMap::new();
                    backend_metadata.insert(
                        "backend_id".to_string(),
                        serde_json::Value::String(backend_id.clone()),
                    );
                    backend_metadata.insert(
                        "model".to_string(),
                        serde_json::Value::String(model_id.clone()),
                    );

                    let response = TurnResponse {
                        output_text: complete_response.output_text,
                        tool_calls: complete_response.tool_calls,
                        usage: complete_response.usage,
                        finish_reason: complete_response.finish_reason,
                        backend_metadata,
                    };
                    return Ok(DispatchResult {
                        response,
                        backend_id,
                        model_id,
                    });
                }
                Err(err) => {
                    let can_retry = self.reliability.can_retry(
                        &err,
                        attempt,
                        false,
                        false,
                        &capabilities,
                        adapter.supports_tool_retry(),
                    );
                    emit_gateway_event(GatewayTelemetryEvent::AttemptFailed {
                        request_id: correlation_id.clone(),
                        attempt,
                        kind: err.kind,
                        retryable: err.retryable,
                        cost_attribution_id: None,
                    });
                    self.reliability
                        .record_failure(
                            &selected.backend_id,
                            ReliabilityLayer::counts_toward_breaker(&err),
                        )
                        .await;

                    if can_retry {
                        observability_metrics::increment_chat_task_retries_total(
                            &selected.backend_id,
                            &selected.resolved_model,
                            1,
                        );
                        attempt += 1;
                        sleep(self.reliability.backoff_delay(attempt)).await;
                        continue;
                    }

                    release_lease(&self.budget_enforcer, &mut lease);
                    emit_gateway_event(GatewayTelemetryEvent::RequestFailed {
                        request_id: correlation_id,
                        attempts: attempt + 1,
                        error_kind: err.kind,
                        cost_attribution_id: None,
                    });
                    return Err(err);
                }
            }
        }
    }

    pub(crate) async fn stream(
        &self,
        payload: &TurnPayload,
        route: Option<&str>,
    ) -> Result<ChatEventStream, GatewayError> {
        let selected = self.router.select(route)?;
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
            .assert_supported(payload, &capabilities)?;

        let budget_lease = self
            .budget_enforcer
            .pre_dispatch(&payload.limits, &selected.backend_id)
            .await?;

        let correlation_id = format!("{}-{}", selected.backend_id, selected.resolved_model);
        emit_gateway_event(GatewayTelemetryEvent::RequestStarted {
            request_id: correlation_id.clone(),
            backend_id: selected.backend_id.clone(),
            model: selected.resolved_model.clone(),
            cost_attribution_id: None,
        });

        let (tx, rx) = mpsc::channel::<Result<ChatEvent, GatewayError>>(128);

        let reliability = self.reliability.clone();
        let budget_enforcer = self.budget_enforcer.clone();
        let payload_clone = payload.clone();

        let request_span = tracing::info_span!(
            target: "ai_gateway",
            "dispatch_stream",
            backend_id = %selected.backend_id,
            model = %selected.resolved_model,
        );

        tokio::spawn(
            async move {
                run_stream_task(
                    tx,
                    &payload_clone,
                    selected,
                    adapter,
                    credential,
                    capabilities,
                    reliability,
                    budget_enforcer,
                    budget_lease,
                    correlation_id,
                )
                .await;
            }
            .instrument(request_span),
        );

        Ok(Box::pin(ReceiverStream::new(rx)))
    }
}

pub(crate) struct DispatchResult {
    pub response: TurnResponse,
    pub backend_id: String,
    pub model_id: String,
}

#[allow(clippy::too_many_arguments)]
async fn run_stream_task(
    tx: mpsc::Sender<Result<ChatEvent, GatewayError>>,
    payload: &TurnPayload,
    selected: crate::ai_gateway::router::SelectedBackend,
    adapter: Arc<dyn BackendAdapter>,
    credential: crate::ai_gateway::types::ResolvedCredential,
    capabilities: BackendCapabilities,
    reliability: ReliabilityLayer,
    budget_enforcer: BudgetEnforcer,
    budget_lease: BudgetLease,
    correlation_id: String,
) {
    let mut attempt = 0_u32;
    let mut usage_emitted = false;
    let mut last_usage: Option<UsageStats> = None;
    let mut lease = Some(budget_lease);

    if tx
        .send(Ok(ChatEvent::Started {
            backend_id: selected.backend_id.clone(),
            model_id: selected.resolved_model.clone(),
        }))
        .await
        .is_err()
    {
        release_lease(&budget_enforcer, &mut lease);
        return;
    }

    loop {
        if tx.is_closed() {
            release_lease(&budget_enforcer, &mut lease);
            return;
        }

        emit_gateway_event(GatewayTelemetryEvent::AttemptStarted {
            request_id: correlation_id.clone(),
            attempt,
            cost_attribution_id: None,
        });

        if let Err(err) = reliability
            .ensure_backend_allowed(&selected.backend_id)
            .await
        {
            let _ = tx.send(Ok(ChatEvent::Failed { error: err.clone() })).await;
            release_lease(&budget_enforcer, &mut lease);
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
            request_id: correlation_id.clone(),
        };

        let invocation = adapter.stream(adapter_ctx, payload).await;

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
                reliability
                    .record_failure(
                        &selected.backend_id,
                        ReliabilityLayer::counts_toward_breaker(&err),
                    )
                    .await;

                if can_retry {
                    observability_metrics::increment_chat_task_retries_total(
                        &selected.backend_id,
                        &selected.resolved_model,
                        1,
                    );
                    attempt += 1;
                    let delay = reliability.backoff_delay(attempt);
                    tokio::select! {
                        _ = tx.closed() => {
                            release_lease(&budget_enforcer, &mut lease);
                            return;
                        }
                        _ = sleep(delay) => {}
                    }
                    continue;
                }

                let _ = tx.send(Ok(ChatEvent::Failed { error: err })).await;
                release_lease(&budget_enforcer, &mut lease);
                return;
            }
        };

        let mut terminal_error: Option<GatewayError> = None;
        let mut terminal_success: Option<FinishReason> = None;
        let mut stream = invocation.stream;
        let cancel = invocation.cancel.clone();

        loop {
            tokio::select! {
                _ = tx.closed() => {
                    if let Some(cancel) = cancel.clone() { cancel(); }
                    release_lease(&budget_enforcer, &mut lease);
                    return;
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

                    let event = map_raw_event(raw_event);

                    if event.is_output() { emitted_output = true; }
                    if event.is_tool() { emitted_tool = true; }

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
                            if tx.send(Ok(ChatEvent::Usage { usage })).await.is_err() {
                                if let Some(cancel) = cancel.clone() { cancel(); }
                                release_lease(&budget_enforcer, &mut lease);
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
                                if let Some(cancel) = cancel.clone() { cancel(); }
                                release_lease(&budget_enforcer, &mut lease);
                                return;
                            }
                        }
                    }
                }
            }
        }

        if let Some(finish_reason) = terminal_success {
            reliability.record_success(&selected.backend_id).await;
            let _ = tx.send(Ok(ChatEvent::Completed { finish_reason })).await;
            release_lease(&budget_enforcer, &mut lease);
            emit_gateway_event(GatewayTelemetryEvent::RequestCompleted {
                request_id: correlation_id.clone(),
                attempts: attempt + 1,
                usage: last_usage,
                cost_attribution_id: None,
            });
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

        reliability
            .record_failure(
                &selected.backend_id,
                ReliabilityLayer::counts_toward_breaker(&err),
            )
            .await;

        if can_retry {
            observability_metrics::increment_chat_task_retries_total(
                &selected.backend_id,
                &selected.resolved_model,
                1,
            );
            attempt += 1;
            let delay = reliability.backoff_delay(attempt);
            tokio::select! {
                _ = tx.closed() => {
                    release_lease(&budget_enforcer, &mut lease);
                    return;
                }
                _ = sleep(delay) => {}
            }
            continue;
        }

        let _ = tx.send(Ok(ChatEvent::Failed { error: err.clone() })).await;
        release_lease(&budget_enforcer, &mut lease);
        emit_gateway_event(GatewayTelemetryEvent::RequestFailed {
            request_id: correlation_id,
            attempts: attempt + 1,
            error_kind: err.kind,
            cost_attribution_id: None,
        });
        return;
    }
}

fn map_raw_event(raw: BackendRawEvent) -> ChatEvent {
    match raw {
        BackendRawEvent::OutputTextDelta { delta } => ChatEvent::TextDelta { delta },
        BackendRawEvent::ToolCallDelta {
            call_id,
            name,
            arguments_delta,
        } => ChatEvent::ToolCallDelta {
            call_id,
            name,
            arguments_delta,
        },
        BackendRawEvent::ToolCallReady { call } => ChatEvent::ToolCallReady { call },
        BackendRawEvent::Usage { usage } => ChatEvent::Usage { usage },
        BackendRawEvent::Completed { finish_reason } => ChatEvent::Completed { finish_reason },
        BackendRawEvent::Failed { error } => ChatEvent::Failed { error },
    }
}

fn release_lease(budget_enforcer: &BudgetEnforcer, lease: &mut Option<BudgetLease>) {
    if let Some(current) = lease.take() {
        budget_enforcer.release(current);
    }
}
