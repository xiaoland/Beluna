use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use beluna::{
    ai_gateway::{
        adapters::BackendAdapter,
        credentials::CredentialProvider,
        gateway::AIGateway,
        telemetry::NoopTelemetrySink,
        types::{
            AIGatewayConfig, AdapterContext, AdapterInvocation, BackendCapabilities,
            BackendDialect, BackendIdentity, BackendProfile, BackendRawEvent, CanonicalRequest,
            CredentialRef, ReliabilityConfig, ResolvedCredential,
        },
    },
    cortex::{
        AIGatewayAttemptExtractor, AIGatewayPayloadFiller, AIGatewayPrimaryReasoner,
        AttemptExtractorPort, AttemptExtractorRequest, CapabilityCatalog, PayloadFillerPort,
        PayloadFillerRequest, PrimaryReasonerPort, PrimaryReasonerRequest, ProseIr, ReactionLimits,
        SenseDelta,
    },
};
use futures_util::stream;

#[derive(Default)]
struct StaticCredentialProvider;

#[async_trait]
impl CredentialProvider for StaticCredentialProvider {
    async fn resolve(
        &self,
        _reference: &CredentialRef,
        _backend: &BackendProfile,
    ) -> Result<ResolvedCredential, beluna::ai_gateway::error::GatewayError> {
        Ok(ResolvedCredential::none())
    }
}

struct CapturingAdapter {
    seen_requests: Arc<Mutex<Vec<CanonicalRequest>>>,
    response_text: String,
    capabilities: BackendCapabilities,
}

#[async_trait]
impl BackendAdapter for CapturingAdapter {
    fn dialect(&self) -> BackendDialect {
        BackendDialect::OpenAiCompatible
    }

    fn static_capabilities(&self) -> BackendCapabilities {
        self.capabilities.clone()
    }

    async fn invoke_stream(
        &self,
        _ctx: AdapterContext,
        req: CanonicalRequest,
    ) -> Result<AdapterInvocation, beluna::ai_gateway::error::GatewayError> {
        self.seen_requests.lock().expect("lock").push(req);
        let response_text = self.response_text.clone();
        Ok(AdapterInvocation {
            stream: Box::pin(stream::iter(vec![
                Ok(BackendRawEvent::OutputTextDelta {
                    delta: response_text,
                }),
                Ok(BackendRawEvent::Completed {
                    finish_reason: beluna::ai_gateway::types::FinishReason::Stop,
                }),
            ])),
            backend_identity: BackendIdentity {
                backend_id: "b1".to_string(),
                dialect: BackendDialect::OpenAiCompatible,
                model: "m1".to_string(),
            },
            cancel: None,
        })
    }
}

fn gateway_with(
    adapter: Arc<CapturingAdapter>,
    capabilities: BackendCapabilities,
) -> (Arc<AIGateway>, Arc<Mutex<Vec<CanonicalRequest>>>) {
    let seen = Arc::clone(&adapter.seen_requests);
    let config = AIGatewayConfig {
        default_backend: "b1".to_string(),
        backends: vec![BackendProfile {
            id: "b1".to_string(),
            dialect: BackendDialect::OpenAiCompatible,
            endpoint: Some("https://example.invalid".to_string()),
            credential: CredentialRef::None,
            default_model: "m1".to_string(),
            capabilities: Some(capabilities),
            copilot: None,
        }],
        reliability: ReliabilityConfig::default(),
        budget: Default::default(),
    };

    let mut adapters: HashMap<BackendDialect, Arc<dyn BackendAdapter>> = HashMap::new();
    adapters.insert(BackendDialect::OpenAiCompatible, adapter);

    let gateway = AIGateway::new(
        config,
        Arc::new(StaticCredentialProvider),
        Arc::new(NoopTelemetrySink),
    )
    .expect("gateway build")
    .with_adapters(adapters);

    (Arc::new(gateway), seen)
}

fn primary_req() -> PrimaryReasonerRequest {
    PrimaryReasonerRequest {
        reaction_id: 1,
        prompt_context: "ctx".to_string(),
        sense_window: vec![SenseDelta {
            sense_id: "s1".to_string(),
            source: "sensor".to_string(),
            payload: serde_json::json!({"v":1}),
        }],
        limits: ReactionLimits {
            max_primary_output_tokens: 77,
            ..ReactionLimits::default()
        },
    }
}

#[tokio::test]
async fn given_primary_adapter_when_called_then_request_limits_are_forwarded() {
    let adapter = Arc::new(CapturingAdapter {
        seen_requests: Arc::new(Mutex::new(Vec::new())),
        response_text: "prose ir".to_string(),
        capabilities: BackendCapabilities::default(),
    });
    let (gateway, seen) = gateway_with(adapter, BackendCapabilities::default());
    let primary = AIGatewayPrimaryReasoner::new(gateway, Some("b1".to_string()), None);

    let ir = primary
        .infer_ir(primary_req())
        .await
        .expect("primary should pass");
    assert_eq!(
        ir,
        ProseIr {
            text: "prose ir".to_string()
        }
    );

    let requests = seen.lock().expect("lock");
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].limits.max_output_tokens, Some(77));
}

