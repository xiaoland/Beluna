# Moira Data And State

## Owned State

1. Local Core artifact catalog, install isolation, and checksum verification state.
2. JSONC profile documents and active profile selection.
3. Supervised Core wake state, including wake/stop status and terminal reason.
4. Local OTLP raw-event storage.
5. Local observability read models for `runs` and `ticks`, plus any Moira-owned tick-trace or thread indexes needed for human-friendly browsing.
6. Local control-plane UI state.

## Consumed State

1. Published Core artifacts and checksum manifests from GitHub Releases.
2. User-provided local Core source folders for development builds.
3. Core OTLP log events and Core exporter status signals.
4. Core schema output used to validate selected JSONC profiles.

## Local Invariants

1. Moira stores full raw OTLP log events locally for the current target design, including full request, response, signal, and topology payloads by default.
2. Raw-event acceptance precedes read-model projection; projections are derived, not alternative sources of truth.
3. `runs` and `ticks` remain the baseline read models. Moira may add lightweight tick-trace or thread indexes where humane browsing would otherwise require reparsing raw payload blobs in the view layer.
4. Selected tick detail, per-tick chronology, and thread browsing remain reconstructable from raw events plus Moira-owned indexes.
5. The selected-tick workspace projects a primary lane-based chronology from raw events before falling back to sectional Cortex/Stem/Spine drilldowns.
6. Chronology lanes are resolved from the strongest stable identity available for each family group rather than from family names themselves.
7. Tick is the canonical operator-facing anchor for explainability and the primary trace selector in Loom.
8. Metrics and traces are not persisted as first-class local observability datasets in the current target design.
9. Goal-forest snapshots are stored or referenced per tick; comparisons are derived later between selected ticks.
10. A supervised wake is either explicitly running, explicitly stopping, or explicitly terminal; hidden supervision state is disallowed.
