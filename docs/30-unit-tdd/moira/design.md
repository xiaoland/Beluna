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

## Local Design Invariants

1. Core remains authoritative after launch; Moira supervises but does not become runtime owner.
2. Quitting Moira stops the supervised Core.
3. Force-kill requires a distinct second confirmation path.
4. Logs are first-class locally stored observability data; metrics/traces remain exporter-status and handoff-link surfaces.
5. Human-friendly browsing is a primary Loom responsibility. Raw JSON inspection is a secondary drilldown surface, not the main browsing mode.
6. Full payload preservation is preferred for first-party local observability while Beluna is still in the observability-heavy early phase.
7. Selected tick inspection is organized around tick-scoped chronology and lane grouping rather than one flat event list.
8. Goal-forest comparison is derived between two ticks rather than stored as a precomputed diff artifact.
9. Clotho and Atropos must reuse the same Moira-owned wake and query surfaces rather than introducing parallel state models.
