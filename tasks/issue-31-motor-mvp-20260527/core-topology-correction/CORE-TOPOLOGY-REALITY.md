# Core Topology Reality

> Last Updated: 2026-06-13
> Status: grounding note

## Purpose

This note corrects the Motor discussion by grounding it in the current Beluna
Core runtime topology.

Motor design should not be inferred from abstract pipeline metaphors alone. It
must fit the authority boundaries and execution paths that currently exist.

## Startup Composition

`core/src/main.rs` composes the runtime as:

```text
SenseAfferentPathway::new_handles
StemPhysicalStateStore
Spine
ContinuityEngine
new_efferent_pathway
Cortex
StemTickRuntime
spawn_efferent_runtime
CortexRuntime
```

Important wiring:

- Spine receives `afferent_ingress` and `stem_control`.
- Cortex receives `afferent_consumer`, `physical_state_reader`, Continuity, and
  `ActProducerHandle`.
- Efferent runtime receives the efferent queue receiver, Continuity, and Spine.
- Stem physical state is read through `PhysicalStateReadPort`.

## Afferent Reality

Current Afferent Pathway:

```text
body endpoint / internal producer
  -> SenseAfferentPathway ingress
  -> deferral scheduler
  -> SenseConsumerHandle
  -> CortexRuntime pending_senses
  -> Cortex on admitted tick
```

Current properties:

- Afferent consumer is Cortex runtime.
- `SenseAfferentPathway` supports deferral controls.
- `AfferentSidecarEvent` exists, but it reports rule/deferral events; it is not a
  general Sense middleware API.
- Cortex drains pending Senses on tick and passes them to `Cortex::cortex`.

Implication for Motor:

- Saying "Motor is Afferent middleware" is not currently backed by a general
  middleware chain.
- Adding Motor requires an explicit topology change:
  - insert Motor before Cortex as a Sense stream stage, or
  - make Afferent Pathway support middleware/fanout semantics, or
  - give Motor a separate tap and define whether it can transform/consume Senses.

This is a real design decision, not a naming detail.

## Efferent Reality

Current Efferent Pathway:

```text
Cortex Primary dynamic act tool
  -> Cortex::dispatch_act
  -> ActProducerHandle::dispatch_and_wait
  -> Efferent queue
  -> Continuity.on_act
  -> Spine.on_act_final
  -> endpoint adapter
```

Current properties:

- Act tools are generated from `PhysicalState.ns_descriptor`.
- Cortex dispatches Acts during the LLM tool-call turn.
- `dispatch_and_wait` currently waits briefly for a terminal result.
- Continuity currently returns `DispatchDecision::Continue`.
- Spine owns endpoint routing and terminal dispatch outcome generation.
- Spine emits dispatch failure Senses through the Afferent Pathway.

Implication for Motor:

- Current durable docs say middleware order is `Continuity.on_act ->
  Spine.on_act_final`.
- Adding Motor to Efferent Pathway changes a cross-unit technical contract.
- Motor cannot be assumed to be a harmless pre-stage without revising the
  Efferent pathway authority model.

## Physical State And Descriptor Reality

`StemPhysicalStateStore` owns:

- cycle id projection.
- ledger snapshot.
- Neural Signal descriptor catalog.
- proprioception.

Spine currently registers body endpoint descriptors by calling Stem control:

```text
Spine endpoint registration -> StemControlPort.apply_neural_signal_descriptor_patch
```

Cortex receives descriptor snapshots through `PhysicalState`.

Implication for Motor:

- Cortex discovers actions through `PhysicalState.ns_descriptor`.
- If Motor exposes lifecycle Acts, those descriptors must be published into Stem
  physical state.
- Motor should not be modeled as a normal Spine body endpoint unless we
  intentionally want a fake adapter/body route.
- A cleaner model is likely an internal descriptor publisher authorized by Stem
  control.

## Continuity Reality

Current Continuity owns:

- persisted `CognitionState`.
- goal forest validation.
- dispatch gate shape through `on_act`.

Current Continuity does not own:

- arbitrary internal component persistence.
- routine registry persistence.
- routine activation state.

Implication for Motor:

- "Motor persists routines through Continuity" is not currently true.
- It may become a design decision, but it expands Continuity's authority beyond
  current cognition-state persistence.
- If routine definitions are learned cognition-adjacent state, we must decide
  whether they belong in Continuity, Motor storage, or a new persistence boundary.

## Cortex Reality

Cortex is not a pure function that returns a final `Vec<Act>`.

Cortex Primary:

- receives Senses, proprioception, goal forest, and descriptor-derived act tools.
- runs an LLM turn loop.
- dispatches Acts through dynamic tools during the turn.
- can expand Senses with helpers or sub-agent helpers.
- must call `break-primary-phase` to finish the tick.
- Attention and Cleanup run after Primary when Primary breaks.

Implication for Motor:

- Motor changes Cortex's action space by changing descriptors.
- Motor changes the feedback loop by emitting Acts and/or Senses.
- Motor should be evaluated as a runtime participant in the sense/act loop, not
  only as an Act transformer.

## Corrected Motor Placement Question

The real Motor topology question is:

```text
Where can a non-LLM, routine-hosting runtime participate in the existing
Sense -> Cortex -> Act loop without violating Stem, Continuity, or Spine authority?
```

Candidate insertion points:

1. Afferent stage before Cortex:
   - Motor sees Senses before Cortex consumes the tick batch.
   - Requires defining observe vs transform vs consume semantics.

2. Efferent stage before Continuity / Spine:
   - Motor sees Acts emitted by Cortex or by routines.
   - Requires revising current `Continuity -> Spine` deterministic order.

3. Descriptor publisher:
   - Motor publishes its own lifecycle Act descriptors through Stem control.
   - Must not pretend to be a Spine body endpoint unless deliberately chosen.

4. Routine storage owner:
   - unresolved.
   - Continuity is not automatically the right owner under current code.

## Immediate Corrections To Prior Packet

Supersede:

- Motor as a simple `Act -> Vec<Act>` middleware response model.
- assuming Afferent Pathway already has general middleware semantics.
- assuming Continuity already owns routine persistence.
- assuming Motor descriptors should naturally be registered like Spine body
  endpoint descriptors.

Keep:

- Motor is outside Cortex.
- Motor must remain visible through Neural Signals and observability.
- Cortex should discover Motor affordances through `PhysicalState.ns_descriptor`.
- Motor exists to reduce Cortex burden in the agent loop, not to become another
  endpoint adapter.
