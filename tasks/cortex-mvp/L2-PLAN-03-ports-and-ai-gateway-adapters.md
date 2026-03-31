# L2-03 - Ports And AI Gateway Adapters
- Task Name: `cortex-mvp`
- Stage: `L2` detailed file
- Date: `2026-02-11`
- Status: `DRAFT_FOR_APPROVAL`

## 1) Cognition Port Interfaces
All model calls are abstracted through traits so runtime uses real `ai_gateway`, tests use mocks.

```rust
#[async_trait::async_trait]
pub trait PrimaryReasonerPort: Send + Sync {
    async fn infer_ir(&self, req: PrimaryReasonerRequest) -> Result<ProseIr, CortexError>;
}

#[async_trait::async_trait]
pub trait AttemptExtractorPort: Send + Sync {
    async fn extract(&self, req: AttemptExtractorRequest) -> Result<Vec<AttemptDraft>, CortexError>;
}

#[async_trait::async_trait]
pub trait PayloadFillerPort: Send + Sync {
    async fn fill(&self, req: PayloadFillerRequest) -> Result<Vec<AttemptDraft>, CortexError>;
}
```

Deterministic clamp remains local Rust logic:
```rust
pub trait AttemptClampPort: Send + Sync {
    fn clamp(
        &self,
        req: AttemptClampRequest,
    ) -> Result<AttemptClampResult, CortexError>;
}
```

## 2) Request/Response Port Contracts
### Primary reasoner
```rust
pub struct PrimaryReasonerRequest {
    pub reaction_id: ReactionId,
    pub context: IntentContext,
    pub sense_window: Vec<SenseDelta>,
    pub env_snapshots: Vec<EndpointSnapshot>,
    pub admission_feedback: Vec<AdmissionOutcomeSignal>,
    pub limits: ReactionLimits,
}
```

Primary output:
1. prose IR text only.
2. no direct structured attempt output.

### Extractor
```rust
pub struct AttemptExtractorRequest {
    pub reaction_id: ReactionId,
    pub prose_ir: ProseIr,
    pub capability_catalog: CapabilityCatalog,
    pub sense_window: Vec<SenseDelta>,
    pub limits: ReactionLimits,
}
```

Extractor output:
1. draft attempts with `intent_span`, `based_on`, `attention_tags`.
2. route candidate must resolve to one `affordance_key`.

### Filler/repair
```rust
pub struct PayloadFillerRequest {
    pub reaction_id: ReactionId,
    pub drafts: Vec<AttemptDraft>,
    pub capability_catalog: CapabilityCatalog,
    pub clamp_violations: Vec<ClampViolation>,
    pub limits: ReactionLimits,
}
```

Filler output:
1. modified drafts only.
2. must not create new draft count beyond input count.

## 3) AI Gateway Runtime Adapter Design
Use `ai_gateway::AIGateway::infer_once` with `BelunaInferenceRequest`.

### Primary adapter behavior
1. call with `OutputMode::Text`.
2. provide prose IR instruction template only.
3. set `limits.max_output_tokens = max_primary_output_tokens`.
4. include `cost_attribution_id` derived from reaction id and stage key.

### Extractor adapter behavior
1. call with tools enabled and required choice for `compile_attempts`.
2. tool JSON schema defines `AttemptDraft[]`.
3. parse tool call arguments as extractor output.
4. set subcall token limits.

### Filler adapter behavior
1. call with tools enabled and required choice for `repair_attempts`.
2. schema accepts prior drafts + clamp violations and returns same-cardinality draft list.
3. parse tool output and return.

## 4) AI Gateway Capability Requirements
Runtime preconditions for selected backend profiles:
1. primary backend:
- `streaming` optional (MVP uses non-stream inference call).
- tool calling not required.
2. sub-LLM backend:
- `tool_calls` required.
- `json_mode` recommended.
3. on capability mismatch:
- return bounded reactor error and produce noop output for that cycle.

## 5) Mock Strategy For Tests
Provide deterministic mock ports:
1. `MockPrimaryReasoner`
- returns fixed prose IR by reaction id.
2. `MockExtractor`
- returns fixed draft arrays.
3. `MockFiller`
- deterministic repair rewrite.
4. `MockClamp` optional for isolated reactor tests; default tests should use real clamp.

Goals:
1. zero external network dependency in tests.
2. fully deterministic cycle behavior.
3. explicit call counters to assert one-primary/N-subcall/one-repair limits.

## 6) Telemetry Without Business-Payload Pollution
Add optional observer port:

```rust
pub trait CortexTelemetryPort: Send + Sync {
    fn on_event(&self, event: CortexTelemetryEvent);
}
```

Direction:
1. call-count/budget/latency/clamp-drop metrics go here.
2. `ReactionResult` remains business-only.

## 7) Error Surface
Add specific error kinds:
1. `InvalidReactionInput`
2. `PrimaryInferenceFailed`
3. `ExtractorInferenceFailed`
4. `FillerInferenceFailed`
5. `ClampRejectedAll`
6. `BudgetExceeded`
7. `CycleTimeout`

Reactor policy:
1. these errors are not propagated as stream break by default.
2. cycle converts terminal cognition errors into noop result.
3. reactor loop continues for next inbox event.

## 8) L2-03 Exit Criteria
This file is complete when:
1. every cognition stage has a strict interface,
2. runtime ai_gateway usage is explicit,
3. test-mock replacement is guaranteed by trait boundaries.
