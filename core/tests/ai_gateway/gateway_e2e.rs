use std::{
    collections::{BTreeMap, HashMap},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::Duration,
};

use async_trait::async_trait;
use futures_util::StreamExt;
use tokio::time::{sleep, timeout};
use tokio_stream::iter;

use beluna::ai_gateway::{
    adapters::BackendAdapter,
    credentials::CredentialProvider,
    error::{GatewayError, GatewayErrorKind},
    gateway::AIGateway,
    types::{
        AIGatewayConfig, AdapterContext, BackendCapabilities, BackendDialect, BackendProfile,
        BudgetConfig, CredentialRef, ModelProfile, ModelTarget, ReliabilityConfig,
        ResolvedCredential, RetryPolicy,
    },
    types_chat::{
        AdapterInvocation, BackendIdentity, BackendRawEvent, BelunaContentPart, BelunaMessage,
        BelunaRole, CanonicalRequest, ChatEvent, ChatRequest, FinishReason, OutputMode,
        RequestLimitOverrides, ToolChoice, UsageStats,
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
            json_schema_mode: false,
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
                Ok(BackendRawEvent::OutputTextDelta {
                    delta: "ok".to_string(),
                }),
                Ok(BackendRawEvent::Completed {
                    finish_reason: FinishReason::Stop,
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
            json_schema_mode: false,
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
                Ok(BackendRawEvent::OutputTextDelta {
                    delta: "partial".to_string(),
                }),
                Ok(BackendRawEvent::Failed {
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

struct UsageOverBudgetThenCompleteAdapter;

#[async_trait]
impl BackendAdapter for UsageOverBudgetThenCompleteAdapter {
    fn dialect(&self) -> BackendDialect {
        BackendDialect::OpenAiCompatible
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
        _ctx: AdapterContext,
        _req: CanonicalRequest,
    ) -> Result<AdapterInvocation, GatewayError> {
        Ok(AdapterInvocation {
            stream: Box::pin(iter(vec![
                Ok(BackendRawEvent::Usage {
                    usage: UsageStats {
                        input_tokens: Some(10),
                        output_tokens: Some(20),
                        total_tokens: Some(30),
                        provider_usage_raw: None,
                    },
                }),
                Ok(BackendRawEvent::OutputTextDelta {
                    delta: "still-running".to_string(),
                }),
                Ok(BackendRawEvent::Completed {
                    finish_reason: FinishReason::Stop,
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

struct CancelAwarePendingAdapter {
    invoked: Arc<AtomicBool>,
    cancel_called: Arc<AtomicBool>,
}

#[async_trait]
impl BackendAdapter for CancelAwarePendingAdapter {
    fn dialect(&self) -> BackendDialect {
        BackendDialect::OpenAiCompatible
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
        _ctx: AdapterContext,
        _req: CanonicalRequest,
    ) -> Result<AdapterInvocation, GatewayError> {
        self.invoked.store(true, Ordering::SeqCst);
        let cancel_called = Arc::clone(&self.cancel_called);

        Ok(AdapterInvocation {
            stream: Box::pin(futures_util::stream::pending()),
            backend_identity: BackendIdentity {
                backend_id: "openai-default".to_string(),
                dialect: BackendDialect::OpenAiCompatible,
                model: "m1".to_string(),
            },
            cancel: Some(Arc::new(move || {
                cancel_called.store(true, Ordering::SeqCst);
            })),
        })
    }
}

fn gateway_config() -> AIGatewayConfig {
    gateway_config_with_budget(BudgetConfig::default())
}

fn gateway_config_with_budget(budget: BudgetConfig) -> AIGatewayConfig {
    let mut route_aliases = BTreeMap::new();
    route_aliases.insert(
        "default".to_string(),
        ModelTarget {
            backend_id: "openai-default".to_string(),
            model_id: "m1".to_string(),
        },
    );

    AIGatewayConfig {
        backends: vec![BackendProfile {
            id: "openai-default".to_string(),
            dialect: BackendDialect::OpenAiCompatible,
            endpoint: Some("https://example.invalid/v1".to_string()),
            credential: CredentialRef::None,
            models: vec![ModelProfile {
                id: "m1".to_string(),
            }],
            capabilities: Some(BackendCapabilities {
                streaming: true,
                tool_calls: false,
                json_mode: false,
                json_schema_mode: false,
                vision: false,
                resumable_streaming: false,
            }),
            copilot: None,
        }],
        route_aliases,
        reliability: ReliabilityConfig {
            request_timeout_ms: 30_000,
            max_retries: 2,
            backoff_base_ms: 1,
            backoff_max_ms: 2,
            retry_policy: RetryPolicy::BeforeFirstEventOnly,
            breaker_failure_threshold: 100,
            breaker_open_ms: 1000,
        },
        budget,
    }
}

fn request() -> ChatRequest {
    ChatRequest {
        request_id: None,
        route: Some("default".to_string()),
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
        cost_attribution_id: None,
    }
}

#[tokio::test]
async fn given_json_schema_output_when_backend_lacks_schema_capability_then_request_fails_fast() {
    let adapter = Arc::new(RetryOnceMockAdapter {
        calls: Arc::new(AtomicUsize::new(0)),
    });
    let mut adapters: HashMap<BackendDialect, Arc<dyn BackendAdapter>> = HashMap::new();
    adapters.insert(BackendDialect::OpenAiCompatible, adapter);

    let gateway = AIGateway::new(gateway_config(), Arc::new(StaticCredentialProvider))
        .expect("gateway should build")
        .with_adapters(adapters);

    let mut req = request();
    req.output_mode = OutputMode::JsonSchema {
        name: "acts_helper_output".to_string(),
        schema: serde_json::json!({
            "type": "object",
            "properties": { "acts": { "type": "array" } },
            "required": ["acts"]
        }),
        strict: true,
    };

    let err = gateway
        .chat_once(req)
        .await
        .expect_err("json schema request should fail on unsupported capability");
    assert_eq!(err.kind, GatewayErrorKind::UnsupportedCapability);
}

#[tokio::test]
async fn given_transient_failure_before_output_when_chat_once_then_gateway_retries_and_succeeds() {
    let calls = Arc::new(AtomicUsize::new(0));
    let adapter = Arc::new(RetryOnceMockAdapter {
        calls: Arc::clone(&calls),
    });

    let mut adapters: HashMap<BackendDialect, Arc<dyn BackendAdapter>> = HashMap::new();
    adapters.insert(BackendDialect::OpenAiCompatible, adapter);

    let gateway = AIGateway::new(gateway_config(), Arc::new(StaticCredentialProvider))
        .expect("gateway should build")
        .with_adapters(adapters);

    let response = gateway
        .chat_once(request())
        .await
        .expect("request should succeed after retry");
    assert_eq!(response.output_text, "ok");
    assert_eq!(calls.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn given_failure_after_output_when_chat_once_then_gateway_does_not_retry() {
    let calls = Arc::new(AtomicUsize::new(0));
    let adapter = Arc::new(OutputThenFailAdapter {
        calls: Arc::clone(&calls),
    });

    let mut adapters: HashMap<BackendDialect, Arc<dyn BackendAdapter>> = HashMap::new();
    adapters.insert(BackendDialect::OpenAiCompatible, adapter);

    let gateway = AIGateway::new(gateway_config(), Arc::new(StaticCredentialProvider))
        .expect("gateway should build")
        .with_adapters(adapters);

    let err = gateway
        .chat_once(request())
        .await
        .expect_err("request should fail without retry after output");
    assert_eq!(err.kind, GatewayErrorKind::BackendTransient);
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn given_failing_stream_when_chat_stream_then_started_is_first_and_terminal_is_emitted_once()
{
    let calls = Arc::new(AtomicUsize::new(0));
    let adapter = Arc::new(OutputThenFailAdapter {
        calls: Arc::clone(&calls),
    });

    let mut adapters: HashMap<BackendDialect, Arc<dyn BackendAdapter>> = HashMap::new();
    adapters.insert(BackendDialect::OpenAiCompatible, adapter);

    let gateway = AIGateway::new(gateway_config(), Arc::new(StaticCredentialProvider))
        .expect("gateway should build")
        .with_adapters(adapters);

    let mut stream = gateway
        .chat_stream(request())
        .await
        .expect("stream should start");
    let first = stream
        .next()
        .await
        .expect("started event")
        .expect("started event should be ok");
    assert!(matches!(first, ChatEvent::Started { .. }));

    let mut terminal_count = 0;
    while let Some(event) = stream.next().await {
        let event = event.expect("gateway event should be valid");
        if matches!(
            event,
            ChatEvent::Completed { .. } | ChatEvent::Failed { .. }
        ) {
            terminal_count += 1;
            assert!(
                matches!(event, ChatEvent::Failed { .. }),
                "this failure-path adapter must terminate with Failed event",
            );
        }
    }

    assert_eq!(terminal_count, 1, "exactly one terminal event is required");
}

#[tokio::test]
async fn given_usage_over_budget_post_check_when_chat_once_then_stream_still_completes() {
    let adapter = Arc::new(UsageOverBudgetThenCompleteAdapter);

    let mut adapters: HashMap<BackendDialect, Arc<dyn BackendAdapter>> = HashMap::new();
    adapters.insert(BackendDialect::OpenAiCompatible, adapter);

    let gateway = AIGateway::new(
        gateway_config_with_budget(BudgetConfig {
            max_request_time_ms: 45_000,
            max_usage_tokens_per_request: Some(1),
            max_concurrency_per_backend: 8,
            rate_smoothing_per_second: None,
        }),
        Arc::new(StaticCredentialProvider),
    )
    .expect("gateway should build")
    .with_adapters(adapters);

    let response = gateway
        .chat_once(request())
        .await
        .expect("usage post-check must not terminate active stream");
    assert_eq!(response.output_text, "still-running");
    assert!(matches!(response.finish_reason, FinishReason::Stop));
    assert_eq!(
        response.usage.as_ref().and_then(|usage| usage.total_tokens),
        Some(30)
    );
}

#[tokio::test]
async fn given_stream_drop_when_inflight_then_adapter_is_cancelled_and_budget_is_released() {
    let invoked = Arc::new(AtomicBool::new(false));
    let cancel_called = Arc::new(AtomicBool::new(false));
    let adapter = Arc::new(CancelAwarePendingAdapter {
        invoked: Arc::clone(&invoked),
        cancel_called: Arc::clone(&cancel_called),
    });

    let mut adapters: HashMap<BackendDialect, Arc<dyn BackendAdapter>> = HashMap::new();
    adapters.insert(BackendDialect::OpenAiCompatible, adapter);

    let gateway = AIGateway::new(
        gateway_config_with_budget(BudgetConfig {
            max_request_time_ms: 45_000,
            max_usage_tokens_per_request: None,
            max_concurrency_per_backend: 1,
            rate_smoothing_per_second: None,
        }),
        Arc::new(StaticCredentialProvider),
    )
    .expect("gateway should build")
    .with_adapters(adapters);

    let mut first_stream = gateway
        .chat_stream(request())
        .await
        .expect("first stream should start");
    let first_started = first_stream
        .next()
        .await
        .expect("first started event should exist")
        .expect("first started event should be valid");
    assert!(matches!(first_started, ChatEvent::Started { .. }));

    timeout(Duration::from_millis(300), async {
        while !invoked.load(Ordering::SeqCst) {
            sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .expect("adapter invocation should start");

    drop(first_stream);

    timeout(Duration::from_millis(500), async {
        while !cancel_called.load(Ordering::SeqCst) {
            sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .expect("adapter cancel handle should be called");

    let mut second_stream = timeout(Duration::from_millis(500), gateway.chat_stream(request()))
        .await
        .expect("second chat_stream should not block after release")
        .expect("second stream should start");
    let second_started = timeout(Duration::from_millis(500), second_stream.next())
        .await
        .expect("second started event should arrive")
        .expect("second started event should exist")
        .expect("second started event should be valid");
    assert!(matches!(second_started, ChatEvent::Started { .. }));
}
