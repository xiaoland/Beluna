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

## Next Stage Working Defaults

These defaults should be used for the next stages unless implementation pressure exposes a concrete problem.

### For Stage 1

- store full raw OTLP log events
- materialize `runs`
- materialize `ticks`
- reconstruct selected tick detail from raw events rather than introducing dedicated Cortex/Stem/Spine tables immediately
- treat run list, tick timeline, and tick detail as the only required Loom surfaces for the first landable slice

### For Stage 2

- close only the Core structured-log gaps exposed by Stage 1 surfaces
- materialize `goal_forest_snapshots` only if selected tick detail or compare workflows need a dedicated table
- keep `signals`, `descriptor_catalog_snapshots`, `topology_events`, and `dispatch_outcomes` for the later dedicated stage unless Stage 1 proves otherwise
- retain full raw payloads in the raw table
- keep projection tables summary-first and preview-first instead of duplicating large bodies
- snapshot proprioception and goal-forest state per tick while Beluna is still in the observability-heavy early phase

### For Stage 3 And Stage 4

- materialize `signals`
- materialize `descriptor_catalog_snapshots`
- materialize `topology_events`
- materialize `dispatch_outcomes`
- complete Stem and Spine dedicated views inside this task

## Deferred Until Stage 5

- Do we require detached signatures in addition to `SHA256SUMS` before calling the artifact story production-ready?

## Deferred But Defaulted For This Task

- Retention and compaction do not block this task; manual reset is acceptable during early development.
- Use a simple linear DuckDB schema version and fail closed on incompatible schema until hardening work lands.
- The query boundary stays raw-first with only the derived tables already listed in this packet.
- Broad automated test expansion is not a default blocker during MVP; live end-to-end inspection is the preferred evidence while Moira read models and views still churn.
- Any event family not explicitly named in `L2.md` remains debug-only for now.

## Future-Scope Boundaries

- When Moira later launches first-party endpoint apps, how should their supervision model differ from Core supervision?
