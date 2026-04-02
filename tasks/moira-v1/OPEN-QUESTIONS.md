# Open Questions

This file tracks unresolved decisions that are large enough to change design or sequencing. If a choice is temporary, keep it here until code and promotion make it real.

## Decisions Already Fixed

- macOS-first
- new `/moira` unit
- Tauri + Rust + Vue/TypeScript
- logs-first
- DuckDB embedded store
- GitHub prereleases first
- JSONC-only config editing
- Core-only supervision in v1
- Quitting Moira also stops the supervised Core.
- Force-kill requires a second confirmation step.
- Config remains JSONC-only for now.
- Local development can point Clotho at a Core source folder and compile before launch.
- `tick` is the preferred product-facing name for the current `cycle_id` anchor.
- Goal-forest comparison is derived between two ticks and is not stored as a precomputed DB diff.
- Metrics and traces remain exporter-status plus handoff-link surfaces only.
- Moira may eventually launch first-party endpoint apps.
- `apple-universal` remains a pure body endpoint UX.
- Clotho trusts GitHub release assets using `beluna-core-<rust-target-triple>.tar.gz` archives and a `SHA256SUMS` checksum file.
- Current macOS-first expected release asset is `beluna-core-aarch64-apple-darwin.tar.gz`.
- For first-party local observability, full payload preservation is preferred over summary-only capture.
- Every Core event consumed by Loom is tick-scoped; `tick` is the root trace anchor for the local tick workspace.
- `ai-gateway.request` is the preferred Stage 2 family name for backend-governed request and retry lifecycle.
- When adapters cannot expose intermediate provider bodies cleanly, `ai-gateway.request` preserves provider request payloads at terminal states rather than forcing synthetic partial-body events.
- The first humane chronology view remains raw-first in the query layer.
- Family naming and lane identity are separate concerns: event families stay owner-centric, while lane types stay entity-centric.
- Current Core observability event mess is migration input, not target contract truth for Stage 2.
- Current canonical Stage 2 names in code are:
  - `ai-gateway.request`
  - `ai-gateway.chat.turn`
  - `ai-gateway.chat.thread`
  - `cortex.primary`
  - `cortex.sense-helper`
  - `cortex.goal-forest-helper`
  - `cortex.acts-helper`
  - `cortex.goal-forest`
  - `stem.tick`
  - `stem.afferent`
  - `stem.efferent`
  - `stem.proprioception`
  - `stem.ns-catalog`
  - `stem.afferent.rule`
  - `spine.adapter`
  - `spine.endpoint`
  - `spine.sense`
  - `spine.act`
- Older task-buffer names such as `ai-gateway.turn`, `cortex.organ`, `stem.signal`, `stem.dispatch`, `stem.descriptor.catalog`, and `spine.dispatch` are historical draft vocabulary only.
- Some Core fixture ids still retain older scenario labels. Those fixture ids are not the canonical family names consumed by Moira.
- Preferred primary lane resolution matrix:
  - `cortex.primary|cortex.sense-helper|cortex.goal-forest-helper|cortex.acts-helper|cortex.goal-forest` -> lane type `cortex`; key `request_id > family > raw_event_id`
  - `ai-gateway.request|ai-gateway.chat.turn|ai-gateway.chat.thread` -> lane type `misc` in the current projection, with interval attachment via `request_id|ai_request_id|thread_id|turn_id` when a related Cortex interval exists
  - `stem.afferent` -> lane type `afferent`; key `sense_id > descriptor_id > endpoint_id > raw_event_id`
  - `stem.efferent` -> lane type `efferent`; key `act_id > descriptor_id > endpoint_id > raw_event_id`
  - `spine.endpoint|spine.adapter|spine.sense|spine.act` -> lane type `spine`; key `endpoint_id > adapter_id > act_id > sense_id > raw_event_id`

## Next Stage Working Defaults

These defaults should be used for the next stages unless implementation pressure exposes a concrete problem.

### For Stage 2

- replace the current summary-first AI-gateway and Cortex organ events with full-payload owner-centric families
- keep the Stage 2 logical catalog coarse-grained:
  - `ai-gateway.request`
  - `ai-gateway.chat.turn`
  - `ai-gateway.chat.thread`
  - `cortex.primary`
  - `cortex.sense-helper`
  - `cortex.goal-forest-helper`
  - `cortex.acts-helper`
  - `cortex.goal-forest`
  - `stem.tick`
  - `stem.afferent`
  - `stem.efferent`
  - `stem.proprioception`
  - `stem.ns-catalog`
  - `stem.afferent.rule`
  - `spine.adapter`
  - `spine.endpoint`
  - `spine.sense`
  - `spine.act`
- require `tick` on every Core event consumed by Moira
- use `tick = 0` for startup or pre-first-grant events rather than leaving them outside the model
- require lane keys sufficient for a per-tick gantt or lane chronology
- keep `runs` and `ticks`
- keep raw OTLP events authoritative
- keep the first humane chronology view raw-first in the query layer
- allow a narrow `tick_lanes` browse projection only if later query assembly proves awkward enough to justify it
- retain full payloads in raw storage
- keep raw JSON drilldown secondary to the structured tick workspace

### For Stage 3

- land the structured tick workspace before introducing a large projection lattice
- add AI-gateway, Stem, and Spine dedicated panels only where the Stage 2 contract makes them readable
- keep broader Stem/Spine indexes deferred until the tick workspace proves they are necessary

### For Stage 4 And Later

- supervision continues to reuse the same wake/tick status surfaces
- wider retention, compaction, and migration work remains later hardening, not a blocker for the contract rewrite

## Narrow Questions To Resolve Right Before Stage 2 Code

1. Should `ai-gateway.request` expose `provider_request` on every request-lifecycle kind when the adapter can produce it cheaply, or only on terminal kinds by default?

2. Do any Stage 2 Loom views need a documented secondary grouping inside one primary lane, or is the current primary lane matrix sufficient for the first humane chronology?

## Deferred Until Stage 5

- Do we require detached signatures in addition to `SHA256SUMS` before calling the artifact story production-ready?

## Deferred But Defaulted For This Task

- Retention and compaction do not block this task; manual reset is acceptable during early development.
- Use a simple linear DuckDB schema version and fail closed on incompatible schema until hardening work lands.
- Broad automated test expansion is not a default blocker during MVP; live end-to-end inspection is the preferred evidence while Moira read models and views still churn.

## Future-Scope Boundaries

- When Moira later launches first-party endpoint apps, how should their supervision model differ from Core supervision?
