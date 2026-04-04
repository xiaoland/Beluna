# Moira Data And State

## Owned State

1. Known local build manifests, installed artifact manifests, install isolation directories, and checksum-verification state.
2. JSONC profile documents keyed by logical `profile_id`, plus later schema-validation result state cached for operator workflows.
3. Supervised Core wake state, including wake/stop status and terminal reason.
4. Local OTLP raw-event storage.
5. Local observability read models for `runs` and `ticks`, plus any Moira-owned chronology, interval-pairing, or targeted lookup indexes needed for human-friendly browsing.
6. Session-local Loom query state such as active feature tab, selected wake, selected tick, active Lachesis detail tab, open dialogs, and current selected build/profile refs.
7. Frontend backend-shaped transport contracts and frontend Loom-facing normalized models, each under their owning layer rather than inside one shared bucket.

## Consumed State

1. Published Core artifacts and checksum manifests from GitHub Releases.
2. User-provided local Core source folders for development builds.
3. Core OTLP log events and Core exporter status signals.
4. Core schema output used to validate selected JSONC profiles.

## State Ownership By Internal Backend Module

1. `clotho`
- Owns known local build manifests, installed artifact manifests, trusted checksum metadata, install isolation metadata, local source-build outputs or references, JSONC profile documents, and validation result state cached for operator workflows.
- Clotho owns durable preparation truth, not the current session-local UI selection for the next wake.
- Within Clotho, artifact state and profile state remain distinct internal concerns even though they share one top-level owner.

2. `lachesis`
- Owns raw OTLP events, `runs` and `ticks`, chronology or interval-pairing indexes, and any future goal-forest comparison materialization that remains Moira-owned.

3. `atropos`
- Owns supervised wake state, process handles or identifiers, explicit stop intent, and terminal reason state.

4. `loom`
- Owns ephemeral UI state such as active feature tab, selected wake, selected tick, active Lachesis detail tab, popup or dialog state, refresh coordination, and current selected build/profile refs.

## Local Invariants

1. Moira stores full raw OTLP log events locally for the current target design, including full request, response, signal, and topology payloads by default.
2. Raw-event acceptance precedes read-model projection; projections are derived, not alternative sources of truth.
3. `runs` and `ticks` remain the baseline read models. Moira may add lightweight chronology, interval-pairing, or targeted lookup indexes where humane browsing would otherwise require reparsing raw payload blobs in the view layer.
4. Selected tick detail, per-tick chronology, nested AI transport investigation, nested chat-capability investigation, and source-grounded inspection remain reconstructable from raw events plus Moira-owned indexes.
5. The selected-tick workspace projects a primary tick chronology from raw events before falling back to sectional Cortex/Stem/Spine inspections.
6. Tick is the canonical operator-facing anchor for explainability and the primary trace selector in Loom.
7. Cortex interval pairing is a Moira-owned projection responsibility built from Core boundary records and stable operation keys such as `request_id`.
8. AI request ids, `thread_id`, `turn_id`, and `endpoint_id` remain inspectable in event bodies and query results without becoming first-class chronology keys by default.
9. Metrics and traces are not persisted as first-class local observability datasets in the current target design.
10. Goal-forest snapshots are stored or referenced per tick; comparisons are derived later between selected ticks.
11. A supervised wake is either explicitly running, explicitly stopping, or explicitly terminal; hidden supervision state is disallowed.
12. Module-owned state must remain writable only through the owning boundary, even when multiple modules share one local database or app-state container.
13. Future Clotho and Atropos persistence must not be folded into Lachesis projections or Lachesis tables as a convenience shortcut.
14. Clotho may own both artifact and profile preparation, but those persisted concerns must remain internally separable rather than collapsing into one undifferentiated preparation blob.
15. Frontend raw transport contracts, normalized Loom-facing models, and query-owned UI state must remain distinct concerns even when they currently describe the same Lachesis operator flow.
16. `profile_id` remains the durable operator-facing key for profile documents; app-local profile path is derived from that key rather than stored as an independent operator input.
17. Current selected launch-target/profile refs remain session-local query state until an explicit persistence slice lands; they must not be mistaken for durable Clotho truth.
