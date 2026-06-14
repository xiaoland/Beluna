# Continuity Store Abstraction

> Last Updated: 2026-06-13
> Status: historical exploration; superseded by
> [OPEN-DECISIONS.md](./OPEN-DECISIONS.md) and
> [IMPLEMENTATION-PLAN.md](./IMPLEMENTATION-PLAN.md)

## Purpose

Continuity should evolve toward a generic durable store boundary instead of a
routine-specific persistence feature.

Routine source persistence is the first forcing case, but Continuity should not
be coupled to Motor routine semantics.

## Current Reality

Current Continuity owns:

- persisted `CognitionState`.
- goal forest validation.
- Efferent dispatch gate shape through `on_act`.

Current Continuity does not yet own:

- generic durable records.
- routine source definitions.
- Motor activation state.
- DSL validation.

## Design Direction

Continuity should care about durable memory properties, not routine execution.

For routine source, Continuity should care about:

- durable identity.
- namespace / collection.
- revision.
- content envelope shape.
- schema version.
- integrity and load-time validation.
- deterministic save / restore.
- migration policy.
- provenance / authoring metadata when needed.
- deletion / replacement semantics.

Continuity should not care about:

- interpreting routine DSL semantics.
- running routines.
- matching Afferent Senses.
- owning activation state.
- deciding whether a routine should emit Acts.
- publishing descriptors.

## Generic Store Record

Candidate abstraction:

```text
ContinuityRecord {
  namespace: string,
  record_id: string,
  revision: u64,
  schema_version: string,
  payload: json,
  metadata: json
}
```

For Motor routine source:

```text
namespace = "motor.routine-source"
record_id = routine_id
payload = RoutineSourceDefinition
metadata = authoring / provenance / timestamps / descriptor refs
```

This lets Continuity persist routine definitions without becoming a Motor
runtime component.

## Routine Source Definition

Routine source can be a domain payload stored by Continuity:

```text
RoutineSourceDefinition {
  routine_id: string,
  dsl: string,
  source: string,
  source_hash: string,
  declared_sense_selector: json,
  declared_act_outputs: json,
  declared_sense_outputs?: json
}
```

Continuity validates:

- required fields exist.
- ids are valid.
- schema version is supported.
- payload size limits.
- deterministic serialization constraints.

Motor validates:

- DSL parses.
- source compiles / typechecks if applicable.
- selectors are meaningful.
- declared outputs match Motor/runtime rules.
- routine can be activated.

## Store API Shape

Potential internal port:

```text
put_record(namespace, record_id, expected_revision?, payload, metadata)
get_record(namespace, record_id)
list_records(namespace)
delete_record(namespace, record_id, expected_revision?)
```

This does not need to be a public body endpoint API.

Open question:

- Should Motor request store changes via Acts, or through an internal
  Continuity port?

Observation:

- If persistence uses Acts, the pathway becomes the command bus for internal
  state changes.
- If persistence uses a port, Continuity remains an explicit internal service.
- The correct choice depends on whether Beluna wants all internal state mutation
  to be Neural Signal-visible or whether observability is sufficient.

## State Boundaries

### Persisted By Continuity

- routine source definitions.
- routine metadata needed to rehydrate Motor registry.
- maybe routine descriptor declarations.

### Owned By Motor Runtime

- active routine instances.
- activation state.
- pending child Act ids.
- retry counters / phase state.
- runtime cancellation flags.

### Published By Stem

- Neural Signal descriptor catalog entries.

### Routed By Spine

- endpoint-directed Acts.
- endpoint feedback Senses from body adapters.

## Why This Abstraction Matters

Without a generic store abstraction, routine persistence will likely leak Motor
semantics into Continuity:

- `save_routine_source`
- `load_motor_routines`
- routine-specific validation in Continuity.

That would make Continuity a Motor dependency instead of a durable memory
boundary.

The better shape is:

```text
Motor owns routine meaning.
Continuity owns durable record correctness.
Stem owns descriptor visibility.
Pathways own signal flow.
```

## Confirmed Direction

- `CognitionState` should migrate into a generic store namespace.
- Continuity should leave room for storage backends beyond the current fs JSON
  implementation.
- No old cognition-state file migration is required for this issue.
- Leave an architectural migration hook for future storage backend and data
  schema migrations.
- Prefer OpenDAL as the storage backend abstraction while keeping it out of the
  Continuity domain store contract.

## Open Decisions

1. What namespace / record id represents the singleton `CognitionState`?
2. Should Continuity support compare-and-set revisions for writes?
3. Should record payloads be arbitrary JSON or typed Rust enums per namespace?
4. Should routine source persistence be Act-mediated or internal-port mediated?
5. What observability is required for store put/get/delete/list?
6. Should the storage backend trait be sync or async?
7. Should the first implementation adopt OpenDAL immediately as the fs/memory
   backend implementation?
