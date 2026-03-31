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
2. Clotho artifact and profile preparation, plus Atropos supervision, remain part of Moira's target responsibility but are not yet fully realized in code.
3. The next internal cleanup stage is behavior-preserving. Its purpose is to establish maintainable internal boundaries before those additional responsibilities expand.

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

2. `query state`
- Owns wake selection, tick selection, refresh orchestration, loading state, and cross-view app state.

3. `projection`
- Owns normalization, chronology reconstruction, interval pairing, AI drilldown linking, and narrative shaping.

4. `presentation`
- Owns Vue view composition, interaction widgets, and JSON inspectors.
- It should consume normalized projections rather than reinterpret OTLP family semantics inline.

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
15. Mythic names belong at the top-level backend boundary only. Inside those modules, functional names remain preferred where they improve readability and grep-ability.
