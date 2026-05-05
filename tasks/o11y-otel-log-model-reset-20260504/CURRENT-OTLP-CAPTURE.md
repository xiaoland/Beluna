# Current OTLP Capture

Capture date: 2026-05-04.

## Method

1. Started the task-local OTLP logs receiver in `capture-receiver/` on `127.0.0.1:4317`.
2. Ran current Core with `cargo run -p beluna -- --config beluna.jsonc`.
3. Sent `SIGTERM` after a short run so the OTLP batch processor flushed one export request.
4. Saved the raw OTLP request and a normalized view under `fixtures/current/`.

Core reached Cortex tick 1 and failed Primary because the local shell did not provide `BAILIAN_API_KEY`.
That failure is acceptable for this slice because the capture target is OTLP record shape rather than successful cognition.

## Captured Files

- [otlp-export-batch-001.json](./fixtures/current/otlp-export-batch-001.json): raw serialized `ExportLogsServiceRequest`.
- [otlp-export-batch-001-summary.json](./fixtures/current/otlp-export-batch-001-summary.json): compact summary emitted by the capture receiver.
- [moira-normalized-events.json](./fixtures/current/moira-normalized-events.json): Moira-shaped normalized rows derived from the raw OTLP request.

## Batch Summary

The first captured batch contains 31 log records.

Observed `scope.name` groups:

| Scope | Count |
|---|---:|
| `core` | 1 |
| `cortex` | 12 |
| `logging` | 1 |
| `observability` | 3 |
| `observability.contract` | 11 |
| `opentelemetry` | 1 |
| `spine` | 2 |

No captured record had native OTLP `trace_id` or `span_id`.

## Resource Shape

Current resource attributes include:

1. `service.name = beluna.core`
2. `telemetry.sdk.language = rust`
3. `telemetry.sdk.name = opentelemetry`
4. `telemetry.sdk.version = 0.31.0`

This is a valid minimal producer resource, but it does not yet carry richer Core build, version, process, or host identity.

## Contract Event Shape

A current contract event lands as:

```text
ScopeLogs.scope.name = "observability.contract"

LogRecord.event_name = "event core/src/observability/runtime/emit.rs:57"
LogRecord.body = "contract_event"
LogRecord.trace_id = ""
LogRecord.span_id = ""

LogRecord.attributes:
  subsystem
  family
  run_id
  tick
  tick_present
  organ_id
  thread_id
  turn_id
  request_id
  endpoint_id
  descriptor_id
  act_id
  sense_id
  adapter_id
  adapter_type
  transition_kind
  outcome
  direction
  binding_kind
  change_mode
  state
  kind
  payload = serialized ContractEvent JSON string
```

The `target: "observability.contract"` used in Core's `tracing::info!` call becomes `scope.name`.
It is not present as a `target` log attribute in this capture.

## Moira Normalize Consequence

Current Moira normalization extracts `target` from log attributes, so captured contract rows have:

```text
target = null
scope.name = "observability.contract"
family = "stem.tick" / "spine.adapter" / ...
subsystem = "stem" / "spine" / ...
payload type = string
body = "contract_event"
trace_id_hex = ""
span_id_hex = ""
```

Moira can still reconstruct current views because `family`, `subsystem`, `run_id`, `tick`, and selected flattened ids are duplicated into log attributes.
That is the current compatibility mechanism.

## Confirmed Smells

1. Owner boundary is hidden inside `family` and duplicated `subsystem`, while the actual OTLP scope is one synthetic `observability.contract`.
2. The real structured event body is a string attribute named `payload`.
3. OTLP body carries only the generic string `contract_event`.
4. Native OTLP trace/span context is unused for captured records.
5. Flattened ids are required for current Moira projection because payload is opaque without parsing.
6. `event_name` currently identifies the Rust callsite, not a stable Beluna event type.

## Slice 0 Result

Slice 0 proves the current system uses OTLP transport correctly but still models first-party observability through a private flattened contract layer.
The next slice should define target fixtures before changing Core or Moira code.
