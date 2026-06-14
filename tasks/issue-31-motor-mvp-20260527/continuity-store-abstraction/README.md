# Continuity Generic Store Abstraction

> Last Updated: 2026-06-14
> Status: implemented in Core; durable docs still pending
> Scope: Continuity durable record boundary for Issue 31 prerequisites

## Objective

Evolve Continuity toward a generic durable store boundary without coupling it to
Motor routine semantics.

Routine source persistence is the first forcing case, but the sub-task is not
"add routine persistence to Continuity". The target is:

```text
Continuity owns generic durable records.
Motor owns routine meaning.
Routine source is stored as ordinary durable record payload.
```

## Current Position

The previous exploration note has been moved to
[CONTINUITY-STORE-ABSTRACTION.md](./CONTINUITY-STORE-ABSTRACTION.md).

Current code reality is recorded in
[CURRENT-REALITY.md](./CURRENT-REALITY.md).

Implementation result is recorded in
[IMPLEMENTATION-RESULT.md](./IMPLEMENTATION-RESULT.md).

Core topology correction is already implemented and recorded under
[../core-topology-correction/](../core-topology-correction/).

## Initial Bias

Confirmed direction:

```text
CognitionState also migrates into a store namespace.
Legacy cognition-state Continuity APIs are removed in this slice rather than
kept as compatibility wrappers.
Continuity storage backend must be abstractable.
No old cognition-state file migration is required now.
Leave migration extension points for future backend/schema migration.
Use OpenDAL itself as `ContinuityStorageBackend`: configured storage services,
router/layers, and blocking `Operator`.
Use OpenDAL's blocking API for this slice to minimize current async call-site
churn.
```

This means the generic store is not a sibling beside cognition persistence. It
becomes the durable storage model for Continuity, with current cognition state
represented as one namespace. Continuity should no longer expose
`cognition_state_snapshot()` / `replace_cognition_state()` style APIs; Cortex
or a Cortex-owned codec should read/write the cognition record through the
generic store boundary.

The first physical layout can be JSON-document storage over local fs, but
Continuity must not be architecturally tied to JSON. The backend boundary is
about physical persistence model as much as location: future records may be
binary blobs, object files, table rows, or another layout. Under this decision,
those physical choices belong behind OpenDAL services/operators, not behind a
second Beluna-defined storage backend trait.

## Guardrails

Continuity should care about:

- namespace.
- record id.
- revision.
- schema version.
- payload/body envelope.
- content representation boundary.
- deterministic save / load.
- basic durable-record shape validation.
- deletion / replacement semantics.
- storage backend abstraction.

Continuity should not care about:

- routine DSL syntax.
- routine selector meaning.
- routine activation state.
- routine execution.
- Motor scheduling.
- Neural Signal descriptor publication.

## Next Work

1. Promote durable docs for Continuity generic store and cognition persistence
   ownership.
2. Continue Issue 31 with Motor routine lifecycle / Act-Sense surface design.
3. Use the generic store API for routine source persistence when Motor lands.

The implementation plan is tracked in
[IMPLEMENTATION-PLAN.md](./IMPLEMENTATION-PLAN.md).
