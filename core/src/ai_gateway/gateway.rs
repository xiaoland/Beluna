use std::{collections::HashMap, sync::Arc};

use futures_util::StreamExt;
use tokio::{sync::mpsc, time::sleep};
use tokio_stream::wrappers::ReceiverStream;

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
    telemetry::{GatewayTelemetryEvent, TelemetrySink},
    types::{
        AIGatewayConfig, AdapterContext, BackendCapabilities, BackendDialect,
        BelunaInferenceRequest, CanonicalFinalResponse, CanonicalRequest, FinishReason,
        GatewayEvent, GatewayEventStream, UsageStats,
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
    telemetry: Arc<dyn TelemetrySink>,
}

impl AIGateway {
    pub fn new(
        config: AIGatewayConfig,
        credential_provider: Arc<dyn CredentialProvider>,
        telemetry: Arc<dyn TelemetrySink>,
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
            telemetry,
        })
    }

    pub fn with_adapters(
        mut self,
        adapters: HashMap<BackendDialect, Arc<dyn BackendAdapter>>,
    ) -> Self {
        self.adapters = adapters;
        self
    }

    pub async fn infer_stream(
        &self,
        request: BelunaInferenceRequest,
    ) -> Result<GatewayEventStream, GatewayError> {
        let canonical_request = self.request_normalizer.normalize(request)?;
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

        self.telemetry
            .on_event(GatewayTelemetryEvent::RequestStarted {
                request_id: canonical_request.request_id.clone(),
                backend_id: selected.backend_id.clone(),
                model: selected.resolved_model.clone(),
                cost_attribution_id: canonical_request.cost_attribution_id.clone(),
            });

        let (tx, rx) = mpsc::channel::<Result<GatewayEvent, GatewayError>>(128);

        let response_normalizer = self.response_normalizer;
        let reliability = self.reliability.clone();
        let budget_enforcer = self.budget_enforcer.clone();
        let telemetry = self.telemetry.clone();

        tokio::spawn(async move {
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
                telemetry,
                budget_lease,
            )
            .await;
        });

        Ok(Box::pin(ReceiverStream::new(rx)))
    }

    pub async fn infer_once(
        &self,
        request: BelunaInferenceRequest,
    ) -> Result<CanonicalFinalResponse, GatewayError> {
        let mut stream = self.infer_stream(request).await?;
        let mut output_text = String::new();
        let mut tool_calls = Vec::new();
        let mut usage: Option<UsageStats> = None;
        let mut finish_reason: Option<FinishReason> = None;
        let mut request_id: Option<String> = None;

        while let Some(item) = stream.next().await {
            let event = item?;
            match event {
                GatewayEvent::Started { request_id: id, .. } => {
                    request_id = Some(id);
                }
                GatewayEvent::OutputTextDelta { delta, .. } => {
                    output_text.push_str(&delta);
                }
                GatewayEvent::ToolCallReady { call, .. } => {
                    tool_calls.push(call);
                }
                GatewayEvent::Usage { usage: current, .. } => {
                    usage = Some(current);
                }
                GatewayEvent::Completed {
                    request_id: id,
                    finish_reason: reason,
                } => {
                    request_id = Some(id);
                    finish_reason = Some(reason);
                    break;
                }
                GatewayEvent::Failed { error, .. } => {
                    return Err(error);
                }
                GatewayEvent::ToolCallDelta { .. } => {}
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

        Ok(CanonicalFinalResponse {
            request_id,
            output_text,
            tool_calls,
            usage,
            finish_reason,
            backend_metadata: Default::default(),
        })
    }
}

#[allow(clippy::too_many_arguments)]
async fn run_stream_task(
    tx: mpsc::Sender<Result<GatewayEvent, GatewayError>>,
    canonical_request: CanonicalRequest,
    selected: crate::ai_gateway::router::SelectedBackend,
    adapter: Arc<dyn BackendAdapter>,
    credential: crate::ai_gateway::types::ResolvedCredential,
    capabilities: BackendCapabilities,
    response_normalizer: ResponseNormalizer,
    reliability: ReliabilityLayer,
    budget_enforcer: BudgetEnforcer,
    telemetry: Arc<dyn TelemetrySink>,
    budget_lease: BudgetLease,
) {
    let request_id = canonical_request.request_id.clone();
    let cost_attribution_id = canonical_request.cost_attribution_id.clone();
    let mut attempt = 0_u32;
    let mut usage_emitted = false;
    let mut last_usage: Option<UsageStats> = None;
    let mut lease = Some(budget_lease);

    if tx
        .send(Ok(GatewayEvent::Started {
            request_id: request_id.clone(),
            backend_id: selected.backend_id.clone(),
            model: selected.resolved_model.clone(),
        }))
        .await
        .is_err()
    {
        release_lease(&budget_enforcer, &mut lease);
        telemetry.on_event(GatewayTelemetryEvent::RequestCancelled {
            request_id,
            cost_attribution_id,
        });
        return;
    }

    loop {
        if tx.is_closed() {
            release_lease(&budget_enforcer, &mut lease);
            telemetry.on_event(GatewayTelemetryEvent::RequestCancelled {
                request_id: request_id.clone(),
                cost_attribution_id: cost_attribution_id.clone(),
            });
            return;
        }

        telemetry.on_event(GatewayTelemetryEvent::AttemptStarted {
            request_id: request_id.clone(),
            attempt,
            cost_attribution_id: cost_attribution_id.clone(),
        });

        if let Err(err) = reliability
            .ensure_backend_allowed(&selected.backend_id)
            .await
        {
            let _ = tx
                .send(Ok(GatewayEvent::Failed {
                    request_id: request_id.clone(),
                    error: err.clone(),
                }))
                .await;
            release_lease(&budget_enforcer, &mut lease);
            telemetry.on_event(GatewayTelemetryEvent::RequestFailed {
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

        let invocation = adapter
            .invoke_stream(adapter_ctx, canonical_request.clone())
            .await;

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

                telemetry.on_event(GatewayTelemetryEvent::AttemptFailed {
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
                            telemetry.on_event(GatewayTelemetryEvent::RequestCancelled {
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
                    .send(Ok(GatewayEvent::Failed {
                        request_id: request_id.clone(),
                        error: err.clone(),
                    }))
                    .await;
                release_lease(&budget_enforcer, &mut lease);
                telemetry.on_event(GatewayTelemetryEvent::RequestFailed {
                    request_id,
                    attempts: attempt + 1,
                    error_kind: err.kind,
                    cost_attribution_id,
                });
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
                    if let Some(cancel) = cancel.clone() {
                        cancel();
                    }
                    release_lease(&budget_enforcer, &mut lease);
                    telemetry.on_event(GatewayTelemetryEvent::RequestCancelled {
                        request_id: request_id.clone(),
                        cost_attribution_id: cost_attribution_id.clone(),
                    });
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

                    let event = match response_normalizer.map_raw(&request_id, raw_event) {
                        Ok(event) => event,
                        Err(err) => {
                            terminal_error = Some(err.with_backend_id(selected.backend_id.clone()));
                            break;
                        }
                    };

                    if ResponseNormalizer::is_output_event(&event) {
                        emitted_output = true;
                    }
                    if ResponseNormalizer::is_tool_event(&event) {
                        emitted_tool = true;
                    }

                    match event {
                        GatewayEvent::Usage { usage, .. } => {
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
                            let usage_event = GatewayEvent::Usage {
                                request_id: request_id.clone(),
                                usage,
                            };
                            budget_enforcer.observe_event(&selected.backend_id, &usage_event).await;
                            if tx.send(Ok(usage_event)).await.is_err() {
                                if let Some(cancel) = cancel.clone() {
                                    cancel();
                                }
                                release_lease(&budget_enforcer, &mut lease);
                                telemetry.on_event(GatewayTelemetryEvent::RequestCancelled {
                                    request_id: request_id.clone(),
                                    cost_attribution_id: cost_attribution_id.clone(),
                                });
                                return;
                            }
                        }
                        GatewayEvent::Completed { finish_reason, .. } => {
                            terminal_success = Some(finish_reason);
                            break;
                        }
                        GatewayEvent::Failed { error, .. } => {
                            terminal_error = Some(error);
                            break;
                        }
                        non_terminal => {
                            if tx.send(Ok(non_terminal)).await.is_err() {
                                if let Some(cancel) = cancel.clone() {
                                    cancel();
                                }
                                release_lease(&budget_enforcer, &mut lease);
                                telemetry.on_event(GatewayTelemetryEvent::RequestCancelled {
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
            reliability.record_success(&selected.backend_id).await;
            let _ = tx
                .send(Ok(GatewayEvent::Completed {
                    request_id: request_id.clone(),
                    finish_reason,
                }))
                .await;
            release_lease(&budget_enforcer, &mut lease);
            telemetry.on_event(GatewayTelemetryEvent::RequestCompleted {
                request_id,
                attempts: attempt + 1,
                usage: last_usage,
                cost_attribution_id,
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

        telemetry.on_event(GatewayTelemetryEvent::AttemptFailed {
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
                    telemetry.on_event(GatewayTelemetryEvent::RequestCancelled {
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
            .send(Ok(GatewayEvent::Failed {
                request_id: request_id.clone(),
                error: err.clone(),
            }))
            .await;
        release_lease(&budget_enforcer, &mut lease);
        telemetry.on_event(GatewayTelemetryEvent::RequestFailed {
            request_id,
            attempts: attempt + 1,
            error_kind: err.kind,
            cost_attribution_id,
        });
        return;
    }
}

fn release_lease(budget_enforcer: &BudgetEnforcer, lease: &mut Option<BudgetLease>) {
    if let Some(current) = lease.take() {
        budget_enforcer.release(current);
    }
}
