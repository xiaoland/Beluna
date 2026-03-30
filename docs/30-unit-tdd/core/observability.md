# Core Observability

This file defines Core's target local OTLP log model after the observability reset and the logical fixture contract used by Core-side event-shape checks.

Cross-unit reconstruction guarantees belong in `docs/20-product-tdd/observability-contract.md`.
Loom composition and operator-facing investigation flows belong in `docs/30-unit-tdd/moira/*`.

Current code may still emit the older coarse Stage 2 family catalog in places until the synchronized refactor lands.
This file is the target lattice for that refactor.
AI observability is split into one capability-neutral gateway transport family plus chat-owned capability families under the gateway namespace.

## Local Rules

1. Core's native telemetry carrier is OpenTelemetry semantic context plus structured log body.
2. Every canonical emitted record is attributed to one `tick`. Bootstrap or pre-first-grant activity uses `tick = 0`.
3. `tick` is the operator-facing trace anchor. Tick-scoped runtime work should remain in one stable trace context per admitted tick and stay unique within one wake.
4. Literal OpenTelemetry `span_id` values are opaque operation-instance identifiers. They are not human-friendly domain labels.
5. Domain identifiers such as `organ_id`, `thread_id`, `turn_id`, `endpoint_id`, and similar fields remain structured-body fields by default. Promote them into broader telemetry context only when cross-record pairing or cross-module correlation requires it.
6. Resource- or process-level identity belongs in OpenTelemetry resource attributes, not duplicated in every event body field set.
7. Core should prefer domain-honest families over coarse catch-all families. Do not merge asymmetrical state machines only because one struct could make many fields optional.
8. `cortex.tick` is removed from the target model. The canonical tick anchor is `stem.tick`.
9. `stem.signal` and `stem.dispatch` are removed from the target model. Their semantics split into `stem.afferent` and `stem.efferent`.
10. Stable Cortex execution families are one family per stable organ rather than one coarse `cortex.organ` family.
11. `spine.endpoint` owns endpoint attachment and lifecycle semantics such as new, register, and drop. `spine.sense` means Spine received one sense from a body endpoint.
12. The outbound Spine act family is `spine.act`.
13. `ai-gateway.request` is the stable capability-neutral gateway transport family. It must not own thread, turn, message, or tool semantics.
14. Goal-forest observability remains grounded in the mutation semantics the runtime actually owns today. The stable target surface is snapshot plus mutation, not speculative botanical diff verbs.
15. During Beluna's early development phase, Core preserves full request, response, signal, topology, and other diagnostic payloads in raw OTLP records by default.
16. Golden fixture bundles live under `core/tests/fixtures/observability/` and should be refreshed only after the family catalog stabilizes enough to justify the maintenance cost.
17. Chat capability observability lives under `ai-gateway.chat.*`. Chat turn and thread semantics must not be hidden inside `ai-gateway.request`.

## Field Notation

In the tables below:

1. ``?`` marks an optional field.
2. Listed fields are structured event-body fields unless stated otherwise.
3. OpenTelemetry context and resource attributes are still required where the Product TDD contract says they matter, even when they are not repeated in every family row below.

## Target Family Lattice

### Stable Non-AI Families

