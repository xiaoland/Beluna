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
5. Goal-forest comparison is derived between two ticks rather than stored as a precomputed diff artifact.
6. Moira lands observability first: the minimum useful slice is raw OTLP ingest plus run- and tick-scoped inspection before artifact-management or supervision expansion.
7. Clotho and Atropos must reuse the same Moira-owned run and query surfaces rather than introducing parallel state models.
