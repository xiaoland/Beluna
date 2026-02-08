use std::{
    collections::{BTreeMap, HashMap},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

use async_trait::async_trait;
use futures_util::StreamExt;
use tokio_stream::iter;

use crate::ai_gateway::{
    adapters::BackendAdapter,
    credentials::CredentialProvider,
    error::{GatewayError, GatewayErrorKind},
    gateway::AIGateway,
    telemetry::NoopTelemetrySink,
    types::{
        AIGatewayConfig, AdapterContext, AdapterInvocation, BackendCapabilities, BackendDialect,
        BackendIdentity, BackendProfile, BelunaContentPart, BelunaInferenceRequest, BelunaMessage,
        BelunaRole, BudgetConfig, CanonicalRequest, CredentialRef, GatewayEvent, OutputMode,
        ReliabilityConfig, RequestLimitOverrides, ResolvedCredential, ToolChoice,
    },
};

#[derive(Default)]
struct StaticCredentialProvider;

#[async_trait]
impl CredentialProvider for StaticCredentialProvider {
    async fn resolve(
        &self,
        _reference: &CredentialRef,
        _backend: &BackendProfile,
    ) -> Result<ResolvedCredential, GatewayError> {
        Ok(ResolvedCredential::none())
    }
}

struct RetryOnceMockAdapter {
    calls: Arc<AtomicUsize>,
}

#[async_trait]
impl BackendAdapter for RetryOnceMockAdapter {
    fn dialect(&self) -> BackendDialect {
        BackendDialect::OpenAiCompatible
    }

    fn static_capabilities(&self) -> BackendCapabilities {
        BackendCapabilities {
            streaming: true,
            tool_calls: false,
            json_mode: false,
            vision: false,
            resumable_streaming: false,
        }
    }

    async fn invoke_stream(
        &self,
        ctx: AdapterContext,
        _req: CanonicalRequest,
    ) -> Result<AdapterInvocation, GatewayError> {
        let previous = self.calls.fetch_add(1, Ordering::SeqCst);
        if previous == 0 {
            return Err(GatewayError::new(
                GatewayErrorKind::BackendTransient,
                "first attempt fails",
            )
            .with_retryable(true)
            .with_backend_id(ctx.backend_id));
        }

        Ok(AdapterInvocation {
            stream: Box::pin(iter(vec![
                Ok(crate::ai_gateway::types::BackendRawEvent::OutputTextDelta {
                    delta: "ok".to_string(),
                }),
                Ok(crate::ai_gateway::types::BackendRawEvent::Completed {
                    finish_reason: crate::ai_gateway::types::FinishReason::Stop,
                }),
            ])),
            backend_identity: BackendIdentity {
                backend_id: "openai-default".to_string(),
                dialect: BackendDialect::OpenAiCompatible,
                model: "m1".to_string(),
            },
            cancel: None,
        })
    }
}

struct OutputThenFailAdapter {
    calls: Arc<AtomicUsize>,
}

#[async_trait]
impl BackendAdapter for OutputThenFailAdapter {
    fn dialect(&self) -> BackendDialect {
        BackendDialect::OpenAiCompatible
    }

    fn static_capabilities(&self) -> BackendCapabilities {
        BackendCapabilities {
            streaming: true,
            tool_calls: false,
            json_mode: false,
            vision: false,
            resumable_streaming: false,
        }
    }

    async fn invoke_stream(
        &self,
        _ctx: AdapterContext,
        _req: CanonicalRequest,
    ) -> Result<AdapterInvocation, GatewayError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(AdapterInvocation {
            stream: Box::pin(iter(vec![
                Ok(crate::ai_gateway::types::BackendRawEvent::OutputTextDelta {
                    delta: "partial".to_string(),
                }),
                Ok(crate::ai_gateway::types::BackendRawEvent::Failed {
                    error: GatewayError::new(GatewayErrorKind::BackendTransient, "stream failed")
                        .with_retryable(true),
                }),
            ])),
            backend_identity: BackendIdentity {
                backend_id: "openai-default".to_string(),
                dialect: BackendDialect::OpenAiCompatible,
                model: "m1".to_string(),
            },
            cancel: None,
        })
    }
}

