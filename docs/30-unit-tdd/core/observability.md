# Core Observability

This file defines Core's target local OTLP log model after the observability reset and the logical fixture contract used by Core-side event-shape checks.

Cross-unit reconstruction guarantees belong in `docs/20-product-tdd/observability-contract.md`.
Loom composition and operator-facing investigation flows belong in `docs/30-unit-tdd/moira/*`.

## Local Rules

1. Core's first-party telemetry carrier is native OTLP Logs shape: resource attributes, instrumentation scope, `eventName`, trace/span context, log attributes, and structured log body.
2. `resource` describes the Core process. Current service name is `beluna.core`; `service.instance.id` is the current `run_id` when the resource is built for a wake.
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
14. Ordinary Rust diagnostics can continue through `tracing` and `opentelemetry-appender-tracing`. The tracing file layer and tracing-to-OTLP bridge both obey `logging.filter`.
15. First-party owner events use the Owner Log Emitter over the OpenTelemetry Logs API so structured body and owner scope remain explicit; this direct owner-log path is independent of the ordinary tracing filter.
16. Core runtime first-party events use native owner-log emission. Historical legacy contract logs remain a Moira ingestion compatibility concern.
17. Event vocabulary uses `started` and `finished` for lifecycle boundaries. Native first-party events do not use `completed` or `dispatched`.
18. Dynamic Spine endpoint and adapter owner scopes use canonical OTLP-safe suffixes. Body keeps the original endpoint id or adapter name.

## Owner Scopes

| Scope | Owner |
|---|---|
| `beluna.core.main.runtime` | boot, config, runtime lifecycle, exporter state |
| `beluna.core.stem.tick` | tick grant and admitted-tick anchors |
| `beluna.core.stem.afferent-pathway` | inbound sense pathway from Spine into Cortex-facing tick input |
| `beluna.core.stem.proprioception` | runtime self-state and environment observations |
| `beluna.core.stem.descriptor-catalog` | neural-signal descriptor catalog changes and lookups |
| `beluna.core.stem.efferent-pathway` | outbound act pathway from Cortex decisions toward Spine |
| `beluna.core.cortex.primary` | primary cognition phase for a tick |
| `beluna.core.cortex.attention` | attention and focus work within a tick |
| `beluna.core.cortex.cleanup` | cleanup phase after primary cognition |
| `beluna.core.cortex.sense-helper` | sense helper cognition work |
| `beluna.core.cortex.acts-helper` | act helper cognition work |
| `beluna.core.cortex.goal-forest` | goal-forest inspection and mutation records |
| `beluna.core.ai-gateway.chat` | chat turn/thread lifecycle and rich chat payloads |
| `beluna.core.ai-gateway.transport` | gateway transport, backend dispatch, capability-level request records |
| `beluna.core.spine.endpoint.<endpoint-id-segment>` | endpoint lifecycle, sense ingress, and act terminal outcomes for one endpoint |
| `beluna.core.spine.adapter.<adapter-name-segment>` | adapter lifecycle and adapter-local state for one adapter |

## Target Event Surface

The v1 native implementation targets boot/tick anchors, Stem pathway/state/catalog records, Cortex owner boundary records, AI Gateway transport/chat records, and dynamic Spine adapter/endpoint records.