#[tokio::test]
async fn given_extractor_adapter_when_json_output_then_attempts_are_parsed() {
    let adapter = Arc::new(CapturingAdapter {
        seen_requests: Arc::new(Mutex::new(Vec::new())),
        response_text: r#"{"attempts":[{"intent_span":"x","based_on":["s1"],"attention_tags":["t"],"affordance_key":"a","capability_handle":"c","payload_draft":{},"requested_resources":{"survival_micro":0,"time_ms":0,"io_units":0,"token_units":0}}]}"#.to_string(),
        capabilities: BackendCapabilities {
            streaming: true,
            tool_calls: false,
            json_mode: true,
            vision: false,
            resumable_streaming: false,
        },
    });
    let (gateway, _seen) = gateway_with(
        adapter,
        BackendCapabilities {
            streaming: true,
            tool_calls: false,
            json_mode: true,
            vision: false,
            resumable_streaming: false,
        },
    );
    let extractor = AIGatewayAttemptExtractor::new(gateway, Some("b1".to_string()), None);

    let drafts = extractor
        .extract(AttemptExtractorRequest {
            reaction_id: 2,
            prose_ir: ProseIr {
                text: "ir".to_string(),
            },
            capability_catalog: CapabilityCatalog::default(),
            sense_window: vec![SenseDelta {
                sense_id: "s1".to_string(),
                source: "sensor".to_string(),
                payload: serde_json::json!({}),
            }],
            limits: ReactionLimits::default(),
        })
        .await
        .expect("extract should pass");
    assert_eq!(drafts.len(), 1);
    assert_eq!(drafts[0].based_on, vec!["s1".to_string()]);
}

#[tokio::test]
async fn given_extractor_without_json_capability_when_called_then_error_is_returned() {
    let adapter = Arc::new(CapturingAdapter {
        seen_requests: Arc::new(Mutex::new(Vec::new())),
        response_text: "{}".to_string(),
        capabilities: BackendCapabilities {
            streaming: true,
            tool_calls: false,
            json_mode: false,
            vision: false,
            resumable_streaming: false,
        },
    });
    let (gateway, _seen) = gateway_with(
        adapter,
        BackendCapabilities {
            streaming: true,
            tool_calls: false,
            json_mode: false,
            vision: false,
            resumable_streaming: false,
        },
    );
    let extractor = AIGatewayAttemptExtractor::new(gateway, Some("b1".to_string()), None);
    let err = extractor
        .extract(AttemptExtractorRequest {
            reaction_id: 3,
            prose_ir: ProseIr {
                text: "ir".to_string(),
            },
            capability_catalog: CapabilityCatalog::default(),
            sense_window: vec![SenseDelta {
                sense_id: "s1".to_string(),
                source: "sensor".to_string(),
                payload: serde_json::json!({}),
            }],
            limits: ReactionLimits::default(),
        })
        .await
        .expect_err("extract should fail");
    assert_eq!(
        err.kind,
        beluna::cortex::CortexErrorKind::ExtractorInferenceFailed
    );
}

#[tokio::test]
async fn given_filler_adapter_when_called_then_json_attempts_are_parsed() {
    let adapter = Arc::new(CapturingAdapter {
        seen_requests: Arc::new(Mutex::new(Vec::new())),
        response_text: r#"{"attempts":[{"intent_span":"fix","based_on":["s1"],"attention_tags":[],"affordance_key":"a","capability_handle":"c","payload_draft":{},"requested_resources":{"survival_micro":0,"time_ms":0,"io_units":0,"token_units":0}}]}"#.to_string(),
        capabilities: BackendCapabilities {
            streaming: true,
            tool_calls: false,
            json_mode: true,
            vision: false,
            resumable_streaming: false,
        },
    });
    let (gateway, _seen) = gateway_with(
        adapter,
        BackendCapabilities {
            streaming: true,
            tool_calls: false,
            json_mode: true,
            vision: false,
            resumable_streaming: false,
        },
    );
    let filler = AIGatewayPayloadFiller::new(gateway, Some("b1".to_string()), None);
    let drafts = filler
        .fill(PayloadFillerRequest {
            reaction_id: 9,
            drafts: vec![beluna::cortex::AttemptDraft {
                intent_span: "x".to_string(),
                based_on: vec!["s1".to_string()],
                attention_tags: vec![],
                affordance_key: "a".to_string(),
                capability_handle: "c".to_string(),
                payload_draft: serde_json::json!({}),
                requested_resources: Default::default(),
                commitment_hint: None,
                goal_hint: None,
            }],
            capability_catalog: CapabilityCatalog::default(),
            clamp_violations: vec![],
            limits: ReactionLimits::default(),
        })
        .await
        .expect("fill should pass");
    assert_eq!(drafts.len(), 1);
    assert_eq!(drafts[0].intent_span, "fix");
}
