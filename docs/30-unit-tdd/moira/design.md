# Moira Design

## Responsibility

Moira provides Beluna's first-party local control plane for:

1. Local Core artifact preparation and version isolation.
2. Local Core wake/stop supervision.
3. OTLP log ingestion, local storage, query, and inspection workflows.
4. Human-facing control and observability UI through Loom.

## Non-Responsibility

1. Moira does not own Core runtime behavior, cognition, routing, or persistence authority.
2. Moira does not own the Core config schema shape or validation rules as an independent authority.
3. Moira does not redefine Core observability emission semantics.
4. Moira does not replace `apple-universal` or other body endpoint UX as endpoint authorities.
5. Moira does not make metrics or traces first-class locally stored signals in the current target design.

## Current Realization Boundary

1. Current code primarily realizes Lachesis local ingestion, storage, query, and Loom inspection for one wake and one selected tick.
2. The backend cleanup split is now explicit in code: `app` composes `clotho`, `lachesis`, and `atropos`, while Lachesis remains the most mature backend owner and Clotho/Atropos now also own operator-facing control-plane behavior.
3. The frontend cleanup and integration pass are now explicit in code: `app/LoomApp.vue` composes the Lachesis workspace, `bridge` owns backend-shaped contracts, `query` owns Lachesis UI state, and the former catch-all helper files are replaced by explicit `projection/lachesis/*` and `presentation/*` owners.
4. Clotho now has operator-facing known local build registration plus app-local JSONC profile document management in Loom; Atropos now has operator-facing runtime status, wake, graceful stop, and force-kill with second confirmation.
5. Published artifact discovery, checksum verification, local source-folder compile, and schema-validation interactions with Core authority remain later Clotho slices rather than current realization.
6. The cleanup stage remains behavior-preserving. Its purpose is to finish establishing maintainable frontend and cross-slice boundaries while those additional responsibilities expand.

## Internal Split

### Tauri Backend Modules

1. `clotho`
- Owns wake-input preparation before Core starts.
- Owns published artifact discovery, checksum trust policy, version isolation, local source-build orchestration, JSONC profile documents, active profile selection, and schema-validation interactions with Core authority.
- Internal Clotho submodules should stay functionally named where that improves readability, for example `artifacts` and `profiles`.

2. `lachesis`
- Owns OTLP receiver lifecycle, raw-event persistence, projections, query surfaces, and ingest-to-Loom event pulses.

3. `atropos`
- Owns `wake`, graceful stop, force-kill, supervised process state, readiness gating, and terminal reason tracking.

4. `app`
- Owns module composition, Tauri command exposure, and app-wide event wiring.
- It is a transport and composition layer, not the owner of Clotho, Lachesis, or Atropos behavior.

### Loom Frontend Layers

1. `bridge`
- Owns Tauri invoke and event subscription bindings only.
- Owns raw Tauri command and event payload contracts for the backend surfaces it calls.
- It should return backend-shaped payloads rather than normalized Loom-facing models.

2. `query state`
- Owns wake selection, tick selection, refresh orchestration, loading state, and cross-view app state.

3. `projection`
- Owns normalization, chronology reconstruction, interval pairing, AI drilldown linking, and narrative shaping.
- Owns Lachesis-specific event labeling, raw-event headlines, narrative section assembly, and JSON drilldown section assembly because those remain source-grounded observability interpretation rather than pure visual formatting.
- During the current cleanup stage, `projection/lachesis/*` is the preferred landing shape for those responsibilities instead of one catch-all helper file.
- Owns normalized Loom-facing models. Raw bridge contracts must not become the de facto model type used by presentation.

4. `presentation`
- Owns Vue view composition, interaction widgets, and JSON inspectors.
- Owns display-only formatting such as time rendering, count rendering, and tone mapping.
- It should consume normalized projections rather than reinterpret OTLP family semantics inline.
- During the current cleanup stage, presentation should stay organized around Loom chrome plus Lachesis operator tasks such as workspace, chronology, narratives, and inspectors, while future Clotho and Atropos views remain separate feature namespaces when they arrive.

## Local Design Invariants

1. Core remains authoritative after launch; Moira supervises but does not become runtime owner.
2. Quitting Moira stops the supervised Core.
3. Force-kill requires a distinct second confirmation path.
4. Logs are first-class locally stored observability data; metrics/traces remain exporter-status and handoff-link surfaces.
5. Human-friendly browsing is a primary Loom responsibility. Raw JSON inspection is a secondary drilldown surface, not the main browsing mode.
6. Full payload preservation is preferred for first-party local observability while Beluna is still in the observability-heavy early phase.
7. Selected tick inspection is organized around tick-scoped chronology and interval work rather than one flat event list.
8. Goal-forest comparison is derived between two ticks rather than stored as a precomputed diff artifact.
9. Clotho and Atropos must reuse the same Moira-owned wake and query surfaces rather than introducing parallel state models.
10. AI transport and chat-capability investigation are normally entered from tick chronology or expanded Cortex intervals rather than through one separate first-class AI browsing mode.
11. Source-grounded inspection means every human-friendly interpretation in Loom can be traced back to the supporting raw OTLP records without leaving the selected wake or tick context.
12. Tauri commands are transport façades over internal backend owners; they must not become catch-all owners of process, preparation, or projection logic.
13. Loom root views and presentational components must not become the primary owners of OTLP interpretation; projection logic belongs in the dedicated query/projection layer.
14. Lachesis modules and Lachesis storage must not become a catch-all home for Clotho or Atropos state merely because they already persist data.
15. Mythic names may appear as stable feature namespaces in both backend and frontend code, but ownership must still remain explicit through backend modules and frontend layers.
16. Inside those backend modules and frontend layers, functional names remain preferred where they improve readability and grep-ability.
17. Frontend helper boundaries follow meaning rather than file history: source-grounded event interpretation belongs in `projection`, while locale formatting and visual tone helpers belong in `presentation`.
18. Frontend contract boundaries follow ownership, not convenience: backend-shaped payload contracts belong in `bridge`, Loom-facing normalized models belong in `projection`, and query-owned UI state must not collapse back into one shared catch-all type bucket.
