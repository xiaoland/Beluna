# Model Claims

## 1. Native OTLP Shape Is The Target Surface

Beluna should model first-party observability using OTLP's native log data model:

1. Resource attributes describe the Core process and producer identity.
2. Instrumentation scope identifies the emitting Core owner.
3. Trace and span context describe wake/tick grouping and operation causality.
4. Log attributes carry event-schema metadata when the event type needs metadata outside the rich body.
5. Log body carries the first-party rich event payload.

The private Rust `ContractEvent` algebra can remain as a temporary implementation detail during migration. The architectural source of truth should move to native OTLP shape plus owner-local event schemas.

## 2. Scope Owns Runtime Boundary

The target scope split is owner-based:

| Scope | Owner |
|---|---|
| `beluna.core.main` | boot, config, runtime lifecycle, signal/exporter state |
| `beluna.core.stem` | tick grant, afferent/efferent pathways, proprioception, neural-signal catalog |
| `beluna.core.cortex` | cognition organs, goal forest, tick-local cognition work |
| `beluna.core.spine` | adapter lifecycle, endpoint lifecycle, sense ingress, act routing/delivery |
| `beluna.core.ai-gateway` | AI gateway transport, backend dispatch, capability-level request records |
| `beluna.core.ai-gateway.chat` | chat thread/turn lifecycle and chat rich payload records |

With this split, `family` can be retired after migration.

## 3. Wake + Tick Is The Trace Boundary

One wake plus one tick should be represented as one trace.

Draft model:

1. `trace_id` is derived from or assigned for `(run_id, tick)`.
2. tick grant is the root span or root-adjacent anchor span.
3. Cortex organ work, AI gateway calls, Stem pathway work, and Spine routing/delivery become child or sibling spans where causality exists.
4. pre-first-tick bootstrap activity uses a `tick = 0` trace inside the wake.

`run_id` and `tick` can remain queryable in Moira read models through bootstrap and tick anchor events, with Core relying on trace context as the primary grouping mechanism once migration lands.

## 4. Event Type Identity Uses `eventName`

Beluna uses OTLP `eventName` as the strict event type marker.

Rules:

1. `eventName` is scoped by `scope.name`.
2. records with the same `scope.name + eventName` follow the same attributes/body schema.
3. examples could be `tick.granted`, `primary.started`, `primary.finished`, `cleanup.finished`, `sense.ingressed`, `act.delivered`, `turn.dispatched`, `turn.committed`.

## 5. First-Party Rich Event Model

Beluna's current `payload` string attribute should be replaced with first-class structured data.

Target rule:

1. event-schema metadata can live in log attributes.
2. the rich event payload lives in `body`.
3. a structured body uses OTLP `AnyValue`; in protobuf JSON fixtures this appears as `body.kvlistValue`.
4. readable summary text can be a field inside the structured body when the event needs one.
5. Moira persists raw OTLP data and derives indexes from native fields.

Attribute placement rule:

1. attributes carry small, stable metadata used for the event type's own lookup, grouping, or filtering.
2. body carries rich payload, snapshots, provider/request/response data, message arrays, and large or schema-deep values.
3. wake/tick grouping comes from trace id plus bootstrap/tick anchor events.
4. event result semantics come from `eventName`, severity, and event-specific body/outcome fields.

## 6. Flatten Attrs Are A Migration Smell

`flatten_contract_event()` exists because Core currently serializes the real event body into `payload` and then duplicates selected fields for query.

Target replacement:

1. scope handles owner grouping.
2. trace/span handles wake/tick grouping and operation causality.
3. event attributes/body carry operation and owner payload semantics.
4. Moira indexes are derived projection surfaces.

The phrase "compact correlation fields" should be withdrawn as a design primitive. The useful idea is narrower: promote a field only when it has a concrete cross-event query or correlation role outside scope/trace/span coverage.

## 7. Domain Ids Stay Local By Default

These fields stay local to the owning event schema:

1. `sense_id`
2. `act_id`
3. `descriptor_id`
4. `adapter_type`
5. `endpoint_id`
6. AI Gateway transport/chat identifiers such as `request_id`, `thread_id`, and `turn_id`

Placement in body or attributes depends on the event type. Promotion into a shared attribute convention requires a named Moira surface, a trace/span limitation, and a fixture that proves the query need.

## 8. AI Gateway Chat Is A Separate Owner Scope

AI Gateway transport and AI Gateway Chat have separate event ownership.

Target split:

1. `beluna.core.ai-gateway` owns backend dispatch and transport request records.
2. `beluna.core.ai-gateway.chat` owns chat thread/turn records.
3. chat ids live in chat event body for rich chat events.
4. transport request ids live in transport event body for request payload/result events.
5. span attributes carry span semantics; log attributes stay event-schema local.

Current implementation uses the name `request_id` for multiple related ids. The target schema should retire the Cortex per-turn operation id from the native log model and keep the AI Gateway backend dispatch id as `transport_request_id`.

## 9. Core Owner Modules Should Emit Their Own Events

The target implementation shape replaces one central `ContractEvent` enum with owner-local event schemas.

Likely shape:

1. owner modules define their own event structs or builders.
2. a small shared emitter writes OTLP/tracing records.
3. shared code owns transport mechanics, timestamp policy, resource setup, and trace/span wiring.
4. owner modules own event meaning and body/attribute schema.