| Logical family | Runtime owner / emit point | Required logical fields | Supports |
|---|---|---|---|
| `stem.tick` | Stem tick grant loop | `run_id`; `tick`; `span_id`; `status`; `tick_seq` | canonical tick anchor and grant narrative |
| `stem.afferent` | afferent enqueue / defer / release / drop boundaries | `run_id`; `tick`; `span_id`; `parent_span_id?`; `kind`; `descriptor_id`; `endpoint_id?`; `sense_id?`; `sense_payload?`; `weight?`; `queue_state?`; `matched_rule_ids?`; `reason?` | afferent pathway inspection |
| `stem.efferent` | efferent enqueue / route / outcome boundaries owned by Stem | `run_id`; `tick`; `span_id`; `parent_span_id?`; `kind`; `act_id`; `descriptor_id?`; `endpoint_id?`; `act_payload?`; `queue_state?`; `continuity_decision?`; `terminal_outcome?`; `reason?` | efferent pathway inspection |
| `stem.proprioception` | physical-state patch / drop mutation boundary | `run_id`; `tick`; `span_id`; `kind`; `entries_or_keys` | proprioception history |
| `stem.ns-catalog` | neural-signal catalog snapshot / update / drop commit | `run_id`; `tick`; `span_id`; `catalog_version`; `change_mode`; `accepted_entries_or_routes`; `rejected_entries_or_routes`; `catalog_snapshot?` | neural-signal catalog history |
| `stem.afferent.rule` | afferent scheduler rule add / remove boundary when explicit in runtime | `run_id`; `tick`; `span_id`; `kind`; `revision`; `rule_id`; `rule?`; `removed?` | afferent-rule lifecycle |
| `cortex.goal-forest` | goal-forest snapshot and mutation path | `run_id`; `tick`; `span_id`; `parent_span_id?`; `kind`; `snapshot?`; `mutation_request?`; `mutation_result?`; `persisted_revision?`; `reset_context_applied?`; `selected_turn_ids?` | goal-forest state and mutation narrative |
| `spine.adapter` | adapter startup / lifecycle / fault handling | `run_id`; `tick`; `span_id`; `adapter_type`; `adapter_id`; `kind`; `reason_or_error?` | adapter topology reconstruction |
| `spine.endpoint` | endpoint connect / drop / lifecycle changes | `run_id`; `tick`; `span_id`; `endpoint_id`; `adapter_id?`; `kind`; `channel_or_session?`; `route_summary?`; `reason_or_error?` | endpoint topology reconstruction |
| `spine.sense` | body-endpoint ingress into Core | `run_id`; `tick`; `span_id`; `parent_span_id?`; `endpoint_id`; `descriptor_id?`; `sense_id`; `kind`; `sense_payload`; `reason?` | sense ingress and endpoint-origin reconstruction |
| `spine.act` | act routing / binding / delivery path owned by Spine | `run_id`; `tick`; `span_id`; `parent_span_id?`; `act_id`; `endpoint_id?`; `descriptor_id?`; `kind`; `binding_kind?`; `channel_id?`; `act_payload?`; `outcome?`; `reason_or_reference?` | act routing and delivery reconstruction |

### Stable Per-Organ Cortex Families

All stable per-organ Cortex execution families share one common event-body shape:

`run_id`; `tick`; `request_id`; `span_id`; `parent_span_id?`; `phase`; `status`; `route_or_backend?`; `input_payload?`; `output_payload?`; `error?`; `ai_request_id?`; `thread_id?`; `turn_id?`

| Logical family | Runtime owner / emit point | Supports |
|---|---|---|
| `cortex.primary` | primary cognition turn boundaries | primary-organ execution and interval pairing |
| `cortex.sense-helper` | sense helper execution boundaries | helper-organ execution and interval pairing |
| `cortex.goal-forest-helper` | goal-forest helper execution boundaries | helper-organ execution and interval pairing |
| `cortex.acts-helper` | acts helper execution boundaries | helper-organ execution and interval pairing |

### Stable AI Families

| Logical family | Runtime owner / emit point | Required logical fields | Supports |
|---|---|---|---|
| `ai-gateway.request` | gateway transport request lifecycle around one backend call | `run_id`; `tick`; `request_id`; `span_id`; `parent_span_id?`; `organ_id?`; `capability`; `backend_id`; `model`; `kind`; `attempt?`; `retryable?`; `provider_request?`; `provider_response?`; `usage?`; `error?` | capability-neutral provider/backend call narrative |
| `ai-gateway.chat.turn` | chat turn commit / failure boundary | `run_id`; `tick`; `thread_id`; `turn_id`; `span_id`; `parent_span_id?`; `organ_id?`; `request_id?`; `status`; `dispatch_payload`; `messages_when_committed?`; `metadata`; `finish_reason?`; `usage?`; `backend_metadata?`; `error?` | turn lifecycle, message/tool inspection, and linked transport drilldown |
| `ai-gateway.chat.thread` | chat thread snapshot on open / clone / turn commit | `run_id`; `tick`; `thread_id`; `span_id`; `parent_span_id?`; `organ_id?`; `request_id?`; `kind`; `messages`; `turn_summaries?`; `source_turn_ids?` | authoritative thread snapshot and thread-level reconstruction |

