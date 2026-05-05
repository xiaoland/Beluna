# Core Observability

This file defines Core's target local OTLP log model after the observability reset and the logical fixture contract used by Core-side event-shape checks.

Cross-unit reconstruction guarantees belong in `docs/20-product-tdd/observability-contract.md`.
Loom composition and operator-facing investigation flows belong in `docs/30-unit-tdd/moira/*`.

## Local Rules

1. Core's first-party telemetry carrier is native OTLP Logs shape: resource attributes, instrumentation scope, `eventName`, trace/span context, log attributes, and structured log body.
2. `resource` describes the Core process. Current service name is `beluna.core`.
3. `scope.name` identifies the Core owner that emitted the record.
4. `eventName` identifies the event type under one scope. The schema key is `scope.name + eventName`.
5. Records with the same schema key keep stable attributes and body schema.
6. One wake plus one tick maps to one trace. Pre-first-tick activity uses `tick = 0` inside the wake.
7. Trace ids and span ids use SHA-256 domain-separated deterministic derivation for the first implementation.
8. `span_key` is scoped by `scope.name`; span keys avoid repeating owner or scope segments.
9. Log attributes carry small, stable metadata used for the event type's own lookup, grouping, or filtering.
10. Log body carries rich payload, snapshots, provider/request/response data, message arrays, and large or schema-deep values.
11. Wake/tick grouping comes from trace id plus bootstrap and tick anchor events. Core avoids repeating wake/tick as attributes on every first-party event.
12. Event result semantics come from `eventName`, severity, and event-specific body/outcome fields.
13. Domain ids such as `sense_id`, `act_id`, `descriptor_id`, `endpoint_id`, `thread_id`, and `turn_id` stay local to the owning event schema. Body or attributes placement is an event-schema choice.
14. Ordinary Rust diagnostics can continue through `tracing` and `opentelemetry-appender-tracing`. First-party owner events use the Owner Log Emitter over the OpenTelemetry Logs API so structured body and owner scope remain explicit.
15. Core runtime first-party events use native owner-log emission. Historical legacy contract logs remain a Moira ingestion compatibility concern.

## Owner Scopes

| Scope | Owner |
|---|---|
| `beluna.core.main` | boot, config, runtime lifecycle, exporter state |
| `beluna.core.stem` | tick grant, afferent/efferent pathways, proprioception, neural-signal catalog |
| `beluna.core.cortex` | cognition organs, goal forest, tick-local cognition work |
| `beluna.core.ai-gateway` | gateway transport, backend dispatch, capability-level request records |
| `beluna.core.ai-gateway.chat` | chat thread/turn lifecycle and rich chat payloads |
| `beluna.core.spine` | adapter lifecycle, endpoint lifecycle, sense ingress, act routing/delivery |

## Current Event Surface

The first native implementation covers eight first-party owner event schemas.

| Scope | `eventName` | Span key | Attribute keys | Body owns |
|---|---|---|---|---|
| `beluna.core.main` | `runtime.booted` | `boot` | none | run id, bootstrap summary, config path, OTLP signal state |
| `beluna.core.stem` | `tick.granted` | `grant` | none | run id, tick, tick sequence, and grant summary |
| `beluna.core.cortex` | `primary.started` | `primary` | none | primary input payload, route, execution summary |
| `beluna.core.cortex` | `primary.finished` | `primary` | none | primary output/error payload, linked AI transport id, thread/turn ids when present |
| `beluna.core.ai-gateway` | `transport.request.completed` | `request:{transport_request_id}` | `ai.capability`; `ai.backend.id`; `ai.model` | transport request id, attempt/retry metadata, usage, provider payloads, terminal error |
| `beluna.core.ai-gateway.chat` | `turn.dispatched` | `turn:{thread_id}:{turn_id}` | none | chat/thread/turn ids, transport request id, dispatch payload, metadata |
| `beluna.core.ai-gateway.chat` | `turn.committed` | `turn:{thread_id}:{turn_id}` | none | chat/thread/turn ids, transport request id, committed messages, finish reason, usage, backend metadata |
| `beluna.core.spine` | `act.delivered` | `delivery:{act_id}` | `spine.act.id`; `spine.endpoint.id`; `spine.descriptor.id` | act delivery summary, binding kind, acknowledgement/reference data |

## Trace And Span Derivation

1. `trace_id = first_16_bytes(sha256("beluna.core.trace" + run_id + tick))`.
2. `span_id = first_8_bytes(sha256("beluna.core.span" + run_id + tick + scope + span_key))`.
3. `runtime.booted` uses `tick = 0`.
4. `tick.granted` is the canonical tick anchor for live tick traces.
5. `primary.started` and `primary.finished` share the `primary` span key for one tick's primary phase.
6. Per-turn chat detail is owned by `beluna.core.ai-gateway.chat` spans.

## Correlation Requirements

Moira owns chronology grouping and interval rendering.
Core exposes stable native fields and event bodies so Moira can inspect one tick without parsing prose.

1. `traceId` groups all first-party owner events for one wake plus tick.
2. `runtime.booted` anchors `tick = 0` bootstrap records and exposes `run_id` in body.
3. `tick.granted` anchors admitted live ticks and exposes `run_id`, `tick`, and `tick_seq` in body.
4. Paired interval records share the same span id through the same scope and span key.
5. AI Gateway transport records expose `transport_request_id` in body and backend/model/capability attributes.
6. AI Gateway Chat records expose `thread_id`, `turn_id`, and `transport_request_id` in body.
7. Spine act delivery records expose act routing ids as event-specific attributes because the event's primary lookup surface is act delivery.

## Minimum Fixture Set

Core-side verification should cover:

1. deterministic trace/span id derivation.
2. `serde_json::Value` conversion into OTLP `AnyValue::Map`.
3. one in-memory log exporter proof for owner scope, `eventName`, structured body, attributes, `trace_id`, and `span_id`.
4. raw OTLP capture comparison for the eight-event first implementation surface.
5. absence of `ContractEvent` and `flatten_contract_event()` from the runtime emission path.

## Change Discipline

1. Renaming one `eventName`, owner scope, span key, or required body/attribute field requires updating this file and the corresponding Core emit point in the same change.
2. Adding a first-party owner event requires an event schema keyed by `scope.name + eventName`.
3. A field becomes a shared attribute convention only after a concrete Moira lookup, grouping, or filtering need is documented.
4. Raw rich payload preservation remains the default during early development.
