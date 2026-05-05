# Rust OTLP Logs Research Notes

Research date: 2026-05-04.

This note records the trade-off around Rust OTLP Logs, direct OpenTelemetry Logs API usage, and the proposed Owner Log Emitter.

## Sources Reviewed

Primary sources:

1. OpenTelemetry Logs API specification.
2. OpenTelemetry Logs SDK specification.
3. OpenTelemetry Logs Data Model.
4. OpenTelemetry general Logs concept docs.
5. OpenTelemetry event semantic conventions.
6. Rust `opentelemetry` and `opentelemetry_sdk` docs.rs pages.
7. Rust `opentelemetry-appender-tracing` docs.rs page.

Secondary source:

1. Uptrace Rust OpenTelemetry Logs guide.

## Source Claims

### Logs API

The OpenTelemetry Logs API is stable at the specification level. It centers on:

1. `LoggerProvider`
2. `Logger`
3. `LogRecord`

The API accepts top-level body, attributes, event name, severity, timestamp, and context. The spec says it is provided for log appenders and bridges, and it can also be called directly by instrumentation libraries, instrumented libraries, and applications.

### Rust Logs Status

OpenTelemetry concept docs list Rust Logs implementation status as Beta.

This matters for Beluna because the target shape may be spec-valid while Rust API ergonomics and crate-level stability remain less mature than traces and metrics.

### Body And EventName

The Logs Data Model defines `Body` as `AnyValue`. It explicitly allows structured data composed of arrays and maps. It also says `EventName` identifies the event class/type and should uniquely identify event structure, including attributes and body.

The event semantic conventions page is stricter: standalone events should use attributes for event details and use body for string display messages. That page is in Development status.

Beluna's confirmed first-party rich event model aligns with the Logs Data Model. It diverges from the current event semantic-convention preference around body usage.

### Rust `AnyValue`

Rust `opentelemetry::logs::AnyValue` supports:

1. integers
2. doubles
3. strings
4. booleans
5. bytes
6. arrays
7. maps

The Rust docs also state that `tracing` and `log` only support basic value types for bridge conversion; complex values go through `Debug` formatting unless a custom appender handles them.

### Rust `LogRecord`

Rust `LogRecord` exposes the methods Beluna needs:

1. `set_event_name`
2. `set_body`
3. `add_attribute`
4. `set_trace_context`
5. `set_target`

The `set_target` docs note that the `opentelemetry-appender-tracing` and `opentelemetry-appender-log` crates create one logger whose scope does not accurately reflect the emitting component, so exporters may use target to override instrumentation scope name.

### `opentelemetry-appender-tracing`

The tracing bridge maps:

1. tracing event name to OTLP `EventName`.
2. tracing target to OTLP scope through exporter grouping.
3. tracing level to OTLP severity.
4. tracing fields to OTLP attributes.
5. tracing `message` to OTLP body.

Its docs list `Valuable` support as a limitation. Complex Rust values therefore flow poorly through the tracing event field path.

## Trade-Offs

### Option A: Use `opentelemetry-appender-tracing` Only

Benefits:

1. idiomatic for Rust application logging.
2. already integrated in Core.
3. stable `eventName` and owner scope are achievable through `name:` and `target:`.
4. trace context can be attached when tracing spans and OpenTelemetry context are wired.

Costs:

1. rich body maps become string/debug payloads.
2. complex payload preservation moves into attributes or string body.
3. Beluna's first-party event schema remains constrained by tracing's field model.

Best fit:

1. ordinary human/debug logs.
2. scalar structured logs.
3. transitional logs where rich body fidelity is not required.

### Option B: Direct OpenTelemetry Logs API

Benefits:

1. exact access to top-level OTLP LogRecord fields.
2. `body = AnyValue::Map` supports the confirmed first-party rich event model.
3. owner scope can be created through owner-scoped loggers.
4. deterministic trace/span context can be set explicitly.
5. tests can validate `SdkLogRecord` shape before OTLP export.

Costs:

1. Rust Logs support is Beta.
2. Core must keep a logger provider handle available to the owner-log emitter.
3. Beluna must own `serde_json::Value -> AnyValue` conversion.
4. Beluna must define a small appender-like boundary and keep it narrow.

Best fit:

1. Beluna first-party event records.
2. records that need structured body fidelity.
3. fixture-backed event schemas.

### Option C: Hybrid

Shape:

1. ordinary Rust logs continue through `tracing` and `opentelemetry-appender-tracing`.
2. Beluna first-party owner events go through a small Owner Log Emitter built on `opentelemetry` Logs APIs.
3. both paths share the existing `SdkLoggerProvider` and OTLP exporter pipeline.

Benefits:

1. keeps idiomatic Rust logging for general logs.
2. gives Beluna native OTLP control for event records.
3. keeps the custom surface small: schema constants, value conversion, id derivation, emit call.

Costs:

1. two emission paths exist during migration.
2. Core initialization must make the logger provider available to both paths.
3. tests must guard that ordinary logs and owner events keep distinct roles.

Recommended direction:

Hybrid. Treat Owner Log Emitter as a first-party event appender. General Rust logs stay on `tracing`.

### Option D: Raw OTLP Proto Export Path

Benefits:

1. complete wire-shape control.
2. direct fixture matching.

Costs:

1. bypasses OpenTelemetry SDK processors and exporter lifecycle.
2. duplicates batching/export concerns already handled by the SDK.
3. increases maintenance surface.

Assessment:

Poor fit for Slice 2.

### Option E: Emit JSON String Body And Parse Later

Benefits:

1. keeps `tracing` bridge.
2. simple producer changes.

Costs:

1. preserves the current string-payload smell.
2. pushes structure recovery into Moira or Collector transforms.
3. weakens the target fixture claim that body is native structured payload.

Assessment:

Useful only as a temporary debugging fallback.

## Owner Log Emitter Definition

Owner Log Emitter should mean:

1. a thin Beluna first-party event appender over existing OpenTelemetry Logs APIs.
2. one module boundary under `core/src/observability/owner_log/`.
3. SDK-owned exporter, batching, and shutdown lifecycle.
4. general `tracing` logs remain on the tracing bridge.
5. owner-specific event-schema code lives at callsites or under `owner_log/schema.rs`.

It owns:

1. owner scope constants.
2. event name constants.
3. rich body conversion to `AnyValue::Map`.
4. compact attribute conversion.
5. SHA-256 domain-separated trace/span id derivation.
6. emit into the existing logger provider.

Callsites own:

1. event meaning.
2. body schema.
3. compact occurrence attributes.
4. span key.

## Confirmed ID Direction

Use SHA-256 domain-separated derivation:

1. `trace_id = first_16_bytes(sha256("beluna.core.trace" + run_id + tick))`
2. `span_id = first_8_bytes(sha256("beluna.core.span" + run_id + tick + scope + span_key))`

Rationale:

1. OTLP TraceId is 16 bytes and SpanId is 8 bytes.
2. the repository already depends on `sha2`.
3. domain separation keeps trace and span derivations distinct.

## Slice 2 Guidance

Before full implementation, do one narrow proof:

1. expose or pass a cloned `SdkLoggerProvider` into `owner_log`.
2. emit one `OwnerLogEvent` into an in-memory log exporter test.
3. assert `scope.name`, `eventName`, structured `body`, attributes, `trace_id`, and `span_id`.
4. then wire one runtime lifecycle event and capture it with the task-local OTLP receiver.

This proof should determine the final module shape before migrating the other six target events.
