# Solidified Core Topology Contract

> Last Updated: 2026-06-14
> Status: confirmed for implementation

## Topology

Afferent and Efferent Pathways are source-port + fixed middleware sequence
runtimes.

```text
Afferent:
  multiple AfferentTx
    -> bounded mpsc
    -> fixed middleware sequence: Motor -> CortexAfferentAdmission

Efferent:
  multiple EfferentTx
    -> bounded mpsc
    -> fixed middleware sequence: Motor -> Continuity -> Spine
```

For the first implementation, Motor can be represented by a pass-through
placeholder middleware until the Motor component exists.

## Source Ports

The public source-port APIs do not accept source metadata.

```text
AfferentTx.emit_sense(sense)
AfferentTx.emit_sense_and_wait(sense)
EfferentTx.emit_act(act)
EfferentTx.emit_act_and_wait(act)
```

No `PathwaySource` / `SourceContext` is part of the pathway contract.

An implementation may bind an internal optional `EmitterId` to a Tx handle for
owner-log/debug attribution only. That id is not part of `SignalEnvelope`, is
not visible to middleware, and cannot carry participant-owned domain metadata.

## Signal Envelope

The pathway runtime may wrap signals in an internal envelope:

```text
SignalEnvelope<S> {
  signal: S,
  pathway_signal_id,
  cycle_id?,
  seq_no
}
```

The envelope is transport/runtime state. It is not a place for component
internals such as Motor routine ids, Cortex thoughts, or Spine adapter state.

## Middleware Decision

Middleware returns:

```text
Accepted
Rejected(reason_code, message?)
Continue(original)
Continue(transformed)
```

Rules:

- First `Accepted` or `Rejected` stops the current signal.
- `Continue` is same-pathway / same-signal-type only.
- `Continue(transformed)` continues from the next middleware in the fixed
  sequence.
- Transform cardinality is not a pathway-level constraint.
- Cross-pathway effects use source ports, not `Continue`.

## Afferent Semantics

Afferent Pathway does not own Cortex deferral.

`CortexAfferentAdmission` is the Afferent-facing Cortex middleware. If it puts a
Sense into a deferred buffer, the Afferent terminal outcome is still accepted
because the Sense has been accepted by Cortex's admission boundary.

The Afferent Pathway does not prevent Sense -> Act -> Sense reflex loops. Bad
routine loops are Motor/Cortex correction problems.

## Efferent Semantics

Efferent Pathway owns terminal Act dispatch result mapping:

```text
Accepted -> ActDispatchResult::Acknowledged
Rejected -> ActDispatchResult::Rejected
runtime failure -> ActDispatchResult::Lost
unhandled -> ActDispatchResult::Rejected(route_not_found)
```

Existing Cortex dynamic act tool behavior should remain request/reply through
`EfferentTx.emit_act_and_wait`.

## Composition

Middleware sequences are assembled directly in `main.rs` for this correction.

Do not introduce:

- dynamic middleware self-registration.
- descriptor-selected routes.
- a global topology registry.
- public tap subscription as a routing concept.
- pathway-level loop budget / lineage machinery.

## Observability

Pathways own minimal lifecycle emission:

- enqueue.
- middleware decision.
- terminal result.
- queue closed / lost.

Tap is not a first-class pathway concept for the MVP. If monitor-facing
subscriptions are needed later, they should observe lifecycle events rather than
participate in signal routing.
