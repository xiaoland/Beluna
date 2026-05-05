# Slice 3 Moira Native Projection Explore

This note captures the first Slice 3 Explore pass after Core started emitting native owner logs.

## MVT Core

- Objective & Hypothesis: make Moira ingest native OTLP log fields directly while keeping Lachesis thin: raw preservation, wake/tick indexes, selected tick event browsing, and source-grounded drilldown first.
- Guardrails Touched: Core owns emission semantics; Moira owns local ingestion, DuckDB storage, query, projection, and Loom inspection. Durable Moira reconstruction guarantees need explicit confirmation before they shrink.
- Verification: native OTLP fixtures should populate `raw_events`, `runs`, `ticks`, selected tick detail, basic event lanes/cards, and raw drilldown from `scope.name`, `eventName`, `traceId`, `spanId`, attributes, and structured body.

## Current Backend Shape

Lachesis backend already has the right outer pipeline:

1. OTLP gRPC logs receiver: `moira/src-tauri/src/lachesis/receiver.rs`.
2. OTLP batch normalization: `moira/src-tauri/src/lachesis/normalize.rs`.
3. DuckDB storage: `moira/src-tauri/src/lachesis/store/mod.rs`.
4. derived `runs` and `ticks`: `moira/src-tauri/src/lachesis/store/write.rs`.
5. query commands: `list_runs`, `list_ticks`, `tick_detail`.

Current normalization is still legacy-centered:

1. extracts `target`, `family`, `subsystem`, `run_id`, and `tick` from attributes.
2. stores body, attributes, resource, and scope as JSON strings.
3. drops native `event_name`, `trace_id`, `span_id`, and `flags`.
4. builds `runs` and `ticks` from `run_id` and `tick`.
5. marks `cortex_handled` through legacy family names.

## Current Frontend Shape

The frontend layering is already usable:

1. `bridge` owns backend-shaped contracts.
2. `query/lachesis` owns wake/tick selection and live refresh.
3. `projection/lachesis` owns normalization, chronology, labels, narratives, and raw JSON sections.
4. `presentation/lachesis` consumes projected models.

Current projection is reconstruction-heavy:

1. `families.ts` classifies legacy family names.
2. `chronology.ts` pairs Cortex start/end records by legacy family, phase, and request id.
3. AI transport/chat records are related back to Cortex intervals through request/thread/turn ids.
4. Stem and Spine tabs are built from family-specific narrative buckets.
5. Raw inspector already works as the strongest source-grounded surface.

## Thin Moira Candidate

Thin Moira should treat native logs as event records first and derived interpretations second.

Recommended first native surface:

1. Store every raw native log with:
   - `scope_name`
   - `event_name`
   - `trace_id_hex`
   - `span_id_hex`
   - `severity_text`
   - `severity_number`
   - `body_json`
   - `attributes_json`
   - `resource_json`
   - `scope_json`
   - legacy `target`, `family`, `subsystem`, `run_id`, `tick` when present
2. Derive a wake key from native anchors:
   - `runtime.booted` body/config plus resource identity can identify bootstrap records.
   - native Core currently derives trace id from `run_id + tick`; Moira receives only the trace id.
   - a stable human wake id still needs either preserved Core `run_id` in resource/body or a Moira-local wake session id.
3. Derive tick rows from `beluna.core.stem / tick.granted`:
   - `tick` comes from anchor body when present, with `tick = 0` reserved for bootstrap.
   - `trace_id_hex` is the machine grouping key.
4. Selected tick detail returns raw native events by `trace_id_hex`.
5. Frontend native projection builds:
   - a scope/event lane timeline by observed time.
   - raw event cards with scope, eventName, trace/span, severity, attributes, body, resource.
   - optional simple summaries from body `summary`.

## Deferred Reconstruction

The following can stay as optional projections after the native read path is live:

1. pairing `primary.started` and `primary.finished` into one interval.
2. linking AI transport/chat records back to Cortex primary.
3. rebuilding Stem afferent/efferent and Spine sense/act pathway sections from native events.
4. goal-forest comparison.
5. indexing event-specific ids such as `transport_request_id`, `thread_id`, `turn_id`, `act_id`, `endpoint_id`, and `descriptor_id`.

## Contract Tension

Current durable docs still promise broad reconstruction:

1. selected tick workspace.
2. Cortex timeline mode.
3. nested AI transport and chat investigation.
4. Cortex/Stem/Spine sectional inspection.
5. raw event inspector.

Thin Moira can keep the workspace and raw inspector quickly. It changes the timing and strength of Cortex/AI/Stem/Spine reconstruction guarantees.

This was confirmed for the first Slice 3 implementation.

## Core Anchor Gap Found During Explore

Core native owner logs currently derive `traceId` from `run_id + tick`, but Moira receives only the derived id.

Current Slice 2 event bodies:

1. `runtime.booted` carries config path and OTLP signal state.
2. `tick.granted` carries `tick_seq`.

For native Moira projection, the anchors should expose enough human-facing identity to build read models:

1. `runtime.booted` body includes `run_id`.
2. `tick.granted` body includes `run_id`, `tick`, and `tick_seq`.

This keeps wake/tick identity inside anchor body rather than repeated on every record.

## Backend Changes

1. Add nullable native columns to `raw_events`:
   - `scope_name`
   - `event_name`
   - `trace_id_hex`
   - `span_id_hex`
   - `trace_flags`
2. Keep legacy columns during compatibility:
   - `target`
   - `family`
   - `subsystem`
   - `run_id`
   - `tick`
3. Add `trace_id_hex` to `ticks`.
4. Change normalize to extract native fields directly from `LogRecord`.
5. Build native ticks from `scope_name = 'beluna.core.stem'` and `event_name = 'tick.granted'`.
6. Query selected tick detail by trace id for native rows, with legacy fallback by `run_id + tick`.

## Frontend Changes

1. Extend bridge contracts and normalized `RawEvent` with native fields:
   - `scopeName`
   - `eventName`
   - `traceId`
   - `spanId`
   - `traceFlags`
2. Add schema-key helpers:
   - `scopeName + eventName`.
3. Add a native chronology path that renders point events by scope/event/span.
4. Keep raw inspector as the guaranteed detailed surface.
5. Adjust labels to prefer:
   - body `summary`
   - `scopeName / eventName`
   - legacy `subsystem / family`

## Implemented Slice 3 Surface

1. Slice 2.5 anchor payload fix for `runtime.booted.run_id` and `tick.granted.run_id/tick/tick_seq`.
2. Backend native raw storage and query fields.
3. Native tick derivation from `tick.granted`.
4. Selected tick detail by trace id.
5. Frontend raw event model with native fields.
6. Native event chronology as point events.
7. Compatibility path for legacy `family + payload`.
8. Tests:
   - backend store test for native trace-backed run/tick/detail projection.
   - frontend projection fixture for native raw events.
   - existing Lachesis query tests updated for native `cortexHandled` semantics.

## Open Decisions

1. Wake identity: Core `run_id` in native logs, Moira-local wake session id, or both.
2. Tick table key: `run_id + tick`, `trace_id_hex`, or a compatibility pair.
3. First UI promise: raw-first native event timeline, or Cortex-primary interval pairing in the first Slice 3 implementation.
4. Durable docs update scope if Moira reconstruction is intentionally simplified.