fn gateway_config() -> AIGatewayConfig {
    AIGatewayConfig {
        default_backend: "openai-default".to_string(),
        backends: vec![BackendProfile {
            id: "openai-default".to_string(),
            dialect: BackendDialect::OpenAiCompatible,
            endpoint: Some("https://example.invalid/v1".to_string()),
            credential: CredentialRef::None,
            default_model: "m1".to_string(),
            capabilities: Some(BackendCapabilities {
                streaming: true,
                tool_calls: false,
                json_mode: false,
                vision: false,
                resumable_streaming: false,
            }),
            copilot: None,
        }],
        reliability: ReliabilityConfig {
            request_timeout_ms: 30_000,
            max_retries: 2,
            backoff_base_ms: 1,
            backoff_max_ms: 2,
            retry_policy: crate::ai_gateway::types::RetryPolicy::BeforeFirstEventOnly,
            breaker_failure_threshold: 100,
            breaker_open_ms: 1000,
        },
        budget: BudgetConfig::default(),
    }
}

fn request() -> BelunaInferenceRequest {
    BelunaInferenceRequest {
        request_id: None,
        backend_id: Some("openai-default".to_string()),
        model: None,
        messages: vec![BelunaMessage {
            role: BelunaRole::User,
            parts: vec![BelunaContentPart::Text {
                text: "hello".to_string(),
            }],
            tool_call_id: None,
            tool_name: None,
        }],
        tools: vec![],
        tool_choice: ToolChoice::Auto,
        output_mode: OutputMode::Text,
        limits: RequestLimitOverrides::default(),
        metadata: BTreeMap::new(),
        stream: true,
    }
}

#[tokio::test]
async fn retries_before_first_output_and_succeeds() {
    let calls = Arc::new(AtomicUsize::new(0));
    let adapter = Arc::new(RetryOnceMockAdapter {
        calls: Arc::clone(&calls),
    });

    let mut adapters: HashMap<BackendDialect, Arc<dyn BackendAdapter>> = HashMap::new();
    adapters.insert(BackendDialect::OpenAiCompatible, adapter);

    let gateway = AIGateway::new(
        gateway_config(),
        Arc::new(StaticCredentialProvider),
        Arc::new(NoopTelemetrySink),
    )
    .expect("gateway should build")
    .with_adapters(adapters);

    let response = gateway
        .infer_once(request())
        .await
        .expect("request should succeed after retry");
    assert_eq!(response.output_text, "ok");
    assert_eq!(calls.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn does_not_retry_after_output_event() {
    let calls = Arc::new(AtomicUsize::new(0));
    let adapter = Arc::new(OutputThenFailAdapter {
        calls: Arc::clone(&calls),
    });

    let mut adapters: HashMap<BackendDialect, Arc<dyn BackendAdapter>> = HashMap::new();
    adapters.insert(BackendDialect::OpenAiCompatible, adapter);

    let gateway = AIGateway::new(
        gateway_config(),
        Arc::new(StaticCredentialProvider),
        Arc::new(NoopTelemetrySink),
    )
    .expect("gateway should build")
    .with_adapters(adapters);

    let err = gateway
        .infer_once(request())
        .await
        .expect_err("request should fail without retry after output");
    assert_eq!(err.kind, GatewayErrorKind::BackendTransient);
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn emits_started_then_terminal_for_failed_stream() {
    let calls = Arc::new(AtomicUsize::new(0));
    let adapter = Arc::new(OutputThenFailAdapter {
        calls: Arc::clone(&calls),
    });

    let mut adapters: HashMap<BackendDialect, Arc<dyn BackendAdapter>> = HashMap::new();
    adapters.insert(BackendDialect::OpenAiCompatible, adapter);

    let gateway = AIGateway::new(
        gateway_config(),
        Arc::new(StaticCredentialProvider),
        Arc::new(NoopTelemetrySink),
    )
    .expect("gateway should build")
    .with_adapters(adapters);

    let mut stream = gateway
        .infer_stream(request())
        .await
        .expect("stream should start");
    let first = stream
        .next()
        .await
        .expect("started event")
        .expect("started event should be ok");
    assert!(matches!(first, GatewayEvent::Started { .. }));
}
