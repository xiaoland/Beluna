# Moira Data And State

## Owned State

1. Known local build manifests, installed artifact manifests, install isolation directories, and checksum-verification state.
2. JSONC profile documents keyed by logical `profile_id`, plus later schema-validation result state cached for operator workflows.
3. Supervised Core wake state, including wake/stop status and terminal reason.
4. Local OTLP raw-event storage, including Moira-local compatibility markers for native owner logs, legacy contract logs, and ordinary logs.
5. Local observability read models for `runs` and `ticks`, plus Moira-owned native event timeline projections and targeted lookup indexes when a Loom workflow needs them.
6. Host-local Loom query state such as selected wake, selected tick, open panels, and current selected build/profile refs.
7. Host-facing runtime contracts and host Loom-facing normalized models, each under their owning layer.
8. Process-local resource status for embedded runtimes, including receiver bind and storage/open failures where relevant.

## Consumed State

1. Published Core artifacts and checksum manifests from GitHub Releases.
2. User-provided local Core source folders for development builds.
3. Core OTLP log events and Core exporter status signals.
4. Core schema output used to validate selected JSONC profiles.
5. Host-provided app data paths and platform adapter selections.

## State Ownership By Internal Backend Module

1. `clotho`
- Owns known local build manifests, installed artifact manifests, trusted checksum metadata, install isolation metadata, local source-build outputs or references, JSONC profile documents, and validation result state cached for operator workflows.
- Clotho owns durable preparation truth. Current session-local UI selection belongs to host UI state.
- Within Clotho, artifact state and profile state remain distinct internal concerns even though they share one top-level owner.

2. `lachesis`
- Owns raw OTLP events, `runs` and `ticks`, native event timelines, optional interval-pairing indexes, and any future goal-forest comparison materialization that remains Moira-owned.

3. `atropos`
- Owns supervised wake state, process handles or identifiers, explicit stop intent, and terminal reason state.

4. `host Loom`
- Owns ephemeral UI state such as selected wake, selected tick, popup or panel state, refresh coordination, and current selected build/profile refs.
- Apple Universal owns the first minimum native Loom state in a Settings-integrated operations panel.

## Local Invariants

1. Moira stores full raw OTLP log events locally for the current target design, including full request, response, signal, and topology payloads by default.
2. Raw-event acceptance precedes read-model projection; projections are derived read models.
3. `runs` and `ticks` remain the baseline read models. Moira derives them from Core `beluna.core.main.runtime / booted` and `beluna.core.stem.tick / granted` anchors plus native trace ids.
4. `raw_events.record_kind` is a Moira-local compatibility marker with three current values: `native_owner`, `legacy_contract`, and `ordinary_log`.
5. Legacy contract records are inspectable through raw storage and compatibility normalization when they carry old `family` and serialized `payload` fields.
6. Selected tick detail and source-grounded inspection are raw-first: Moira can show native OTLP event records before deeper owner-specific reconstruction exists.
7. The selected-tick workspace projects a native Core owner-lane timeline for handled ticks and keeps raw inspection available as the strongest detailed surface.
8. Tick is the canonical operator-facing anchor for explainability. Native `traceId` is the primary machine grouping key for one wake plus one tick.
9. Owner interval pairing is an optional Moira projection built from Core boundary records that share owner scope and span id. Event names identify the point or interval boundary on that owner lane.
10. AI transport ids, `thread_id`, `turn_id`, `endpoint_id`, and act routing ids remain inspectable in raw body/attributes and query results. They become dedicated indexes only when a Loom workflow needs them.
11. Metrics and OTLP traces remain exporter-status and handoff-link surfaces in the current target design. Log records may still carry `traceId` and `spanId`.
12. Goal-forest snapshots are stored or referenced per tick; comparisons are derived later between selected ticks.
13. A supervised wake is either explicitly running, explicitly stopping, or explicitly terminal; hidden supervision state is disallowed.
14. Module-owned state must remain writable only through the owning boundary, even when multiple modules share one local database or app-state container.
15. Future Clotho and Atropos persistence belongs to explicit Clotho or Atropos ownership.
16. Clotho may own both artifact and profile preparation, with those persisted concerns kept internally separable.
17. Frontend raw transport contracts, normalized Loom-facing models, and query-owned UI state must remain distinct concerns even when they currently describe the same Lachesis operator flow.
18. `profile_id` remains the durable operator-facing key for profile documents; app-local profile path is derived from that key rather than stored as an independent operator input.
19. Current selected launch-target/profile refs remain session-local query state until an explicit persistence slice lands.
