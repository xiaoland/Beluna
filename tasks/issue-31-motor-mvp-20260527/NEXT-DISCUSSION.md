# Next Discussion Agenda

> Last Updated: 2026-06-14

## Current Direction

Before Motor implementation, Issue 31 needs two prerequisite subtasks:

1. Core topology correction:
   - model Afferent and Efferent Pathways as buses.
   - each bus supports multiple tx/source ports and fixed middleware
     participants.
   - middleware sequence order is assembled in `main.rs`.
   - terminal behavior is a middleware decision.

2. Continuity persistence expansion:
   - evolve Continuity toward a generic durable store.
   - use routine source as the first forcing case, but do not couple Continuity
     to Motor routine semantics.

Relevant notes:

- [CORE-TOPOLOGY-REALITY.md](./CORE-TOPOLOGY-REALITY.md)
- [PATHWAY-BUS-MODEL.md](./PATHWAY-BUS-MODEL.md)
- [PATHWAY-BUS-IMPLEMENTATION.md](./PATHWAY-BUS-IMPLEMENTATION.md)
- [PATHWAY-SOURCE-MIDDLEWARE-MODEL.md](./PATHWAY-SOURCE-MIDDLEWARE-MODEL.md)
- [PATHWAY-MIDDLEWARE-CONTRACT.md](./PATHWAY-MIDDLEWARE-CONTRACT.md)
- [continuity-store-abstraction/CONTINUITY-STORE-ABSTRACTION.md](./continuity-store-abstraction/CONTINUITY-STORE-ABSTRACTION.md)

## Current Execution Topic

Core topology correction and Continuity generic store prerequisites are now
implemented in Core.

Relevant result files:

- [core-topology-correction/IMPLEMENTATION-RESULT.md](./core-topology-correction/IMPLEMENTATION-RESULT.md)
- [continuity-store-abstraction/IMPLEMENTATION-RESULT.md](./continuity-store-abstraction/IMPLEMENTATION-RESULT.md)

## Recommended Next Topic

Motor routine lifecycle / Act-Sense surface design.

Reason:

- Motor needs to receive Senses and emit Acts without becoming a private Cortex
  helper.
- Cortex, Motor, Continuity, Spine, and observability all need receiver roles.
- Cortex and Motor can both be Efferent tx.
- endpoints, Spine, Motor, and internal runtime components can all be Afferent
  tx.

## Bus Contract

Proposed shape:

```text
PathwayBus<S>
  tx: multiple SourcePort<S>
  middlewares: fixed sequence
  policy: middleware decisions + unhandled behavior
```

Questions:

- Should tx/rx be explicit types in code, e.g. `AfferentTx`, `AfferentRx`,
  `EfferentTx`, `EfferentRx`?
- Should diagnostic lifecycle subscriptions be exposed to monitor code, or is
  owner-log emission enough for MVP?

## Subscription Modes

Current decision:

- Do not make `Tap` a first-class public pathway concept for Motor MVP.
- Observability should be bus-owned lifecycle emission.
- Behavioral participants use middleware/source ports.
- Components do not self-register middleware at runtime; Core/Stem composition
  builds ordered routes and gives them to pathway runtime tasks.
- Ordered middleware sequences are assembled directly in `main.rs`.
- Afferent deferral is not pathway-owned; it is Cortex's Afferent-facing
  admission / attention gate.
- There is no descriptor-selected routing in Afferent/Efferent Pathway MVP.
- Afferent/Efferent do not own reflex-loop prevention; bad routine loops are
  Cortex/routine-author correction problems.

Questions:

- Is owner-log emission enough for MVP, or does monitor need an in-process
  lifecycle event subscription?
- Should lifecycle event subscription be considered diagnostic infrastructure
  rather than a pathway routing mode?
- Should `main.rs` keep sequence assembly inline, or use small helper functions
  without introducing a topology registry?

## Middleware Decision Contract

Invariant:

- `Continue` is same-pathway / same-signal-type only.

Proposed decision set:

```text
Accepted
Rejected
Continue(original)
Continue(transformed)
```

Questions:

- Should `Accepted` carry a typed receipt?
- Should `Rejected` reuse existing `ActDispatchResult::Rejected` semantics on
  Efferent?
- What is the Afferent equivalent of rejected / accepted?
- Should `Continue(transformed)` be represented as `Vec<S>` or a named
  replacement collection type?

## Motor Under Bus Model

Motor uses:

- Afferent middleware subscription to receive Senses.
- Efferent tx to emit routine-produced Acts.
- Efferent middleware subscription to handle Motor-owned lifecycle Acts.
- Afferent tx to emit lifecycle/routine status Senses.

Questions:

- Does Motor consume routine-triggering Senses or continue them to Cortex?
- What source context must accompany Motor-emitted Acts?
- Do Motor-produced Acts re-enter the Efferent bus from the start of routing?

## Continuity Generic Store

Question:

What does Continuity care about when storing routine source?

Candidate answer:

- namespace.
- record id.
- revision.
- schema version.
- payload envelope.
- deterministic save/restore.
- migration/load-time validation.
- provenance metadata if needed.

What Continuity should not care about:

- routine DSL semantics.
- Sense selector meaning.
- routine activation state.
- routine execution.
- Motor scheduling.

Questions:

- Is generic `ContinuityRecord` the right shape?
- Should routine source be stored as `namespace = motor.routine-source`?
- Should Motor request store changes through Acts or an internal Continuity port?
- Is activation state explicitly excluded from Continuity MVP?

## Durable Doc Boundary

These prerequisites touch durable contracts.

Likely durable owners:

- `docs/20-product-tdd/coordination-model.md`
- `docs/20-product-tdd/system-state-and-authority.md`
- `docs/30-unit-tdd/core/design.md`
- `docs/30-unit-tdd/core/data-and-state.md`
- `docs/30-unit-tdd/core/interfaces.md`

Before implementation, restate the intended durable-doc changes for
confirmation.