| Scope | `eventName` | Span key | Attribute keys | Body owns |
|---|---|---|---|---|
| `beluna.core.main.runtime` | `booted` | `boot` | none | run id, bootstrap summary, config path, OTLP signal state |
| `beluna.core.stem.tick` | `granted` | `grant` | none | run id, tick, tick sequence, and grant summary |
| `beluna.core.cortex.primary` | `started` | `primary` | none | primary input payload, route, execution summary |
| `beluna.core.cortex.primary` | `finished` | `primary` | none | primary output/error payload, linked AI transport id, thread/turn ids when present |
| `beluna.core.cortex.attention` | `started`; `finished` | `attention` | none | attention input/output/error payloads, route, linked AI transport id, thread/turn ids when present |
| `beluna.core.cortex.cleanup` | `started`; `finished` | `cleanup` | none | cleanup input/output/error payloads, route, linked AI transport id, thread/turn ids when present |
| `beluna.core.cortex.sense-helper` | `started`; `finished` | `sense-helper` | none | sense helper input/output/error payloads, route, linked AI transport id, thread/turn ids when present |
| `beluna.core.cortex.goal-forest` | `started`; `finished` | `goal-forest` | none | goal-forest helper input/output/error payloads, route, linked AI transport id, thread/turn ids when present |
| `beluna.core.cortex.acts-helper` | `started`; `finished` | `acts-helper` | none | acts helper input/output/error payloads, route, linked AI transport id, thread/turn ids when present |
| `beluna.core.ai-gateway.transport` | `request.started`; `request.finished`; `request.failed` | `request:{transport_request_id}` | `ai.capability`; `ai.backend.id`; `ai.model` | transport request id, parent span id, organ id, attempt/retry metadata, provider request/response payloads, usage, terminal error |
| `beluna.core.ai-gateway.transport` | `attempt.failed` | `request:{transport_request_id}` | `ai.capability`; `ai.backend.id`; `ai.model` | attempt number, retry decision, provider error, request summary |
| `beluna.core.ai-gateway.chat` | `turn.started`; `turn.finished`; `turn.failed` | `turn:{thread_id}:{turn_id}` | none | chat/thread/turn ids, parent span id, organ id, transport request id, turn start payload, final messages or terminal error, finish reason, usage, backend metadata |
| `beluna.core.ai-gateway.chat` | `thread.opened`; `thread.derived`; `thread.rewritten`; `thread.snapshot` | `thread:{thread_id}` | none | thread id, source/kept/dropped turn ids, messages, turn summaries, context reason, continuation state |
| `beluna.core.stem.afferent-pathway` | `sense.enqueued`; `sense.deferred`; `sense.released`; `sense.dropped` | `sense:{sense_id}` or `descriptor:{descriptor_id}` | none | sense/endpoint/descriptor ids, tick when known, sense payload, weight, queue state, matched rule ids, reason |
| `beluna.core.stem.afferent-pathway` | `rules.added`; `rules.removed`; `rules.replaced` | `rule:{rule_id}` | none | rule id, revision, rule snapshot, removed flag |
| `beluna.core.stem.proprioception` | `patched`; `dropped` | `state` | none | proprioception entry patch or dropped keys |
| `beluna.core.stem.descriptor-catalog` | `snapshot`; `updated`; `dropped` | `version:{catalog_version}` | none | catalog version, accepted entries/routes, rejected entries/routes, optional catalog snapshot |
| `beluna.core.stem.efferent-pathway` | `act.enqueued`; `act.started`; `act.finished`; `act.failed`; `act.dropped` | `act:{act_id}` | none | act/endpoint/descriptor ids, tick when known, act payload, queue state, continuity decision, terminal outcome, reason/reference |
| `beluna.core.spine.endpoint.<endpoint-id-segment>` | `connected`; `registered`; `disconnected`; `dropped` | `endpoint` | none | endpoint id, canonical endpoint segment, adapter id/name, transition, channel/session, route summary, reason/error |
| `beluna.core.spine.endpoint.<endpoint-id-segment>` | `sense.received` | `sense:{sense_id}` | optional `spine.descriptor.id` | sense/endpoint/descriptor ids, sense payload, reason |
| `beluna.core.spine.endpoint.<endpoint-id-segment>` | `act.started`; `act.finished`; `act.rejected`; `act.lost` | `act:{act_id}` | `spine.act.id`; optional `spine.descriptor.id` | act routing summary, binding kind/channel, outcome, reason/reference, act payload |
| `beluna.core.spine.adapter.<adapter-name-segment>` | `enabled`; `disabled`; `faulted` | `adapter` | none | adapter name/type, canonical adapter segment, lifecycle state, reason/error |

## Trace And Span Derivation

1. `trace_id = first_16_bytes(sha256("beluna.core.trace" + run_id + tick))`.
2. `span_id = first_8_bytes(sha256("beluna.core.span" + run_id + tick + scope + span_key))`.
3. `beluna.core.main.runtime / booted` uses `tick = 0`.
4. `beluna.core.stem.tick / granted` is the canonical tick anchor for live tick traces.
5. Cortex owner boundary pairs share a scope-local organ span key, for example `primary`, `attention`, or `cleanup`.
6. Per-turn chat detail is owned by `beluna.core.ai-gateway.chat` spans.
7. Dynamic Spine endpoint and adapter scopes include canonical suffixes in `scope.name`; the span key avoids repeating the endpoint or adapter owner segment.

## Correlation Requirements

Moira owns chronology grouping and interval rendering.
Core exposes stable native fields and event bodies so Moira can inspect one tick without parsing prose.

1. `traceId` groups all first-party owner events for one wake plus tick.
2. `beluna.core.main.runtime / booted` anchors `tick = 0` bootstrap records and exposes `run_id` in body.
3. `beluna.core.stem.tick / granted` anchors admitted live ticks and exposes `run_id`, `tick`, and `tick_seq` in body.
4. Paired interval records share the same span id through the same scope and span key.
5. AI Gateway transport records expose `transport_request_id` in body and backend/model/capability attributes.
6. AI Gateway Chat records expose `thread_id`, `turn_id`, and `transport_request_id` in body.
7. Spine endpoint and adapter records expose original endpoint id or adapter name in body and use canonical dynamic owner scope suffixes for lane identity.
8. Spine act records may expose `spine.act.id` as an event-specific attribute because act lookup is useful within endpoint lanes.

## Minimum Fixture Set

Core-side verification should cover:

1. deterministic trace/span id derivation.
2. `serde_json::Value` conversion into OTLP `AnyValue::Map`.
3. one in-memory log exporter proof for owner scope, `eventName`, structured body, attributes, `trace_id`, and `span_id`.
4. raw OTLP capture comparison for the target first-party owner event surface.
5. absence of `ContractEvent` and `flatten_contract_event()` from the runtime emission path.
6. dynamic Spine owner scope canonicalization with original endpoint id or adapter name preserved in body.
7. native vocabulary guardrails for `started`/`finished` and absence of `completed`/`dispatched` in first-party event names.

## Change Discipline

1. Renaming one `eventName`, owner scope, span key, or required body/attribute field requires updating this file and the corresponding Core emit point in the same change.
2. Adding a first-party owner event requires an event schema keyed by `scope.name + eventName`.
3. A field becomes a shared attribute convention only after a concrete Moira lookup, grouping, or filtering need is documented.
4. Raw rich payload preservation remains the default during early development.
