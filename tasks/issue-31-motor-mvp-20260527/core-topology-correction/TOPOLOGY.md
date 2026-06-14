# Motor Topology Notes

> Last Updated: 2026-06-13

## Placement

Motor should be modeled as a core-internal component outside Cortex.

Peer set:

- `stem`
- `cortex`
- `motor`
- `continuity`
- `spine`
- `ledger`
- `ai_gateway`
- `body`

This means Motor is not:

- a Cortex Primary helper
- a Spine adapter
- a Continuity persistence detail

Motor is endpoint-shaped because Act/Sense routing is Neural Signal descriptor driven.

However, Motor is not a normal Spine-routed external body endpoint: it is an inner endpoint-shaped efferent middleware component that runs before Continuity and Spine.

Motor endpoint id is fixed as:

```text
motor
```

Motor lifecycle Acts live under that endpoint namespace.

The corrected routine model is not "routine-specific Act descriptor invokes
routine". The primary active routine path is:

```text
Sense -> active routine -> Acts
```

See [ROUTINE-REFLEX-MODEL.md](./ROUTINE-REFLEX-MODEL.md).

## Dispatch Pipeline

Restore act dispatch as a pipeline model for Efferent Pathway work:

```text
Cortex / Motor emitted Act -> Motor -> Continuity -> Spine
```

The important change is that the pipeline stages should be middleware-like.

Each stage can:

- inspect an Act
- claim / intercept it
- transform it
- emit zero or more Acts
- continue the original Act unchanged
- reject / stop the dispatch
- report side effects through observability and afferent feedback

## Motor As Dual Pathway Middleware

Cortex still produces Acts.

Motor does not require a special Cortex-to-Motor private channel.

On the Efferent Pathway, Motor receives Acts as a middleware stage and decides:

1. If the Act is not Motor-owned:
   - pass it through unchanged to Continuity.

2. If the Act is Motor-owned lifecycle work:
   - intercept the Act.
   - create, delete, activate, or terminate a routine.
   - request Continuity persistence through Acts when needed.
   - emit lifecycle result Senses when needed.
   - do not send the claimed original Act to Spine as if it were a body endpoint act.

This preserves a simple Cortex model:

- Cortex chooses lifecycle Acts for Motor.
- The dispatch pipeline decides how those Acts are handled.
- Motor behavior is visible as pathway middleware, not hidden as a Cortex helper.

## Continuity And Spine

Continuity and Spine remain later pipeline stages:

- Continuity observes, validates, persists, or gates act-related state.
- Spine mechanically routes endpoint-directed Acts.

The model should avoid making Continuity and Spine special-cased into one hard-coded function. They are pipeline stages with middleware semantics.

## Afferent Pathway Position

Motor is also connected to the Afferent Pathway.

Reason:

- A routine is not only a one-shot Act expander.
- A routine may continuously take over procedural action across multiple feedback steps.
- That requires Motor to observe routine-related Senses and decide whether to emit the next procedural Acts without returning control to Cortex for every step.

On the Afferent Pathway, Motor receives Senses as middleware and may call active
routines.

Working model:

```text
Sense producers -> Motor afferent middleware -> Cortex-facing afferent flow
                         |
                         v
         active routine(state, sense) -> state + Acts
```

Once active, a routine is mechanically driven by matched Senses. Motor stores
explicit activation state and emits returned Acts to the Efferent Pathway.

Open detail:

- whether Motor merely observes routine-correlated Senses or can consume / transform them before Cortex sees them.
- whether routine-produced terminal Senses should always be visible to Cortex.

## Current Code Evidence

Current code already resembles a pipeline:

- `core/src/stem/efferent_pathway.rs` exposes `ActProducerHandle` and `EfferentActEnvelope`.
- `process_efferent_dispatch` currently calls Continuity and then Spine.
- Motor can be inserted before Continuity if the dispatch path is generalized into middleware stages.
- `core/src/stem/afferent_pathway.rs` already models Sense ingress as a bounded pathway with sidecar events and a consumer handle.
- Active Motor routines can be modeled as afferent middleware callbacks that emit Acts.
- The earlier `Act -> Vec<Act>` middleware response model is no longer the Motor routine contract. See [MIDDLEWARE-RESPONSE.md](./MIDDLEWARE-RESPONSE.md) as historical design context.

## Open Topology Edges

1. Whether Motor must be allowed to emit multiple Acts concurrently or must serialize them.
2. Whether Motor claiming is based only on endpoint id `motor`, or endpoint id plus built-in lifecycle descriptor id.
3. Whether Continuity should see the original claimed lifecycle Act, the Motor-emitted Acts, or both as separate observability / persistence events.
4. Whether `DispatchDecision` should be retired in favor of `Vec<Act>` middleware responses.
5. Whether Motor can consume Afferent Senses, or should only observe and emit additional Senses.
6. Whether routine activation is global or scoped by activation id / conversation / artifact.
7. Whether explicit routine state is raw JSON, typed DSL values, or a restricted Motor state object.
