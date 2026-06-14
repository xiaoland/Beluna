# Pathway Bus Implementation Notes

> Last Updated: 2026-06-14
> Status: proposed prerequisite

## Decision

Do not make `Tap` a first-class public pathway concept for Motor MVP.

For now, observability should be owned by the Afferent/Efferent pathway buses
themselves. A pathway bus is the only place that can authoritatively know:

- which source emitted a signal.
- when the signal entered the queue.
- which middleware saw it.
- which middleware accepted, rejected, transformed, or passed it through.
- whether a signal became unhandled, dropped, lost, or terminally handled.

A future `Tap`-like feature can exist as a diagnostic event subscription, but it
should observe bus lifecycle events, not become the ordinary way observability
works.

## Why Not Tap First

`Tap` mixes two different needs:

- behavioral subscription: a component receives a business signal and may act on
  it.
- observation: runtime records what happened to the signal.

Motor, Continuity, Cortex, and Spine need behavioral subscriptions. They should
be modeled as middleware/source participants.

Observability needs lifecycle records. It should not depend on a business-level
subscriber because subscriber failures, filtering bugs, or backpressure would
make observability less authoritative than the pathway it is trying to observe.

## Current Implementation Pressure

Current Afferent implementation:

```text
SenseAfferentPathway::send
  -> ingress mpsc
  -> deferral scheduler
  -> egress mpsc
  -> single SenseConsumerHandle
```

The sidecar is already a separate `broadcast` event stream, but the main signal
path is still single-consumer.

Correction:

- the current deferral scheduler should not remain pathway-owned.
- it is a Cortex consumption pre-component: a Cortex-facing admission /
  attention gate that accepts Senses from Afferent and decides when Cortex proper
  should receive them.
- Motor should not be delayed by Cortex's deferral policy unless an explicit
  route/policy says so.

Current Efferent implementation:

```text
ActProducerHandle::enqueue / dispatch_and_wait
  -> mpsc queue
  -> hard-coded runtime:
       Continuity.on_act
       Spine.on_act_final
```

This is a pipeline with one producer handle shape and no route registry.

## Proposed Runtime Shape

Each pathway becomes a small bus runtime:

```text
PathwayBus<S>
  source ports
  ingress queue
  fixed middleware sequence
  pathway runtime task
  lifecycle event emitter
```

There is no component self-registration in the MVP model. Components expose
middleware/source capabilities, but `main.rs` composition wires them into a
pathway topology and gives the fixed sequence to the pathway runtime.

For example:

```text
main.rs composition
  builds Afferent middleware sequence: Motor -> CortexAfferentAdmission
  builds Efferent middleware sequence: Motor -> Continuity -> Spine
  starts pathway runtime tasks with those routes
```

The pathway owns sequence execution order. Motor, Cortex, Continuity, and Spine
do not call `register_middleware` at runtime.

`Pathway runtime task` means a Tokio task that owns the pathway inbox and
executes the ordered middleware sequence. It is not a separate actor framework
concept.

Conceptual envelope:

```text
SignalEnvelope<S> {
  signal: S,
  pathway_signal_id,
  cycle_id?,
  seq_no
}
```

Source ports introduce new signals:

```text
AfferentTx.emit_sense(sense)
AfferentTx.emit_sense_and_wait(sense)
EfferentTx.emit_act(act)
EfferentTx.emit_act_and_wait(act)
```

Middleware participates in routing:

```text
middleware.handle(envelope) -> Accepted | Rejected | Continue(...)
```

Cross-pathway effects use source ports, not `Continue`.

## Tokio Shape

Minimum implementation shape:

```rust
pub struct AfferentTx {
    emitter_id: Option<EmitterId>,
    tx: tokio::sync::mpsc::Sender<AfferentBusCommand>,
}

pub struct EfferentTx {
    emitter_id: Option<EmitterId>,
    tx: tokio::sync::mpsc::Sender<EfferentBusCommand>,
}

enum PathwayBusCommand<S> {
    Emit(SignalEnvelope<S>),
    Shutdown,
}

struct PathwayMiddlewareSequence<S> {
    middleware: Vec<MiddlewareSlot<S>>,
}
```

`emitter_id` is optional internal implementation metadata bound when a Tx handle
is created. It can help owner-log / debugging attribute queue ingress to a
topological emitter, but it is not part of the public source-port API, not part
of `SignalEnvelope`, and not visible to middleware.

Do not model `PathwaySource` / `SourceContext` as an emission argument. That
would invite components to leak internal domain metadata into pathway transport
metadata.

The bus runtime task owns:

- bounded ingress `mpsc::Receiver`.
- bus-owned monotonic sequence allocation.
- fixed middleware sequence.
- pathway lifecycle event emission.
- optional source response completion for request/reply style emissions.

Current `ActProducerHandle::dispatch_and_wait` is not a separate pathway
component. It is a convenience API over Efferent emission:

