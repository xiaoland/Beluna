# Slice 2 Core Emitter Boundary

Slice 2 introduces the Core-side emitter boundary for the confirmed native OTLP log model.

Status: implemented for the first native owner-log surface.

## Evidence From Current Core

Current owner-facing observability functions already sit in one tactical layer:

1. `core/src/observability/runtime/stem.rs`
2. `core/src/observability/runtime/cortex.rs`
3. `core/src/observability/runtime/ai_gateway.rs`
4. `core/src/observability/runtime/spine.rs`

Those wrappers still construct `ContractEvent` variants and call:

1. `emit_contract_event()`
2. `flatten_contract_event()`
3. `tracing::{info,warn}` with `target = "observability.contract"`

Current output shape:

1. `target` becomes OTLP `scope.name` through the OTLP exporter grouping path.
2. tracing callsite name becomes OTLP `eventName`.
3. tracing `message` becomes string `body`.
4. tracing fields become log attributes.
5. serialized `ContractEvent` becomes string attribute `payload`.

## Relevant Dependency Behavior

The current `opentelemetry-appender-tracing` bridge supports:

1. `tracing::info!(name: "...", target: "...")` mapping to stable OTLP `eventName`.
2. tracing `target` mapping to OTLP log scope through exporter grouping.
3. scalar field mapping into OTLP log attributes.

Structured `body` needs the direct OpenTelemetry logs API. The tracing bridge maps `message` to a string body and all other fields to attributes.

The direct OpenTelemetry logs API supports the target shape:

1. create a logger with an owner instrumentation scope.
2. set `eventName`.
3. set `body` to `AnyValue::Map`.
4. add compact attributes.
5. set explicit `trace_id` and `span_id`, or use active OpenTelemetry context.

In this packet, "direct OpenTelemetry logs API" means Core creates and emits `LogRecord`s through the `opentelemetry` crate's log traits and the existing `SdkLoggerProvider`. The tracing bridge remains useful for ordinary tracing logs.

## Recommended Slice 2 Shape

Introduce an Owner Log Emitter as an OpenTelemetry Logs integration surface.

The emitter should lean on existing crate APIs:

1. `opentelemetry::logs::LoggerProvider`
2. `opentelemetry::logs::Logger`
3. `opentelemetry::logs::LogRecord`
4. `opentelemetry::logs::AnyValue`

The emitter owns shared integration mechanics:

1. owner-scoped logger lookup.
2. `serde_json::Value -> AnyValue` conversion.
3. deterministic trace/span id generation.
4. wake/tick trace derivation.
5. severity and timestamp policy.

Candidate files:

1. `core/src/observability/owner_log/mod.rs`: public module boundary for Core.
2. `core/src/observability/owner_log/emit.rs`: OpenTelemetry logger integration.
3. `core/src/observability/owner_log/ids.rs`: deterministic `TraceId` and `SpanId` helpers.
4. `core/src/observability/owner_log/value.rs`: `serde_json::Value -> AnyValue` conversion.
5. `core/src/observability/owner_log/schema.rs`: owner scope and event-name constants.

Candidate core API:

```rust
pub(crate) struct OwnerLogEvent {
    pub scope: OwnerScope,
    pub event_name: &'static str,
    pub tick: u64,
    pub span_key: String,
    pub severity: OwnerLogSeverity,
    pub attributes: Vec<OwnerLogAttribute>,
    pub body: serde_json::Value,
}

pub(crate) fn emit(event: OwnerLogEvent);
```

Emitter responsibilities:

1. derive trace id for `current_run_id() + tick`.
2. derive span id from `current_run_id() + tick + scope + span_key`.
3. emit via an owner-scoped OpenTelemetry logger.
4. preserve structured body as `AnyValue::Map`.

Callsite responsibilities:

1. choose `scope`.
2. choose stable `eventName`.
3. choose event-schema attributes when the event type needs them.
4. construct rich structured `body`.
5. provide a stable `span_key` for related start/finish records.

## Proposed Trace Id Rule

Decision: confirmed.

Use deterministic ids for the log model:

1. `trace_id = first_16_bytes(sha256("beluna.core.trace" + run_id + tick))`
2. `span_id = first_8_bytes(sha256("beluna.core.span" + run_id + tick + scope + span_key))`

This is a domain-separated derivation from a longer digest into OTLP's required 16-byte TraceId and 8-byte SpanId sizes.

Alternatives:

1. BLAKE3 extendable output can produce exactly 16 or 8 bytes with one additional dependency.
2. MD5 produces 16 bytes, with security-history baggage that makes it a poor default for new telemetry ids.
3. 64-bit hash algorithms produce span-sized output, while trace ids still need a 128-bit companion.

The current repository already depends on `sha2`, so SHA-256 domain-separated derivation is the smallest dependency footprint.

## Span Key Discipline

`span_key` is scoped by `scope.name`, so it should avoid repeating owner or scope segments.

Initial span keys:

| Scope | Event surface | Span key |
|---|---|---|
| `beluna.core.main` | `runtime.booted` | `boot` |
| `beluna.core.stem` | `tick.granted` | `grant` |
| `beluna.core.cortex` | `primary.started`, `primary.finished` | `primary` |
| `beluna.core.ai-gateway` | `transport.request.completed` | `request:{transport_request_id}` |
| `beluna.core.ai-gateway.chat` | `turn.dispatched`, `turn.committed` | `turn:{thread_id}:{turn_id}` |
| `beluna.core.spine` | `act.delivered` | `delivery:{act_id}` |

## Minimal First Implementation Surface

Start with the Slice 1 scope/event surface plus the AI Gateway Chat owner split.

Six owner scopes:

1. `beluna.core.main`
2. `beluna.core.stem`
3. `beluna.core.cortex`
4. `beluna.core.ai-gateway`
5. `beluna.core.ai-gateway.chat`
6. `beluna.core.spine`

Eight event classes:

1. `beluna.core.main` / `runtime.booted`
2. `beluna.core.stem` / `tick.granted`
3. `beluna.core.cortex` / `primary.started`
4. `beluna.core.cortex` / `primary.finished`
5. `beluna.core.ai-gateway` / `transport.request.completed`
6. `beluna.core.ai-gateway.chat` / `turn.dispatched`
7. `beluna.core.ai-gateway.chat` / `turn.committed`
8. `beluna.core.spine` / `act.delivered`

Cortex event names should be concrete-organ lifecycle names. `organ.started` and `organ.finished` were early fixture names and should stay out of the implementation path.

Cortex primary span key should be `primary` for the whole primary phase. Current per-turn Cortex `request_id` values can leave the target schema once AI Gateway Chat owns per-turn spans and payloads.

The remaining old contract wrappers stay on the legacy path temporarily.

## Verification

1. unit-test `serde_json::Value -> AnyValue` conversion.
2. unit-test deterministic trace/span id generation.
3. in-memory log exporter test for one representative owner record.
4. fixture comparison against `fixtures/target/target-record-summary.json`.
5. capture through the task-local OTLP receiver and compare raw OTLP shape.

Current verification status:

1. owner-log unit tests cover items 1 through 3.
2. full Core tests pass with native owner logs and legacy `ContractEvent` logs emitted together.
3. task-local OTLP receiver capture remains useful for a manual raw export check after a real runtime exercise.
