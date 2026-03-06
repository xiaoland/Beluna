use std::{
    collections::BTreeMap,
    sync::atomic::{AtomicU64, Ordering},
};

use tokio::time::sleep;

use crate::{
    ai_gateway::{
        adapters::BackendAdapter,
        credentials::CredentialProvider,
        error::{GatewayError, GatewayErrorKind},
        resilience::{ResilienceEngine, ResilienceLease},
        router::BackendRouter,
        telemetry::{GatewayTelemetryEvent, emit_gateway_event},
        types::{
            AdapterContext, BackendCapabilities, BackendDialect, BackendId, BackendProfile,
            ResolvedCredential,
        },
    },
    observability::metrics as observability_metrics,
};

use super::{capabilities::CapabilityGuard, types::{TurnPayload, TurnResponse}};

pub(crate) struct ChatRuntime {
    pub router: BackendRouter,
    pub credential_provider: std::sync::Arc<dyn CredentialProvider>,
    pub adapters: std::collections::HashMap<BackendDialect, std::sync::Arc<dyn BackendAdapter>>,
    pub capability_guard: CapabilityGuard,
    pub resilience: ResilienceEngine,
    pub default_route: Option<String>,
    pub default_turn_timeout_ms: u64,
}

#[derive(Clone)]
pub(crate) struct BoundBackend {
    pub backend_id: BackendId,
    pub model: String,
    pub profile: BackendProfile,
    pub credential: ResolvedCredential,
    pub capabilities: BackendCapabilities,
    pub adapter: std::sync::Arc<dyn BackendAdapter>,
}

impl ChatRuntime {
    pub(crate) async fn dispatch_complete(
        &self,
        backend: &BoundBackend,
        payload: &TurnPayload,
    ) -> Result<TurnResponse, GatewayError> {
        self.capability_guard
            .assert_supported(payload, &backend.capabilities)?;

        let lease = self
            .resilience
            .pre_dispatch(&payload.limits, &backend.backend_id)
            .await?;
        let mut lease = Some(lease);

        let request_id = next_request_id(&backend.backend_id, &backend.model);
        emit_gateway_event(GatewayTelemetryEvent::RequestStarted {
            request_id: request_id.clone(),
            backend_id: backend.backend_id.clone(),
            model: backend.model.clone(),
            cost_attribution_id: None,
        });

        let mut attempt = 0_u32;
        loop {
            emit_gateway_event(GatewayTelemetryEvent::AttemptStarted {
                request_id: request_id.clone(),
                attempt,
                cost_attribution_id: None,
            });

            if let Err(err) = self
                .resilience
                .ensure_backend_allowed(&backend.backend_id)
                .await
            {
                release_lease(&self.resilience, &mut lease);
                emit_gateway_event(GatewayTelemetryEvent::RequestFailed {
                    request_id,
                    attempts: attempt + 1,
                    error_kind: err.kind,
                    cost_attribution_id: None,
                });
                return Err(err);
            }

            let adapter_ctx = AdapterContext {
                backend_id: backend.backend_id.clone(),
                model: backend.model.clone(),
                profile: backend.profile.clone(),
                credential: backend.credential.clone(),
                timeout: lease
                    .as_ref()
                    .map(|item| item.effective_timeout)
                    .unwrap_or_else(|| {
                        std::time::Duration::from_millis(self.resilience.config().request_timeout_ms)
                    }),
                request_id: request_id.clone(),
            };

            match backend.adapter.complete(adapter_ctx, payload).await {
                Ok(complete_response) => {
                    self.resilience.record_success(&backend.backend_id).await;
                    release_lease(&self.resilience, &mut lease);

                    emit_gateway_event(GatewayTelemetryEvent::RequestCompleted {
                        request_id,
                        attempts: attempt + 1,
                        usage: complete_response.usage.clone(),
                        cost_attribution_id: None,
                    });

                    let mut backend_metadata = BTreeMap::new();
                    backend_metadata.insert(
                        "backend_id".to_string(),
                        serde_json::Value::String(backend.backend_id.clone()),
                    );
                    backend_metadata.insert(
                        "model".to_string(),
                        serde_json::Value::String(backend.model.clone()),
                    );

                    return Ok(TurnResponse {
                        output_text: complete_response.output_text,
                        tool_calls: complete_response.tool_calls,
                        pending_tool_call_continuation: false,
                        usage: complete_response.usage,
                        finish_reason: complete_response.finish_reason,
                        backend_metadata,
                    });
                }
                Err(err) => {
                    let can_retry = self.resilience.can_retry(
                        &err,
                        attempt,
                        false,
                        false,
                        &backend.capabilities,
                        backend.adapter.supports_tool_retry(),
                    );
                    self.resilience
                        .record_failure(
                            &backend.backend_id,
                            ResilienceEngine::counts_toward_breaker(&err),
                        )
                        .await;

                    emit_gateway_event(GatewayTelemetryEvent::AttemptFailed {
                        request_id: request_id.clone(),
                        attempt,
                        kind: err.kind,
                        retryable: err.retryable,
                        cost_attribution_id: None,
                    });

                    if can_retry {
                        observability_metrics::increment_chat_task_retries_total(
                            &backend.backend_id,
                            &backend.model,
                            1,
                        );
                        attempt = attempt.saturating_add(1);
                        sleep(self.resilience.backoff_delay(attempt)).await;
                        continue;
                    }

                    release_lease(&self.resilience, &mut lease);
                    emit_gateway_event(GatewayTelemetryEvent::RequestFailed {
                        request_id,
                        attempts: attempt + 1,
                        error_kind: err.kind,
                        cost_attribution_id: None,
                    });
                    return Err(err);
                }
            }
        }
    }

    pub(crate) async fn resolve_backend(
        &self,
        route: Option<&str>,
    ) -> Result<BoundBackend, GatewayError> {
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

        Ok(BoundBackend {
            backend_id: selected.backend_id,
            model: selected.resolved_model,
            profile: selected.profile,
            credential,
            capabilities,
            adapter,
        })
    }
}

fn next_request_id(backend_id: &str, model: &str) -> String {
    static SEQ: AtomicU64 = AtomicU64::new(1);
    let seq = SEQ.fetch_add(1, Ordering::Relaxed);
    format!("{}-{}-{}", backend_id, model, seq)
}

fn release_lease(resilience: &ResilienceEngine, lease: &mut Option<ResilienceLease>) {
    if let Some(current) = lease.take() {
        resilience.release(current);
    }
}
