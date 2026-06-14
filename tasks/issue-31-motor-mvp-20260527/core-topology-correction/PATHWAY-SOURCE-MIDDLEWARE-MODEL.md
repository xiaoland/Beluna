# Pathway Source + Middleware Model

> Last Updated: 2026-06-14
> Status: proposed prerequisite

## Purpose

Motor requires a more general pathway topology before it can be implemented
cleanly.

The prerequisite correction is:

```text
Pathway = bus with multiple source ports and receiver subscriptions
```

This applies to both Afferent and Efferent Pathways.

More precisely:

```text
Component = pathway bus participant with one or more ports/subscriptions
```

Participant capabilities:

- `SourcePort` / tx.
- `ReceiverPort` / rx.
- middleware subscription.
- tap subscription.

A component may hold multiple capabilities on one or both pathways.

Terminal behavior is not a fixed component role. It is a middleware decision:

```text
Accepted | Rejected | Continue(original/transformed)
```

See [PATHWAY-MIDDLEWARE-CONTRACT.md](./PATHWAY-MIDDLEWARE-CONTRACT.md).
See [PATHWAY-BUS-MODEL.md](./PATHWAY-BUS-MODEL.md) for the bus-first model.

## Why This Is Needed

Current Core has pathway-like code, but not a general pathway bus model:

- Afferent is currently a Sense ingress queue, deferral scheduler, and Cortex
  consumer.
- Efferent is currently a queue whose runtime calls Continuity and then Spine.

Motor needs to participate in both directions:

- On Afferent, Motor must see selected Senses and may produce Acts.
- On Efferent, Motor must see selected Acts, including lifecycle Acts addressed
  to `motor`, and may mutate routine registry / activation state.

This should not be modeled as Motor being a special Cortex helper or a fake
Spine endpoint.

## Core Abstraction

### Source

A source introduces signals into a pathway.

In code terms, a source is exposed as a source port / emission port, not as a
middleware `Continue` result.

Examples:

- body endpoints source Senses into Afferent.
- internal runtime failures source Senses into Afferent.
- Cortex sources Acts into Efferent.
- Motor routines may source Acts into Efferent.
- Motor lifecycle handling may source Senses into Afferent.

Source identity should be observable, but source identity is not routing
authority by itself.

Important rule:

- Source emission can introduce a different signal type into another pathway.
- Middleware `Continue` cannot. `Continue` is same-pathway / same-signal-type.

Example:

```text
Afferent middleware receives Sense
  -> Continue(original/transformed Sense) for the current Afferent flow
  -> optionally emit Act through EfferentSourcePort
```

### Middleware

A middleware is a receiver subscription that participates in routing.

Possible operations:

- observe a signal.
- transform a signal.
- consume / stop a signal.
- emit additional signals to the same or opposite pathway through source ports.
- return an error that the pathway converts into a terminal outcome or failure
  Sense.

Middleware order is part of the pathway contract for middleware subscriptions.

### Tap

A tap is a receiver subscription that observes matching signals without changing
routing.

Examples:

- observability.
- monitoring projections.
- diagnostics.

## Component Role Mapping

### Cortex

Cortex can be modeled as a pathway participant with roles:

- Afferent middleware receiver that usually accepts Senses into cognition.
- Efferent source port.

This is cleaner than treating Cortex as external to pathways.

But Cortex should not be flattened into "just another middleware" without
qualification, because it owns cognition transformation and tick-admitted LLM
execution.

### Spine

Spine can be modeled as:

- Efferent middleware receiver that usually accepts/rejects endpoint-directed Acts.
- Afferent source port for dispatch failure Senses and endpoint feedback routed from
  adapters.
- descriptor/proprioception publisher through Stem control during endpoint
  registration.

This is cleaner than keeping Spine outside the pathway model.

But Spine should remain the terminal dispatch authority for endpoint-directed
Acts.

### Continuity

Continuity can be modeled as:

- Efferent middleware receiver for dispatch gating / guardrails / store Acts.
- Store service for durable cognition-adjacent state.

The store role should not be over-coupled to routine storage. See
[CONTINUITY-STORE-ABSTRACTION.md](./CONTINUITY-STORE-ABSTRACTION.md).

### Motor

Motor can be modeled as:

- Afferent middleware receiver.
- Efferent middleware receiver.
- Efferent source port when active routines emit Acts.
- Afferent source port when lifecycle/routine status produces Senses.
- descriptor publisher through Stem control for Motor affordances.

## Afferent Pathway Target Shape

Current:

```text
Sense producers -> SenseAfferentPathway -> CortexRuntime
```

Target:

```text
AfferentBus<Sense>
  tx: endpoint / internal / Motor sources
  rx: Motor, Cortex, observability, future subscribers
```

Potential middleware:

- deferral / gating middleware.
- Motor middleware.
- observability middleware.

Open design:

- Afferent middleware may consume or transform Senses before Cortex.
- whether Motor should be observe-only for MVP.
- whether middleware can emit Acts during Afferent handling.
- how to prevent infinite Sense/Act reflex loops.

If Afferent middleware consumes or transforms Senses, the pathway must preserve
world-model coherence through:

- lineage from original Sense to emitted/forwarded Senses.
- explicit consumed outcome.
- observability event for consume/transform.
- clear rule for whether Cortex sees original, transformed, or no Sense.

## Efferent Pathway Target Shape

Current:

```text
Act sources -> Efferent queue -> Continuity -> Spine
```

Target:

```text
EfferentBus<Act>
  tx: Cortex / Motor / future sources
  rx: Motor, Continuity, Spine, observability, future subscribers
```

Potential middleware:

- Motor.
- Continuity.
- observability / lineage middleware.

Open design:

- exact middleware order.
- whether Motor must run before Continuity.
- whether routine-produced Acts enter at the beginning of Efferent or after
  Motor.
- how terminal outcomes are represented when a middleware consumes an Act.

## Authority Boundaries

Stem should remain the likely owner of pathway runtime structure because it
already owns:

- tick authority.
- Afferent and Efferent pathway modules.
- PhysicalState / descriptor catalog.

Continuity should not become the pathway runtime owner.

Spine should remain the endpoint dispatch authority, not the generic Efferent
pipeline owner.

Cortex should remain a pathway participant/source/middleware receiver, not the
pathway runtime owner.

## Prerequisite Outcome

Before Motor implementation, Core should have explicit contracts for:

- Afferent bus tx/source port API.
- Afferent bus rx/subscription API.
- Afferent middleware decision contract.
- Efferent bus tx/source port API.
- Efferent bus rx/subscription API.
- Efferent middleware decision contract.
- middleware ordering.
- tap subscriber visibility.
- signal lineage and source attribution.
- error and consumed-signal outcomes.

## Motor Fit After This Prerequisite

Once the bus model exists:

- Motor can subscribe as Afferent middleware.
- Motor can subscribe as Efferent middleware.
- Cortex can manage Motor through descriptor-visible lifecycle Acts.
- Motor routines can emit Acts through Efferent tx.
- Motor can emit lifecycle / routine Senses through Afferent tx.

This makes Motor a normal participant in Core topology instead of a special case.
