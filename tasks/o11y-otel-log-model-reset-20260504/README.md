# OTLP Log Model Reset Task Packet

This task packet captures the current architecture discussion about replacing Beluna's private
observability contract algebra with a native OpenTelemetry log and trace model.

The packet is tactical and non-authoritative. Stable outcomes should later be promoted into
Product TDD, Core Unit TDD, Moira Unit TDD, and deployment docs.

## MVT Core

- Objective & Hypothesis: reshape Beluna observability around OTLP resource, scope, trace, span, log attributes, and log body semantics so Core owners emit honest local events and Moira reconstructs views from native telemetry shape.
- Guardrails Touched: Core remains the owner of runtime emission semantics; Moira remains the first-party local consumer for ingestion, storage, query, projection, and Loom inspection.
- Verification: a future implementation must prove the target model with captured OTLP `LogRecord` fixtures, Core emit tests, and Moira projection tests that read from resource/scope/trace/span/body/attributes without private `ContractEvent` flattening.

## Trigger

Current discussion identified several linked smells:

1. `ContractEvent` acts as a god object across AI Gateway, Cortex, Stem, Spine, and future Main observability.
2. `family` carries both owner namespace and event type.
3. `flatten_contract_event()` compensates for weak use of OTLP scope and trace/span context.
4. `payload` is serialized into a string attribute while the OTLP body is only a generic message.
5. Domain-local ids are promoted too broadly as shared log fields.

## Current Draft Direction

1. Scope should identify the Core owner that produced the log:
   - `beluna.core.main`
   - `beluna.core.stem`
   - `beluna.core.cortex`
   - `beluna.core.spine`
   - `beluna.core.ai-gateway`
   - `beluna.core.ai-gateway.chat`
2. One wake plus one tick should map to one trace.
3. Trace and span context should carry causal grouping inside a tick.
4. Pre-first-tick lifecycle activity should use `tick = 0` inside the wake.
5. Log attributes should carry event-schema metadata only when the event type needs it.
6. Log body should carry the first-party rich event payload.
7. `eventName` should be the event type identifier and should require stable attributes/body schema for records with the same `scope.name + eventName`.
8. `request_id`, `thread_id`, and `turn_id` remain AI Gateway transport/chat payload semantics.
9. `sense_id`, `act_id`, `descriptor_id`, `adapter_type`, and similar ids stay local to the owning event schema; each event type chooses body or attributes from its own semantics.

## Packet Files

- [CURRENT-OTLP-CAPTURE.md](./CURRENT-OTLP-CAPTURE.md): Slice 0 capture result and current OTLP shape.
- [TARGET-OTLP-FIXTURES.md](./TARGET-OTLP-FIXTURES.md): Slice 1 candidate target fixtures and review questions.
- [SLICE-2-EMITTER-BOUNDARY.md](./SLICE-2-EMITTER-BOUNDARY.md): Slice 2 Core emitter boundary evidence and proposed implementation shape.
- [RUST-OTLP-LOGS-RESEARCH.md](./RUST-OTLP-LOGS-RESEARCH.md): Rust OTLP Logs research notes and Owner Log Emitter trade-off.
- [AI-GATEWAY-CHAT-BOUNDARY.md](./AI-GATEWAY-CHAT-BOUNDARY.md): current `request_id` implementation evidence and target chat payload boundary.
- [SLICE-3-MOIRA-NATIVE-PROJECTION.md](./SLICE-3-MOIRA-NATIVE-PROJECTION.md): Slice 3 native Moira projection Explore notes, including the thin Moira candidate.
- [SLICE-4-LEGACY-COMPATIBILITY.md](./SLICE-4-LEGACY-COMPATIBILITY.md): Slice 4 legacy compatibility marker and raw drilldown notes.
- [MODEL-CLAIMS.md](./MODEL-CLAIMS.md): current claims and proposed target semantics.
- [OPEN-QUESTIONS.md](./OPEN-QUESTIONS.md): decisions that need human confirmation before durable docs or implementation.
- [MIGRATION-SLICES.md](./MIGRATION-SLICES.md): possible implementation slices after the model is confirmed.
- [capture-receiver](./capture-receiver/): task-local OTLP logs receiver used to capture raw `ExportLogsServiceRequest` fixtures.
- [fixtures/current](./fixtures/current/): captured current OTLP and Moira-normalized shape.
- [fixtures/target](./fixtures/target/): candidate native OTLP shape for review.

## Active Mode

Execute / verified. Slice 2 native Core owner log emission, Slice 3 raw-first Moira native projection, and Slice 4 legacy compatibility markers have been applied in this worktree.

Remaining work starts with `ContractEvent` shrink/removal.
