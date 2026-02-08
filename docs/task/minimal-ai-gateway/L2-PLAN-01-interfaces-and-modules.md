# L2-01 - Interfaces and Modules

- Task Name: `minimal-ai-gateway`
- Stage: `L2` detail: interfaces/modules
- Date: 2026-02-08
- Status: `DRAFT_FOR_APPROVAL`

## 1) Target Module Layout

```text
src/
  ai_gateway/
    mod.rs
    gateway.rs
    router.rs
    request_normalizer.rs
    response_normalizer.rs
    capabilities.rs
    credentials.rs
    budget.rs
    reliability.rs
    telemetry.rs
    error.rs
    types.rs
    adapters/
      mod.rs
      openai_compatible.rs
      ollama.rs
      github_copilot.rs
      http_common.rs
      copilot_rpc.rs
```

## 2) Public Facade API

```rust
pub type GatewayEventStream = Pin<Box<dyn Stream<Item = Result<GatewayEvent, GatewayError>> + Send>>;

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
    pub async fn infer_stream(&self, request: BelunaInferenceRequest)
        -> Result<GatewayEventStream, GatewayError>;

    pub async fn infer_once(&self, request: BelunaInferenceRequest)
        -> Result<CanonicalFinalResponse, GatewayError>;
}
```

## 3) Core Internal Contracts

```rust
pub trait BackendAdapter: Send + Sync {
    fn dialect(&self) -> BackendDialect;

    fn static_capabilities(&self) -> BackendCapabilities;

    fn invoke_stream<'a>(
        &'a self,
        ctx: AdapterContext,
        req: CanonicalRequest,
    ) -> BoxFuture<'a, Result<AdapterInvocation, AdapterError>>;
}

pub struct AdapterInvocation {
    pub stream: AdapterEventStream,
    pub backend_identity: BackendIdentity,
    pub cancel: Option<AdapterCancelHandle>,
}

pub type AdapterEventStream = Pin<Box<dyn Stream<Item = Result<BackendRawEvent, AdapterError>> + Send>>;
pub type AdapterCancelHandle = Arc<dyn Fn() + Send + Sync>;

#[async_trait::async_trait]
pub trait CredentialProvider: Send + Sync {
    async fn resolve(&self, reference: &CredentialRef, backend: &BackendProfile)
        -> Result<ResolvedCredential, CredentialError>;
}

pub trait TelemetrySink: Send + Sync {
    fn on_event(&self, event: GatewayTelemetryEvent);
}
```

Note:
- `BackendAdapter` owns transport + dialect mapping responsibility.
- `ResponseNormalizer` turns `BackendRawEvent` into canonical `GatewayEvent`.
- `AdapterInvocation.cancel` is used to abort underlying transport when consumer drops the stream.

## 4) Router and Selection Model

```rust
pub struct BackendRouter {
    default_backend: BackendId,
    backends: HashMap<BackendId, BackendProfile>,
}

impl BackendRouter {
    pub fn select(&self, req: &CanonicalRequest) -> Result<SelectedBackend, GatewayError>;
}

pub struct SelectedBackend {
    pub backend_id: BackendId,
    pub profile: BackendProfile,
    pub resolved_model: String,
}
```

Selection algorithm:

1. Read `request.backend_id` override, else use `default_backend`.
2. Load backend profile by ID.
3. Resolve model by `request.model_override` else backend `default_model`.
4. Return `SelectedBackend`.
5. Router selection is deterministic; no multi-backend fallback in MVP.

## 5) Request Path Execution Wiring

`AIGateway::infer_stream` low-level call sequence:

1. Validate and normalize `BelunaInferenceRequest` -> `CanonicalRequest`.
2. Router selects backend profile.
3. Resolve credentials through `CredentialProvider`.
4. Capability guard validates request flags vs effective backend capabilities.
5. Budget enforcer pre-checks request bounds; acquires concurrency permit.
6. Reliability layer runs invocation loop around adapter.
7. Response normalizer emits canonical stream.
8. Telemetry sink receives lifecycle/usage/error metrics.
9. If consumer drops `GatewayEventStream`, cancel underlying in-flight request and release budget/concurrency resources.

## 6) Ownership Boundaries

- `RequestNormalizer`:
  - Beluna-internal schema to canonical schema only.
  - No backend-specific conditionals.

- `BackendAdapter`:
  - canonical schema to backend wire payload.
  - backend transport/session logic.
  - backend wire payload to backend-raw events.

- `ResponseNormalizer`:
  - backend-raw events to canonical gateway events.
  - ordering and lifecycle guarantees.

- `ReliabilityLayer`:
  - retries/circuit breaker around adapter invocation.
  - no payload mutation.

## 7) Required Crate Additions for This Design

- `reqwest`
- `futures-core`
- `futures-util`
- `tokio-stream`
- `thiserror`
- `tracing`
- `async-trait`
- `uuid` (stable request IDs)

Status: `READY_FOR_L2_REVIEW`
