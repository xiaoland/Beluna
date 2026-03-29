# Moira Data And State

## Owned State

1. Local Core artifact catalog, install isolation, and checksum verification state.
2. JSONC profile documents and active profile selection.
3. Supervised Core run state, including wake/stop status and terminal reason.
4. Local OTLP raw-event storage.
5. Local observability read models for `runs` and `ticks`.
6. Local control-plane UI state.

## Consumed State

1. Published Core artifacts and checksum manifests from GitHub Releases.
2. User-provided local Core source folders for development builds.
3. Core OTLP log events and Core exporter status signals.
4. Core schema output used to validate selected JSONC profiles.

## Local Invariants

1. Moira stores full raw OTLP log events locally for the current target design.
2. Raw-event acceptance precedes read-model projection; projections are derived, not alternative sources of truth.
3. The minimum derived read models are `runs` and `ticks`; tick detail is reconstructable from raw events keyed by run and tick.
4. Dedicated subsystem-specific projections are optional follow-on optimizations and are not required for the first landable slice.
5. Tick is the canonical operator-facing anchor for Cortex inspection.
6. Metrics and traces are not persisted as first-class local observability datasets in the current target design.
7. Goal-forest snapshots are stored or referenced per tick; comparisons are derived later between selected ticks.
8. A supervised run is either explicitly running, explicitly stopping, or explicitly terminal; hidden supervision state is disallowed.
