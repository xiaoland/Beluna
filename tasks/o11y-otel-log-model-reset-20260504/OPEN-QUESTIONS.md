# Open Questions

## Confirmed Decisions

### Event Type Marker

Beluna uses OTLP `eventName` as the event type identifier.

1. `scope.name + eventName` is the event schema key.
2. the same schema key requires stable attributes and body schema.
3. current callsite-style event names are legacy implementation detail.

### Body And Attributes Boundary

Beluna adopts a first-party rich event model.

1. attributes carry event-schema metadata when the event type needs it.
2. body carries the rich structured event payload.
3. raw full payload preservation remains a first-party local requirement during early development.
4. `body.kvlistValue` is the OTLP protobuf JSON encoding for a structured body map.

This decision must account for OTLP attribute limits, DuckDB storage, Loom raw drilldown, and provider/chat payload size.

### Bootstrap Semantics

Pre-first-tick activity uses `tick = 0` inside the wake.

Bootstrap anchor event name remains `runtime.booted`.

### Trace Id Generation

One wake plus one tick maps to one trace.

Core uses SHA-256 domain-separated deterministic derivation for Slice 2 fixtures.

### AI Gateway Chat Scope

AI Gateway Chat uses `beluna.core.ai-gateway.chat` as its owner scope.

1. `beluna.core.ai-gateway` owns transport/backend dispatch records.
2. `beluna.core.ai-gateway.chat` owns thread/turn rich event records.
3. chat ids live in body by default.
4. transport request ids live in body by default.
5. attributes stay event-schema local.

### Span Key Discipline

`span_key` is scoped by `scope.name`, so it avoids repeating owner or scope segments.

Initial keys:

1. `boot`
2. `grant`
3. `primary`
4. `request:{transport_request_id}`
5. `turn:{thread_id}:{turn_id}`
6. `delivery:{act_id}`

### Attribute Placement

1. attributes carry small, stable metadata used for the event type's own lookup, grouping, or filtering.
2. body carries rich payload, snapshots, provider/request/response data, message arrays, and large or schema-deep values.
3. wake/tick grouping comes from trace id plus bootstrap/tick anchor events.
4. event result semantics come from `eventName`, severity, and event-specific body/outcome fields.

### Slice 2 First Surface

Owner Log Emitter first implementation covers eight event classes.

### Span Taxonomy First Surface

Slice 2 emits deterministic span ids for:

1. bootstrap.
2. tick grant.
3. Cortex primary phase.
4. AI Gateway backend request.
5. AI Gateway Chat turn.
6. Spine act delivery.

Point events can remain log records attached to the current span.

### AI Gateway Chat Identifier Names

Current implementation evidence:

1. Cortex creates an organ operation id and currently stores it in chat metadata as `request_id`.
2. AI Gateway Chat creates a backend dispatch id via `next_request_id()`.
3. AI Gateway transport forwards the backend dispatch id as the HTTP `x-request-id`.
4. AI Gateway response metadata returns the backend dispatch id to Cortex as `ai_request_id`.
5. Target Cortex primary span can use span key `primary` for the whole primary phase, with per-turn detail owned by AI Gateway Chat spans.

Slice 2 target naming:

1. AI Gateway backend dispatch id is `transport_request_id`.
2. Cortex primary logs can link to the backend dispatch id in body.
3. thread and turn ids stay in AI Gateway Chat body.

### AI Gateway Chat Dispatch Event

`turn.dispatched` is accepted as the AI Gateway Chat dispatch lifecycle event.

## Remaining Questions

### Moira Trace Ingestion

Moira currently ingests OTLP Logs. Trace ingestion is deferred.

Current implication:

1. Logs can carry `trace_id` and `span_id` for grouping.
2. parent/child span topology remains a later trace-ingestion or span-registry design.
3. Slice 2 can proceed with log records attached to trace/span context.

### Moira Indexing

Indexing choices are deferred.

Current implication:

1. target fixtures show candidate event-specific attributes.
2. Moira native projection can preserve raw attributes and body first.
3. specific indexes can be added from observed Loom query needs.

### Moira Compatibility

What legacy ingestion support should Moira keep?

Likely split:

1. new native OTLP projection path for future logs.
2. read-only compatibility projection for legacy `family` / `payload` logs.
3. explicit legacy marker in Loom raw drilldown.
