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

1. Current code realizes Lachesis local ingestion, storage, query, and Loom inspection for one wake and one selected tick.
2. The backend owner split is explicit in code: `app` composes `clotho`, `lachesis`, and `atropos`, while each backend owner now has operator-facing responsibilities in the current shell.
3. The frontend owner split is explicit in code: `app/LoomApp.vue` is a thin root shell, `bridge` owns backend-shaped contracts, `query` owns station-local orchestration, `projection` owns normalization, and `presentation` owns shell chrome plus feature panels and dialogs.
4. Loom now exposes three mythic operator stations instead of one stacked control page: `Lachesis`, `Atropos`, and `Clotho`.
5. Clotho currently owns launch-target preparation plus app-local JSONC profile-document list/load/save. `profile_id` is a logical Clotho key that maps to app-local `profiles/<profile-id>.jsonc`.
6. Atropos currently owns runtime status, wake, graceful stop, force-kill, and app-exit stop wiring for the supervised Core process.
7. Current Clotho realization now includes:
   - known local build registration
   - explicit forge from a local Beluna repo root or `core/` crate root
   - published release discovery for the current supported target
   - checksum verification against `SHA256SUMS`
   - version-isolated install directories for installed artifacts
8. Schema-validation interactions with Core authority remain deferred rather than current realization.
9. Future growth should extend these explicit owners rather than reviving catch-all backend modules, catch-all query state, or one permanently stacked Loom control surface.

## Internal Split

### Tauri Backend Modules

1. `clotho`
- Owns wake-input preparation before Core starts.
- Owns published artifact discovery, checksum trust policy, version isolation, local source-build orchestration, known local build manifests, JSONC profile documents, and schema-validation interactions with Core authority.
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
- Owns wake selection, tick selection, runtime refresh orchestration, loading state, station-local dialog state, and cross-station app state such as the active Loom feature tab.
- Current explicit owners include `query/lachesis/workspace`, `query/atropos/runtime`, `query/clotho/builds`, `query/clotho/profiles`, and `query/loom/navigation`.

3. `projection`
- Owns normalization, chronology reconstruction, interval pairing, AI drilldown linking, and narrative shaping.
- Owns Lachesis-specific event labeling, raw-event headlines, narrative section assembly, and JSON drilldown section assembly because those remain source-grounded observability interpretation rather than pure visual formatting.
- During the current cleanup stage, `projection/lachesis/*` is the preferred landing shape for those responsibilities instead of one catch-all helper file.
- Owns normalized Loom-facing models. Raw bridge contracts must not become the de facto model type used by presentation.

4. `presentation`
- Owns Vue view composition, interaction widgets, and JSON inspectors.
- Owns display-only formatting such as time rendering, count rendering, and tone mapping.
- It should consume normalized projections rather than reinterpret OTLP family semantics inline.
- Presentation stays organized around Loom chrome plus feature namespaces such as `lachesis/workspace`, `lachesis/chronology`, `lachesis/inspectors`, `atropos/runtime`, `clotho/workshop`, and `clotho/dialogs`.
- Shared shell affordances such as feature tabs, status chrome, and modal scaffolding belong in `presentation/loom/chrome`; feature-specific semantics stay inside the corresponding mythic namespace.

## Local Design Invariants

1. Core remains authoritative after launch; Moira supervises but does not become runtime owner.
2. Quitting Moira stops the supervised Core.
3. Force-kill requires a distinct second confirmation path.
4. Logs are first-class locally stored observability data; metrics/traces remain exporter-status and handoff-link surfaces.
5. Human-friendly browsing is a primary Loom responsibility. Raw JSON inspection is a secondary drilldown surface, not the main browsing mode.
6. Full payload preservation is preferred for first-party local observability while Beluna is still in the observability-heavy early phase.
7. Selected tick inspection is organized around Cortex-focused timeline and narrative modes plus sectional subsystem views rather than one flat event list.
8. Goal-forest comparison is derived between two ticks rather than stored as a precomputed diff artifact.
9. Clotho and Atropos must reuse the same Moira-owned wake and query surfaces rather than introducing parallel state models.
10. AI transport and chat-capability investigation are normally entered from the Cortex timeline mode or expanded Cortex intervals rather than through one separate first-class AI browsing mode.
11. Source-grounded inspection means every human-friendly interpretation in Loom can be traced back to the supporting raw OTLP records without leaving the selected wake or tick context.
12. Tauri commands are transport façades over internal backend owners; they must not become catch-all owners of process, preparation, or projection logic.
13. Loom root views and presentational components must not become the primary owners of OTLP interpretation; projection logic belongs in the dedicated query/projection layer.
14. Lachesis modules and Lachesis storage must not become a catch-all home for Clotho or Atropos state merely because they already persist data.
15. Mythic names may appear as stable feature namespaces in both backend and frontend code, but ownership must still remain explicit through backend modules and frontend layers.
16. Inside those backend modules and frontend layers, functional names remain preferred where they improve readability and grep-ability.
17. Clotho wake preparation is anchored on a launch-target ref that may resolve to a registered local build or an installed release artifact; Atropos still consumes only Clotho-prepared wake input.
18. Explicit forge is a preparation action owned by Clotho, not an implicit wake-time side effect owned by Atropos.
19. Frontend helper boundaries follow meaning rather than file history: source-grounded event interpretation belongs in `projection`, while locale formatting and visual tone helpers belong in `presentation`.
20. Frontend contract boundaries follow ownership, not convenience: backend-shaped payload contracts belong in `bridge`, Loom-facing normalized models belong in `projection`, and query-owned UI state must not collapse back into one shared catch-all type bucket.
21. `profile_id` is a logical Clotho identifier, not a raw filesystem path; Loom may show the derived path, but path derivation remains Clotho-owned behavior.
22. Current selected launch target and selected profile refs are session-local query state until an explicit persistence slice lands; app-local Clotho documents and manifests remain the durable preparation truth.
