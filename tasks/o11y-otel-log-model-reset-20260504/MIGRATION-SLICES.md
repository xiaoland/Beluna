# Migration Slices

These slices assume the model claims are confirmed first.

## Slice 0: Capture Current OTLP Reality

Status: completed in [CURRENT-OTLP-CAPTURE.md](./CURRENT-OTLP-CAPTURE.md).

Goal: produce a direct OTLP fixture that shows the current `ResourceLogs -> ScopeLogs -> LogRecord` shape.

Deliverables:

1. one captured contract log record from current Core.
2. one Moira normalize fixture showing current resource/scope/body/attributes.
3. notes on where `target`, `payload`, `family`, `run_id`, and `tick` land.

Verification:

1. fixture proves current `target` maps to `scope.name`.
2. fixture proves current `payload` is a string attribute.
3. fixture proves current `body` is the message string.

## Slice 1: Target OTLP Fixture Spec

Status: completed in [TARGET-OTLP-FIXTURES.md](./TARGET-OTLP-FIXTURES.md).

Goal: define small target fixtures before changing emit code.

Fixtures:

1. main lifecycle log.
2. stem tick grant.
3. cortex concrete-organ interval.
4. ai-gateway transport request.
5. ai-gateway chat turn.
6. spine act delivery.

Verification:

1. every fixture has resource identity.
2. every fixture has owner scope.
3. tick-scoped fixtures share a trace id.
4. records carry native trace and span ids, with log-level parentage limitation documented.
5. event body/attributes follow the chosen body boundary.

Delivered artifacts:

1. [fixtures/target/otlp-target-batch-001.json](./fixtures/target/otlp-target-batch-001.json) contains the candidate native OTLP batch.
2. [fixtures/target/target-record-summary.json](./fixtures/target/target-record-summary.json) contains a compact review summary.
3. [TARGET-OTLP-FIXTURES.md](./TARGET-OTLP-FIXTURES.md) records embedded decisions and questions.

Confirmed Slice 1 decisions:

1. `eventName` is the stable event type marker under `scope.name`.
2. `body` carries the rich structured event payload; `body.kvlistValue` is the OTLP protobuf JSON encoding for a structured body map.
3. pre-first-tick lifecycle activity uses `tick = 0` inside the wake.
4. Moira trace ingestion and owner-local indexing are deferred.
5. AI Gateway Chat uses its own owner scope, `beluna.core.ai-gateway.chat`.

## Slice 2: Core Emitter Boundary

Status: completed for the first native owner-log surface.

Goal: introduce a small shared emitter that accepts owner-provided log records.

Likely changes:

1. create owner-local event modules for main, stem, cortex, spine, ai-gateway, and ai-gateway chat.
2. create shared OTLP/tracing emit helpers for resource, scope, trace/span, severity, timestamp.
3. route existing emit wrappers through owner-local records.

Verification:

1. owner-log unit tests assert structured body, owner scope, event name, compact attributes, and native trace/span ids.
2. full Core test suite passes with native owner-log emission alongside legacy `ContractEvent` emission.

## Slice 3: Moira Native Projection

Status: completed for the raw-first native Moira path in [SLICE-3-MOIRA-NATIVE-PROJECTION.md](./SLICE-3-MOIRA-NATIVE-PROJECTION.md).

Goal: make Lachesis read from OTLP-native fields.

Likely changes:

1. normalize `scope.name` as owner source.
2. persist `eventName`, `trace_id`, `span_id`, and parent span information when present.
3. derive wake/tick read models from trace/span plus payload fields.
4. keep raw resource/scope/body/attributes accessible in Loom.

Verification:

1. Moira store test proves native `tick.granted` anchors create run/tick projections and selected tick detail by trace id.
2. frontend raw-event normalization test proves native records keep scope/event/trace/span identity.
3. full Core and Moira checks pass.

## Slice 4: Legacy Compatibility Projection

Status: completed in [SLICE-4-LEGACY-COMPATIBILITY.md](./SLICE-4-LEGACY-COMPATIBILITY.md).

Goal: keep old local logs inspectable while future logs use the native model.

Likely changes:

1. mark legacy records that carry `family` and serialized `payload`.
2. keep read-only compatibility normalization for historical packets.
3. surface legacy status in raw drilldown.

Verification:

1. legacy records are tagged as `legacy_contract` and keep their parsed rich payload.
2. native owner records are tagged as `native_owner` and keep OTLP identity.
3. raw drilldown surfaces the compatibility marker.

## Slice 5: Remove ContractEvent God Object

Status: completed in [SLICE-5-CONTRACT-EVENT-REMOVAL.md](./SLICE-5-CONTRACT-EVENT-REMOVAL.md).

Goal: retire the central all-family enum after owner emitters and Moira native projection are stable.

Likely changes:

1. delete or shrink central `ContractEvent`.
2. move schemas to owner modules.
3. remove `flatten_contract_event()` from the target path.
4. keep fixture validation near owner event modules.

Verification:

1. `ContractEvent`, `emit_contract_event()`, and `flatten_contract_event()` are absent from `core/src`.
2. runtime callsites use observability classification enums re-exported from `observability::runtime`.
3. Core tests pass with native owner logs as the runtime first-party path.

## Slice 6: Durable Documentation Promotion

Status: completed for the Slice 2 native Core log model. Moira projection docs will need another pass after Slice 3 implementation.

Goal: promote verified decisions into authoritative docs.

Expected updates:

1. Product TDD observability contract.
2. Core Unit TDD observability.
3. Moira Unit TDD design/interfaces/data-and-state/operations/verification.
4. Deployment observability signal configuration.

Verification:

1. docs describe the implemented shape.
2. fixtures and tests enforce the promoted claims.
