# Moira Design

## Responsibility

Moira provides Beluna's first-party local control-plane runtime for:

1. Local Core artifact preparation and version isolation.
2. Local Core wake/stop supervision.
3. OTLP log ingestion, local storage, query, and inspection workflows.
4. Host-facing APIs for Loom operator surfaces.
5. Future sandbox and ledger supervision through platform adapters.

## Authority Boundaries

1. Core retains runtime behavior, cognition, routing, persistence authority, config schema shape, endpoint protocol authority, and observability emission semantics.
2. Moira owns local preparation, supervision, observability ingestion/storage/query/projection, and future platform adapter policies.
3. Beluna Human Interface hosts own platform-native UX, endpoint interaction presentation, and host-native Loom composition.
4. Metrics and traces remain exporter-status and handoff-link surfaces for the current Moira design.

## Current Realization Boundary

1. Current Rust backend code realizes Lachesis local ingestion, storage, query, and inspection for one wake and one selected tick.
2. The backend owner split is explicit in code: `moira/runtime` owns `clotho`, `lachesis`, `atropos`, and the host-facing `runtime` API.
3. The Tauri container under `moira/src-tauri` is a transitional adapter that resolves Tauri app paths, installs a Tauri event sink/task spawner, and exposes existing command names.
4. Clotho currently owns launch-target preparation plus app-local JSONC profile-document list/load/save. `profile_id` is a logical Clotho key that maps to app-local `profiles/<profile-id>.jsonc`.
5. Atropos currently owns runtime status, wake, graceful stop, force-kill, and app-exit stop wiring for the supervised Core process.
6. Current Clotho realization includes known local build registration, explicit forge from a local Beluna repo root or `core/` crate root, published release discovery, checksum verification against `SHA256SUMS`, and version-isolated install directories.
7. Schema-validation interactions with Core authority remain deferred to a later Clotho slice.
8. The current Tauri/Vue Loom is transitional implementation evidence. Apple Universal receives the first minimum native Loom surface in the issue #30 slice.

## Runtime Split

1. `clotho`
- Owns wake-input preparation before Core starts.
- Owns published artifact discovery, checksum trust policy, version isolation, local source-build orchestration, known local build manifests, JSONC profile documents, and schema-validation interactions with Core authority.
- Internal Clotho submodules stay functionally named where that improves readability, for example `artifacts` and `profiles`.

2. `lachesis`
- Owns OTLP receiver lifecycle, raw-event persistence, projections, query surfaces, and ingest pulses.

3. `atropos`
- Owns `wake`, graceful stop, force-kill, supervised process state, readiness gating, and terminal reason tracking.

4. `host API`
- Owns the stable surface exposed to host apps, including typed queries, operations, runtime status, and event/pulse delivery.
- Current Rust implementation lives in `moira/runtime/src/runtime` as `MoiraRuntime`, `MoiraRuntimeConfig`, `MoiraPaths`, `MoiraEventSink`, and `MoiraTaskSpawner`.
- Apple Universal consumes the first narrow host API needed for Settings-integrated minimum Loom.

5. `platform adapters`
- Own OS-specific process, filesystem, permission, sandbox, and ledger integration points.
- Sandbox and ledger implementation belong to later slices.

6. `transitional app adapter`
- The current Tauri app composes `moira/runtime` and Vue Loom during migration.
- Tauri command handlers stay as transport facades while Apple Universal coverage lands.

## Host-Native Loom

Loom is the operator experience, implemented by Human Interface hosts.

Apple Universal first-slice Loom uses a Settings-integrated operations panel with:

1. Connection and body endpoint socket discovery.
2. Core control context through Clotho and Atropos.
3. Moira runtime and Lachesis receiver status.
4. Wake list, tick list, and selected tick raw-first inspection.

The current frontend layering in `moira/src` remains useful as migration input:

1. `bridge` shows backend-shaped transport contracts.
2. `query` shows selection and refresh orchestration.
3. `projection` shows source-grounded normalization and narrative ownership.
4. `presentation` shows the current Vue composition that Apple-native UI will replace selectively.

## Runtime Multiplicity

The issue #30 first implementation uses process-local embedded Moira runtime inside Apple Universal.

1. Apple Universal can start Core through embedded Clotho and Atropos.
2. Apple Universal can use body endpoint socket discovery to connect to Core started by another process or prior session.
3. Local resource conflicts, such as an occupied OTLP receiver port or locked storage path, surface as runtime status.
4. Cross-client Owner/Attach authority coordination belongs to a later task packet.

## Local Design Invariants

1. Core remains authoritative after launch; Moira supervises local lifecycle.
2. Force-kill requires a distinct second confirmation path.
3. Logs are first-class locally stored observability data.
4. Human-friendly browsing is a primary Loom responsibility. Raw JSON inspection is the strongest source-grounded fallback.
5. Full payload preservation is preferred for first-party local observability while Beluna is in the observability-heavy early phase.
6. Selected tick inspection is organized around a Core owner-lane timeline, narrative modes, and sectional subsystem views.
7. Goal-forest comparison is derived between two ticks.
8. Clotho and Atropos reuse the same Moira-owned wake and query surfaces.
9. AI transport and chat-capability investigation normally enters from the Core owner-lane timeline or expanded owner intervals.
10. Every human-friendly interpretation in Loom traces back to supporting raw OTLP records inside the selected wake or tick context.
11. Tauri commands are transport facades over internal backend owners during migration.
12. Mythic names may appear as stable feature namespaces in backend and host UI code, with ownership carried by explicit modules and host layers.
13. Functional names remain preferred inside backend modules and host UI layers where they improve readability and grep-ability.
14. Clotho wake preparation is anchored on a launch-target ref that may resolve to a registered local build or installed release artifact; Atropos consumes Clotho-prepared wake input.
15. Explicit forge is a preparation action owned by Clotho.
16. Host UI helper boundaries follow meaning: source-grounded event interpretation belongs in projection/runtime query owners, while display formatting belongs in host UI.
17. `profile_id` is a logical Clotho identifier. App-local profile path is derived from that key.
18. Current selected launch target and selected profile refs are session-local UI state until an explicit persistence slice lands.
19. During the OTLP migration, Lachesis carries `record_kind` as a Moira-local marker for native owner logs, legacy contract logs, and ordinary logs.
