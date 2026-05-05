# Target OTLP Fixtures

Slice 1 defines target fixtures before changing Core or Moira code.

These fixtures are tactical review artifacts. Stable decisions should later be promoted into durable docs.

## Files

- [otlp-target-batch-001.json](./fixtures/target/otlp-target-batch-001.json): OTLP-shaped target `ExportLogsServiceRequest`.
- [target-record-summary.json](./fixtures/target/target-record-summary.json): compact readable summary of the target records.

## Fixture Scope

The target batch covers:

1. Main runtime lifecycle.
2. Stem tick grant.
3. Cortex concrete-organ interval start and finish.
4. AI Gateway transport request completion.
5. AI Gateway chat turn dispatch and commit.
6. Spine act delivery.

## Decisions Embedded In The Fixture

### `eventName`

The fixture uses `eventName` as the event type marker.

Rule under test:

1. `scope.name + eventName` identifies one event schema.
2. the same event name under another scope may have a different schema.
3. current callsite-style event names are legacy implementation detail.

Cortex event names use the concrete organ name, such as `primary.started`, `primary.finished`, or `cleanup.finished`.
The early fixture names `organ.started` and `organ.finished` were incorrect because they made multiple organ schemas share one event name.

### Scope

The fixture uses owner scopes:

1. `beluna.core.main`
2. `beluna.core.stem`
3. `beluna.core.cortex`
4. `beluna.core.ai-gateway`
5. `beluna.core.ai-gateway.chat`
6. `beluna.core.spine`

Owner identity is carried through `scope.name`; the target fixture omits `family`.

### Trace

The fixture uses:

1. one pre-first-tick trace for `tick = 0`
2. one tick trace for `wake + tick = target-wake-0001 + 1`

Pre-first-tick lifecycle events belong to `tick = 0` inside the wake.
The bootstrap anchor body carries `run_id`. The tick anchor body carries `run_id`, `tick`, and `tick_seq`.

### Span

Log records carry native OTLP `traceId` and `spanId`.

Important limitation: OTLP `LogRecord` carries `traceId` and `spanId`; parent span topology needs another source.
Parentage must come from OTLP traces, a span registry, or an explicit bridge attribute.
This fixture keeps log-level shape focused on native log fields.

### Body And Attributes

This fixture uses:

1. attributes for event-schema metadata when the event type needs it.
2. `body` for the rich structured event payload.
3. `body.kvlistValue` as the OTLP protobuf JSON encoding for a structured body map.

This keeps payload structured and confines the current serialized `payload` string attribute to legacy compatibility.

Attribute placement rule:

1. attributes carry small, stable metadata used for the event type's own lookup, grouping, or filtering.
2. body carries rich payload, snapshots, provider/request/response data, message arrays, and large or schema-deep values.
3. wake/tick grouping comes from trace id plus bootstrap/tick anchor events.
4. event result semantics come from `eventName`, severity, and event-specific body/outcome fields.

## Domain Id Placement

Domain ids are local to the owning event schema.

Examples:

1. `thread_id`
2. `turn_id`
3. `transport_request_id`
4. `act_id`
5. `endpoint_id`
6. `descriptor_id`

Placement in body or attributes depends on the event type. For example, `act.delivered` can carry `act_id`, `endpoint_id`, and `descriptor_id` as event-specific attributes, while adapter lifecycle events do not carry those fields. Promotion into shared attributes requires a concrete cross-event query need. Span attributes carry span semantics.

## AI Gateway Chat Boundary

The fixture assigns `turn.committed` to `beluna.core.ai-gateway.chat`.

Chat turn body carries:

1. `thread_id`
2. `turn_id`
3. `transport_request_id`
4. committed messages
5. response finish/usage/backend metadata

The current implementation's full `dispatch_payload` overlaps with committed messages on `chat.turn.committed`.
The target fixture keeps dispatch/request payload on `turn.dispatched` and committed-state payload on `turn.committed`.

Decision: `turn.dispatched` is accepted as the AI Gateway Chat dispatch event name.

## Resolved For Slice 2

1. Core derives trace and span ids with SHA-256 domain-separated deterministic helpers.
2. Slice 2 span keys cover bootstrap, tick grant, Cortex primary, AI Gateway backend request, AI Gateway Chat turn, and Spine act delivery.
3. Owner Log Emitter preserves owner scope, `eventName`, compact attributes, native trace/span ids, and structured body through the direct OpenTelemetry Logs API.

Deferred:

1. Moira trace ingestion for parent/child span topology.
2. owner-local attribute indexing after Loom query needs are clearer.