```text
emit Act with response channel
  -> pathway middleware sequence reaches terminal outcome
  -> complete response with ActDispatchResult
```

In the bus model, this can become `EfferentTx.emit_act_and_wait(...)` or a
similar request/reply method on the source port.

Afferent can expose the symmetric shape:

```text
emit Sense with response channel
  -> pathway middleware sequence reaches terminal outcome
  -> complete response with AfferentDispatchResult
```

The MVP does not have to use `AfferentTx.emit_sense_and_wait(...)`, but designing
the source port symmetrically keeps Afferent terminal outcomes explicit instead
of treating Sense ingress as fire-and-forget forever.

The first implementation can stay single-route FIFO:

```text
bounded mpsc ingress
  -> one Tokio runtime task
  -> sequential ordered middleware calls per signal
```

This preserves deterministic ordering. Route parallelism can be introduced later
only for independent signals if evidence shows the bus is the bottleneck.

## Routing Flow

For one input signal:

1. A source port wraps it in a `SignalEnvelope`.
2. The bus emits an admission / enqueue lifecycle event.
3. The runtime calls each middleware in the fixed sequence.
4. Each middleware decision is recorded by the bus.
5. `Accepted` or `Rejected` stops routing for the current signal.
6. `Continue(original)` passes the same signal to the next middleware.
7. `Continue(transformed)` passes replacement same-type signals to the remaining
   middleware sequence.
8. If no middleware terminally handles the signal, the bus applies its
    unhandled policy.

Recommendation:

- `Continue(transformed)` should continue from the next middleware, not restart
  from the beginning.
- The pathway does not prevent Sense -> Act -> Sense loops. If a routine causes
  an infinite reflex loop, that is a routine/Cortex correction problem, not an
  Afferent/Efferent responsibility.

## Efferent Bus

Current `ActProducerHandle` evolves into `EfferentTx`.

The hard-coded runtime in `spawn_efferent_runtime` becomes an Efferent route
runtime task with a composed ordered route:

```text
Motor -> Continuity -> Spine
```

Examples:

```text
motor.routine.activate
  Motor: Accepted

continuity.store.put
  Motor: Continue(original)
  Continuity: Accepted

endpoint act
  Motor: Continue(original)
  Continuity: Continue(original)
  Spine: Accepted | Rejected | Lost
```

For Efferent, terminal middleware outcomes map back into the existing dispatch
result surface:

- `Accepted` -> acknowledged receipt/result.
- `Rejected` -> `ActDispatchResult::Rejected`.
- internal pathway/runtime failure -> `ActDispatchResult::Lost`.
- no matching terminal middleware -> `ActDispatchResult::Rejected` with
  `route_not_found`.

## Afferent Bus

Current `SenseAfferentPathway` evolves into `AfferentBus`.

The bus owns transport-level concerns:

- source ports.
- bounded queue / backpressure.
- route order.
- same-type middleware decisions.
- lifecycle records for pathway routing.

The bus does not own Cortex deferral rules, release behavior, or deferred Sense
storage.

Deferral becomes part of Cortex's Afferent-facing admission component:

```text
CortexAfferentAdmission
  receives Sense as Afferent middleware
  either admits it to Cortex's inbox now
  or accepts ownership into a Cortex-side deferred buffer
  later releases it into Cortex proper
```

From the bus perspective, `CortexAfferentAdmission` can terminally `Accept` a
Sense. Deferred / released / evicted is Cortex-side lifecycle, not Afferent
transport lifecycle.

Likely middleware order:

```text
Motor -> CortexAfferentAdmission
```

Examples:

```text
routine-triggering sense
  Motor: Accepted or Continue(original)
  Motor emits Acts through EfferentTx when a routine fires.

ordinary sense
  Motor: Continue(original)
  CortexAfferentAdmission: Accepted
```

Open policy:

- Whether Motor consumes a routine-triggering Sense or lets Cortex also receive
  it should be decided per routine / descriptor class, not hard-coded globally.

## Observability Minimum

The bus should emit canonical lifecycle events. Component-local logs remain
useful, but they are secondary.

Minimum pathway-owned events:

- source emit / enqueue.
- route start.
- middleware decision with component id, decision, output count, and latency.
- terminal accepted / rejected / lost / unhandled.

Cortex admission emits its own deferred / released / evicted events because that
state is no longer owned by Afferent Pathway.

This replaces a public `Tap` requirement for MVP while still giving Agent Task
Tests enough evidence to attribute Motor's effect.

## Open Decisions

1. What is the exact Afferent terminal result vocabulary?
2. Should `Accepted` carry a typed receipt, or is lifecycle metadata enough for
   MVP?
3. What should `AfferentTx.emit_sense_and_wait(...)` return when Cortex
   admission accepts a Sense into a deferred buffer rather than Cortex proper?
   Decision: ordinary accepted; deferral is Cortex-internal.
4. Should Cortex admission be represented as one middleware slot or as a nested
   `CortexAfferentAdmission -> CortexIngest` internal pair?
