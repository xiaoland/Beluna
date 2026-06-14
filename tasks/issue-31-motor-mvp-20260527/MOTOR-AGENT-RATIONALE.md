# Motor Agent Rationale

> Last Updated: 2026-06-13
> Status: grounding note

## Purpose

This note reframes Motor from Beluna's actual runtime premise:

- Beluna is an agent.
- Cortex is currently implemented through LLM-driven cognition.
- Beluna acts in the world through Neural Signal descriptors, Senses, and Acts.

Motor should be justified by this reality, not by a metaphysical analogy.

## Agent Premise

Beluna's core loop is:

```text
Senses -> cognition -> Acts -> world feedback -> Senses
```

The product premise is not "chatbot with tools". It is an embodied agent:

- Senses enter through body endpoints and internal runtime signals.
- Physical state exposes available affordances and runtime context.
- Cortex chooses how to understand Senses and emit Acts.
- Acts affect the world through endpoint dispatch.
- World feedback returns as Senses.

Motor must improve this loop.

## Cortex Reality

Cortex Primary is LLM-based.

This gives Beluna flexible reasoning, but also creates hard runtime constraints:

- expensive per reasoning step.
- latency-bound by model calls and tool-call turns.
- nondeterministic in repeated mechanical workflows.
- sensitive to context rot and prompt/history shape.
- weak as a long-running deterministic scheduler.
- weak at preserving low-level procedural invariants across many edits.
- requires descriptor-shaped affordances to know what actions are possible.
- must spend cognitive budget deciding even simple repeated reactions unless
  another runtime component owns them.

Cortex should remain responsible for:

- interpreting user intent.
- setting goals and constraints.
- choosing or authoring routines.
- deciding when a mechanical pattern should exist or stop.
- integrating ambiguous, novel, or semantic feedback.

Cortex should not be required to re-think every small procedural reaction.

## Why Motor Exists

Motor exists because an LLM Cortex is a poor substrate for mechanical
closed-loop control.

Motor should handle work that is:

- procedural.
- repetitive.
- local.
- descriptor-bound.
- validated by concrete Senses.
- easy to specify as a routine once, but wasteful for Cortex to re-plan every
  time.

Examples:

- after a file write Sense, run a validator Act.
- after a validator failure Sense, apply a bounded structural repair Act.
- after repeated failure, stop and emit a failure Sense.
- preserve artifact structural invariants while Cortex focuses on intent and
  content.

This is not because Motor is "the cerebellum" in an abstract sense.

It is because Beluna needs a non-LLM runtime layer that can turn selected Senses
into selected Acts with better determinism, latency, cost, and observability than
an LLM turn.

## Motor's Product Role

Motor should be a learned reflex host.

Its value is not raw capability. Cortex can already emit Acts.

Motor's value is:

- reducing Cortex cognitive load.
- improving repeatability of mechanical action loops.
- shortening sense-to-act latency for routine reactions.
- making procedural behavior observable and testable.
- keeping mechanical state explicit instead of buried in LLM context.
- letting Cortex author new reflexes when it notices a repeated pattern.

Motor is therefore not:

- a replacement for Cortex.
- a generic workflow engine independent from Beluna's Neural Signals.
- a hidden helper that bypasses descriptors and observability.
- an endpoint adapter owned by Spine.

## Correct Routine Meaning

A routine should be understood as:

```text
a Cortex-authored, Motor-hosted transition over Senses and explicit state,
whose outputs are Acts and optional Senses.
```

Candidate shape:

```text
routine(state, sense) -> { state, acts, senses?, status? }
```

This fits the agent premise:

- input is feedback from the world.
- output is action back into the world.
- state is explicit and inspectable.
- Cortex can create/activate/terminate routines through Neural Signals.

## Why Explicit State

If routines are purely stateless:

- they cannot track phases.
- they cannot enforce retry limits.
- they cannot suppress duplicate reactions.
- they cannot correlate feedback to prior emitted Acts.
- Motor becomes a hard-coded workflow engine to compensate.

If routines hide mutable state inside the DSL runtime:

- Motor cannot observe or explain behavior.
- tests cannot replay state transitions cleanly.
- termination becomes ambiguous.
- restart semantics become accidental.
- failures become harder to attribute.

Therefore the preferred model is:

```text
pure routine source + Motor-owned explicit activation state
```

## Why Neural Signals Still Matter

Motor should not bypass Beluna's Neural Signal model.

Neural Signals are how Cortex sees and acts through the body:

- lifecycle Acts let Cortex manage routines.
- descriptors let Cortex discover Motor affordances.
- routine-produced Acts remain ordinary Acts into the Efferent Pathway.
- routine-produced Senses report routine status back through Afferent Pathway.
- observability can attribute behavior to routine id, activation id, triggering
  Sense id, and emitted Act ids.

Motor exists below Cortex's conscious reasoning, but it must still be visible to
the agent through the same signal model.

## MVP Acceptance Reframing

The slide / Markdown / HTML acceptance criterion is not "Motor understands edit
intent".

Cortex should understand intent.

Motor should improve the mechanical loop around artifact mutation:

- preserve structure.
- validate renderability.
- react to validator Senses.
- bound retries.
- emit structured completion/failure Senses.

The correct test is not whether Motor can replace Cortex, but whether Cortex
plus Motor beats Cortex alone on repeatable artifact-editing loops with
observable attribution.

## Design Pressure From This Rationale

This rationale implies:

1. Motor must be integrated into the actual Sense/Act pathways, not treated as a
   private Cortex helper.
2. Motor state must be explicit enough for observability and Agent Task Tests.
3. Motor routine source should be small and Cortex-authorable.
4. Motor should make Cortex's descriptor-visible affordance space richer, not
   hide behavior outside the agent model.
5. Motor persistence and descriptor publication need explicit authority
   decisions because current Core does not already provide them.