## Correlation Requirements

Moira owns chronology grouping and interval rendering.
Core's responsibility is to expose enough stable correlation that Moira can investigate one tick without parsing prose or inventing missing semantics.

1. `stem.tick` provides the canonical tick anchor.
2. Per-organ Cortex start and end records must share one stable operation key such as `request_id` so Moira can pair them into one interval.
3. Nested work inside one tick must carry stable `span_id` and `parent_span_id` where one operation is causally under another.
4. `ai-gateway.request` records must correlate to the invoking Cortex interval through `parent_span_id` and the request identity exposed by Cortex as `ai_request_id` when that bridge is available.
5. `ai-gateway.chat.turn` and `ai-gateway.chat.thread` records may additionally expose `thread_id`, `turn_id`, and optional linked `request_id` for targeted drilldown, but those identifiers do not need to become first-class chronology keys by default.
6. Domain identifiers such as `endpoint_id` remain available for inspection and targeted correlation without being promoted automatically into first-class chronology keys.
7. If one future Loom surface requires a domain identifier to become a first-class grouping key, that requirement should be added from Moira Unit TDD rather than guessed here.

Implementation notes:

1. Literal OpenTelemetry `span_id` remains an opaque instance id even when a family also exposes domain identifiers for inspection.
2. This section defines the minimum correlation Core must support. It does not freeze Moira's final lane decomposition.
3. If one future operator context needs a richer grouping key, that is a Moira concern layered on top of this minimum contract.

## Minimum Fixture Set

When the target family catalog is refreshed into fixtures, the minimum coverage should include:

1. `ai-gateway.request` and `ai-gateway.chat.*`
- one gateway request success with provider request / response payloads
- one gateway request retry or failure with attempt metadata
- one committed chat turn with user, assistant, tool-call, and tool-result messages
- one failed chat turn with input payload and terminal error
- one thread snapshot after a completed turn
- one thread snapshot after thread clone or reset-style rewrite

2. `cortex.*`
- one `cortex.primary` start / end pair with full input and output
- one helper-organ start / end pair
- one per-organ error termination
- one goal-forest snapshot
- one goal-forest mutation / persist event with reset-context details when applicable

3. `stem.*`
- one `stem.tick`
- one `stem.afferent` enqueue or accept event with full sense payload
- one `stem.afferent` defer or release path
- one `stem.efferent` queue admission
- one `stem.efferent` terminal outcome with full act payload
- one `stem.proprioception` patch and one drop
- one `stem.ns-catalog` update or drop
- one `stem.afferent.rule` add or remove when that lifecycle remains explicit

4. `spine.*`
- one adapter lifecycle change
- one endpoint lifecycle change
- one `spine.sense` ingress event
- one `spine.act` bind / route event
- one `spine.act` terminal outcome

## Change Discipline

1. Renaming a logical family or changing its required logical field set requires updating this file and the corresponding Core emit points in the same change.
2. When a family is marked provisional in this file, later naming churn is acceptable only inside the explicitly provisional scope.
3. If a change affects only Core-local debug decoration and not the canonical reconstruction fields, keep it in Core Unit TDD unless it changes a cross-unit guarantee.
4. If a new field or family is required so Moira can reconstruct a new domain guaranteed by Product TDD, update Product TDD first and then update this file.
5. Do not collapse separate pathway or lifecycle families back into one coarse catch-all family unless the runtime genuinely owns one uniform state machine.
