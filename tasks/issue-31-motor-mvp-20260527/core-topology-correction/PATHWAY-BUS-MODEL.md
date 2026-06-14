# Pathway Bus Model

> Last Updated: 2026-06-14
> Status: proposed prerequisite

## Core Correction

Afferent and Efferent Pathways should be modeled closer to buses than
single-receiver pipelines.

Core shape:

```text
PathwayBus<S>
  = multiple Tx source ports
  + multiple Rx participants
  + fixed middleware sequence execution
  + observability
```

Where:

- Afferent bus signal type is `Sense`.
- Efferent bus signal type is `Act`.

## Why Bus, Not Plain Chain

Current code already hints at a bus-like need:

- multiple components can produce Senses.
- Cortex is not the only possible receiver of Senses once Motor exists.
- Cortex and Motor can both produce Acts.
- Continuity, Motor, Spine, observability, and future components may all need to
  receive or handle Acts.

The pathway should therefore support:

- multiple tx.
- multiple rx.
- one globally ordered middleware sequence where ordering matters.
- fanout where observation does not consume the signal.

## Ports

### Source Port / Tx

A source port introduces a new signal into a pathway.

Conceptual shape:

```text
AfferentTx.emit_sense(sense)
AfferentTx.emit_sense_and_wait(sense)
EfferentTx.emit_act(act)
EfferentTx.emit_act_and_wait(act)
```

Source identity is not part of the public emission contract.

If runtime attribution is needed, a Tx handle may carry an internal
implementation-only `EmitterId` bound at construction time. That id is not
visible to middleware and must not carry participant-owned internals such as
Motor routine id or routine activation id.

Examples:

- body endpoint -> AfferentTx.
- Spine dispatch failure -> AfferentTx.
- Cortex -> EfferentTx.
- Motor routine -> EfferentTx.
- Motor lifecycle result -> AfferentTx.

### Middleware Sequence

Receivers do not self-register into a pathway in the MVP model.

Components expose middleware/source capabilities, and `main.rs` composition
wires those capabilities into a fixed ordered sequence. The pathway runtime
executes that sequence.

Conceptual shape:

```text
PathwayMiddlewareSequence<S> = [MiddlewareSlot<S>, ...]
```

Examples:

- Afferent sequence: Motor -> CortexAfferentAdmission.
- Efferent sequence: Motor -> Continuity -> Spine.

The ordering authority belongs to `main.rs` composition and the pathway runtime,
not to Motor, Cortex, Continuity, or Spine individually.

## Subscription Modes

For Motor MVP, the bus does not need `Tap` as a first-class public subscription
mode.

The required behavioral subscription mode is middleware.

### Middleware

Middleware receivers participate in routing.

They return:

```text
Accepted
Rejected
Continue(original)
Continue(transformed)
```

This decision affects the signal's remaining middleware sequence.

Use cases:

- Motor accepting `motor.routine.activate`.
- Continuity accepting `continuity.l1_memory.update`.
- Spine accepting endpoint-directed Acts.
- Motor consuming or transforming selected Afferent Senses.

Observation should be bus-owned lifecycle emission, not a business-level tap.
See [PATHWAY-BUS-IMPLEMENTATION.md](./PATHWAY-BUS-IMPLEMENTATION.md).

## Bus Dispatch Semantics

The bus dispatches one signal through an ordered middleware sequence.

For each middleware receiver:

- `Continue(original)` keeps the same signal moving.
- `Continue(transformed)` replaces the current same-type signal with one or more
  same-pathway signals for the remaining sequence.
- `Accepted` marks the current signal as successfully handled and stops routing
  that signal.
- `Rejected` marks the current signal as refused and stops routing that signal.

Diagnostic event subscribers may be added later, but they should observe pathway
lifecycle events rather than participate in signal routing.

## Same-Type Rule

Middleware `Continue` is same-pathway / same-signal-type only.

```text
Afferent middleware: Sense -> Continue(Sense)
Efferent middleware: Act -> Continue(Act)
```

Cross-pathway effects use source ports:

```text
Motor receives Sense from AfferentBus
  -> middleware decision for current Sense
  -> optionally emits Acts through EfferentTx
```

## Multiple Rx And Terminal Handling

"Terminal" is not a fixed receiver role.

Any middleware receiver can terminally handle a signal by returning `Accepted` or
`Rejected`.

Examples:

- Motor accepts `motor.routine.activate`.
- Continuity accepts `continuity.l1_memory.update`.
- Spine accepts endpoint-directed Acts.
- Cortex accepts Afferent Senses into cognition.

This works with multiple participants because the bus does not send to one
hard-coded terminal. The bus walks a fixed middleware sequence until one
middleware accepts/rejects it or no receiver handles it.

## Unhandled Signal Policy

The bus must define what happens when no middleware accepts/rejects a signal.

Potential policies:

- Efferent unhandled Act -> rejected with `route_not_found`.
- Afferent unhandled Sense -> dropped with observability.
- Afferent unhandled Sense -> accepted by default no-op.

This should differ by pathway.

## Decided

1. Middleware execution is one global fixed sequence per pathway, not
   descriptor-selected route lists.
2. First `Accepted` or `Rejected` stops the current signal.
3. `Continue(transformed)` continues through the remaining middleware sequence
   from the next middleware.
4. Transform cardinality is not a pathway-level constraint.

## Open Decisions

1. Should diagnostic lifecycle subscriptions be exposed to monitor code, or is
   owner-log emission enough for MVP?
2. What are the unhandled policies for Afferent and Efferent buses?
3. Should internal `EmitterId` be recorded in observability, or only middleware
   decisions and signal ids?
