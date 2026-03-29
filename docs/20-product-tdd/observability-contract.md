# Observability Contract

This file defines the authoritative cross-unit contract between Core OTLP log emission and Moira local observability consumption.

Core-internal subsystem organization belongs in Core Unit TDD.
Loom screen composition and operator interaction design belong in Moira Unit TDD.

## Scope

1. Beluna's first-party local observability contract is log-first.
2. `core` owns OTLP log emission semantics; `moira` owns local ingestion, storage, query, and control-plane behavior built on those semantics.
3. Metrics and traces are limited to exporter status and handoff destinations in the current contract.

## Cross-Unit Reconstruction Rules

1. Moira must be able to reconstruct one local run from raw Core OTLP log events plus Moira-owned supervision state.
2. In observability and local control-plane contexts, `tick` is the canonical operator-facing anchor for one cognition cycle.
3. Free-form message or body fields may supplement debugging, but they must not be the sole source for cycle, signal-flow, topology, or dispatch reconstruction.
4. Goal-forest comparison is derived from selected snapshots rather than emitted as canonical diff state.
5. Raw OTLP log events must remain preservable for drilldown from higher-level inspection surfaces.

## Required Structured Observability Surfaces

1. Run-scoping surface
- Core logs must expose stable run correlation fields and timestamps sufficient to scope emitted events to one runtime execution.

2. Cycle-inspection surface
- Core logs must expose structured records sufficient to inspect one tick as a unit, including trigger context, selected or consumed senses summary, proprioception snapshot or stable reference, emitted acts summary, and goal-forest snapshot linkage.

3. Goal-forest snapshot surface
- Core logs must expose per-tick goal-forest snapshot data or stable references sufficient for later side-by-side comparison.

4. Signal-flow surface
- Core logs must expose structured records for afferent and efferent flow transitions, including correlation identity, descriptor identity, endpoint identity when relevant, and `tick` when known.

5. Descriptor and topology surface
- Core logs must expose structured records for descriptor-catalog changes, adapter lifecycle changes, and endpoint lifecycle changes.

6. Dispatch-outcome surface
- Core logs must expose structured terminal dispatch outcomes with enough correlation and binding information to connect them back to the related cycle, signal, endpoint, or act.

## Consumer Guarantees

1. Moira may rely on the required surfaces above to implement:
- run-scoped runtime inspection
- cycle reconstruction and goal-forest comparison
- signal-flow reconstruction
- topology and dispatch reconstruction

2. Product TDD does not freeze Core subsystem family names or Moira screen/view composition; those belong to the respective unit-local contracts as long as the cross-unit guarantees remain intact.

## Non-Guarantees In Current Contract

1. First-party local metrics dashboards or trace explorers.
2. Canonical precomputed goal-forest diff storage.
3. Cross-run analytics or fleet-wide aggregation.
4. One fixed UI decomposition for Loom.

## Compatibility Rule

1. Removing one of the required structured observability surfaces, or dropping structured semantics that Moira depends on for reconstruction, is a breaking cross-unit change.
2. Core may evolve its internal subsystem event-family catalog and Moira may evolve Loom composition as long as the cross-unit reconstruction guarantees remain intact.
3. Breaking changes require synchronized updates to Product TDD, the affected Core and Moira Unit TDD docs, and corresponding verification guardrails.
