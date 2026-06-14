# Routine State Model

> Last Updated: 2026-06-13
> Status: proposed decision

## Problem

Under the corrected Motor model, active routines are called by Motor when
Afferent Senses pass through Motor's middleware position:

```text
Afferent Sense -> Motor -> active routine -> Efferent Acts
```

The open question is whether a routine should be stateless or stateful.

## Recommendation

Use explicit activation state.

The routine source should be pure from the DSL/runtime point of view, but an
active routine instance may have state owned and stored by Motor.

Recommended shape:

```text
routine(state, sense) -> { state, acts, senses?, status? }
```

For simple routines, `state` can be `null` and the function degenerates to:

```text
routine(sense) -> { acts }
```

This gives Motor enough power for multi-step mechanical behavior without hiding
mutable memory inside the routine runtime.

## State Layers

### 1. Routine Definition State

Persistent learned knowledge:

- routine id
- DSL source
- declared input Sense selector
- declared output Act shape or allowed Act descriptors
- optional produced Sense descriptors
- version / owner metadata

Owner:

- definition authored by Cortex.
- persistence requested through Continuity by Act.
- registry owned by Motor.

### 2. Activation State

Runtime state for an active routine:

- activation id
- current explicit routine state value
- scope
- selector bindings
- pending child Act ids
- retry counters / phase markers when the routine returns them
- cancellation / termination flag
- observability lineage

Owner:

- Motor.

Persistence:

- non-persisted for MVP unless later evidence requires restart recovery.

### 3. Hidden DSL Runtime State

Examples:

- mutable globals inside the script
- retained closure state
- host-object mutation not represented in returned state

Recommendation:

- disallow for MVP.

Reason:

- hard to observe.
- hard to test.
- hard to reset on termination.
- hard to attribute failures.
- makes routine behavior depend on invisible interpreter lifetime.

## Why Not Pure Stateless Only

Pure `routine(sense) -> acts` is attractive, but it is too weak for realistic
Motor behavior.

Examples that need state:

- retry limit after repeated validation failure Senses.
- phase tracking across write, render, validate, and repair.
- avoiding duplicate responses to repeated Senses.
- correlating endpoint feedback to routine-emitted child Acts.
- terminating after completion.

All of these can be encoded outside the routine only if Motor becomes a
hard-coded workflow engine, which defeats the purpose of Cortex-authored
routines.

## Why Not Hidden Stateful Routines

Hidden mutable routine state is powerful but creates the wrong ownership.

If the script runtime owns hidden state, then Motor cannot easily:

- inspect current phase.
- include state in observability.
- terminate cleanly.
- replay a case in Agent Task Tests.
- decide whether state is safe to discard on restart.
- compare Cortex-only vs Cortex-plus-Motor behavior.

The routine should decide the next state, but Motor should own the storage.

## Function Contract

Candidate MVP contract:

```text
fn on_sense(state, sense) -> RoutineReaction
```

Where:

```text
RoutineReaction {
  state: json,
  acts: Vec<Act>,
  senses?: Vec<Sense>,
  status?: "active" | "completed" | "failed" | "terminated"
}
```

Interpretation:

- `state` replaces the previous activation state.
- `acts` are emitted to the Efferent Pathway by Motor.
- `senses` are optional routine-produced feedback into the Afferent Pathway.
- `status` tells Motor whether the activation remains active.

## Boundary With Lifecycle Acts

Lifecycle Acts such as create, delete, activate, and terminate still exist.

They are not a separate "control plane" model and should not be described as
routine execution.

They are simply Efferent Acts that Motor's Efferent middleware can handle to
mutate routine definitions or activation state.

The routine call contract remains afferent-driven:

```text
Sense + state -> state + Acts
```

## Open Decisions

1. Is activation state represented as raw JSON, typed DSL values, or a restricted
   Motor state object?
2. Should state be persisted by Continuity in a later version, or explicitly
   runtime-only forever?
3. Does `terminate-routine` discard activation state immediately or allow a
   final routine-produced Sense?
4. Should `status = completed` automatically deactivate the routine?
5. Can a routine have multiple simultaneous activations with different state?
