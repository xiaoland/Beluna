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
3. `moira/ffi` owns the first narrow C ABI proof for Apple Universal, currently exposing runtime status JSON, minimum Loom snapshot JSON, Core lifecycle operation JSON, raw profile document JSON, structured profile draft JSON, and known-local-build registration JSON from a process-local `MoiraRuntime`.
4. Clotho currently owns launch-target preparation plus app-local JSONC profile-document list/load/save. `profile_id` is a logical Clotho key that maps to app-local `profiles/<profile-id>.jsonc`.
5. Atropos currently owns runtime status, wake, child process signal-environment restoration, graceful stop, force-kill, and app-exit stop wiring for the supervised Core process.
6. Current Clotho realization includes known local build registration, explicit forge from a local Beluna repo root or `core/` crate root, published release discovery, checksum verification against `SHA256SUMS`, and version-isolated install directories.
7. Schema-validation interactions with Core authority remain deferred to a later Clotho slice.
8. Apple Universal receives the first minimum native Loom surface in the issue #30 slice, and its acceptance criteria come from the selected Apple contract.
9. The legacy Tauri/Vue Loom has been retired from the active Moira code surface after its backend behavior moved into `moira/runtime` and the selected Apple minimum Loom gate was covered.

## Runtime Split

1. `clotho`
- Owns wake-input preparation before Core starts.
- Owns published artifact discovery, checksum trust policy, version isolation, local source-build orchestration, known local build manifests, JSONC profile documents, and schema-validation interactions with Core authority.
- Internal Clotho submodules stay functionally named where that improves readability, for example `artifacts` and `profiles`.

2. `lachesis`
- Owns OTLP receiver lifecycle, raw-event persistence, projections, query surfaces, and ingest pulses.

3. `atropos`
- Owns `wake`, graceful stop, force-kill, supervised process state, readiness gating, and terminal reason tracking.
- On Unix, owns child process signal-environment restoration before `exec`, including clearing inherited signal masks and restoring SIGTERM/SIGINT dispositions for supervised Core.

4. `host API`
- Owns the stable surface exposed to host apps, including typed queries, operations, runtime status, and event/pulse delivery.
- Current Rust implementation lives in `moira/runtime/src/runtime` as `MoiraRuntime`, `MoiraRuntimeConfig`, `MoiraPaths`, `MoiraEventSink`, and `MoiraTaskSpawner`.
- Current Apple proof adapter lives in `moira/ffi` and bridges `MoiraRuntime.status()`, `MoiraRuntime::loom_snapshot(selection)`, Atropos wake/stop/force-kill operations, Clotho raw profile load/save, structured profile draft load/save, and known-local-build registration as C ABI JSON.
- Apple Universal macOS builds package the current FFI adapter as bundled runtime dylibs through the host app target.
- Apple Universal consumes the first narrow host API needed for compact Settings status, standalone Core Control, and standalone O11y / Lachesis browsing.

5. `platform adapters`
- Own OS-specific process, filesystem, permission, sandbox, and ledger integration points.
- Sandbox and ledger implementation belong to later slices.

## Host-Native Loom

Loom is the operator experience, implemented by Human Interface hosts.

Apple Universal first-slice Loom proof used a Settings-integrated operations panel with:

1. Connection and body endpoint socket discovery.
2. Core control context through Clotho and Atropos.
3. Moira runtime and Lachesis receiver status.
4. Launch-target/profile read context.
5. Wake list, tick list, and selected tick raw-first inspection.

Apple Universal follow-on UI splits Moira operator surfaces into three panels:

1. Core Control owns Clotho launch context plus Atropos wake, graceful stop, force kill, terminal reason, and process-state UI.
2. O11y / Lachesis owns wake/tick browsing, selected tick raw-first inspection, raw event inspection, Cortex timeline, narrative investigation, and owner-specific drilldown.
3. Settings owns Moira configuration such as runtime paths, receiver bind address, socket candidates, refresh policy, diagnostics policy, and host-local preferences.

Current Apple Core Control follow-on state has landed the standalone Core Control panel, lifecycle ABI, first Clotho target registration/update controls, and profile draft controls for `core_config`, env files, and inline environment variables.

Current Apple O11y / Lachesis follow-on state has landed the first standalone panel using `MoiraLoomSnapshot` for wake/tick navigation, selected tick summary, raw event selection, and raw JSON inspection. Rich timeline projections, narrative investigation, owner-specific drilldown, and event/pulse refresh remain separate follow-on owners.

Apple Universal Core Control keeps runtime refresh, Core lifecycle, launch-target management, and profile management operation errors scoped to their owning panel sections.

Apple Universal Core Control presents Atropos process state inside Operations with the lifecycle controls.

Legacy Tauri/Vue concepts with durable value have follow-on owners:

1. Apple Core Control owns wake, graceful stop, force kill, terminal reason, and process-state UI.
2. Apple Clotho Management owns launch-target registration, forge/install workflows, and profile editing.
3. Apple O11y / Lachesis owns timeline, narrative, owner-specific panels, and richer raw drilldown.
4. Host Event/Pulse API owns live update delivery for Apple Universal and future Human Interface hosts.

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
11. Mythic names may appear as stable feature namespaces in backend and host UI code, with ownership carried by explicit modules and host layers.
12. Functional names remain preferred inside backend modules and host UI layers where they improve readability and grep-ability.
13. Clotho wake preparation is anchored on a launch-target ref that may resolve to a registered local build or installed release artifact; Atropos consumes Clotho-prepared wake input.
14. Explicit forge is a preparation action owned by Clotho.
15. Host UI helper boundaries follow meaning: source-grounded event interpretation belongs in projection/runtime query owners, while display formatting belongs in host UI.
16. `profile_id` is a logical Clotho identifier. App-local profile path is derived from that key.
17. Current selected launch target and selected profile refs are session-local UI state until an explicit persistence slice lands.
18. During the OTLP migration, Lachesis carries `record_kind` as a Moira-local marker for native owner logs, legacy contract logs, and ordinary logs.
