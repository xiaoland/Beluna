# Prerequisite Subtasks

> Last Updated: 2026-06-14
> Status: active Issue 31 breakdown

## Purpose

Motor implementation should not start until two prerequisite subtasks are
handled.

## Subtask 1: Core Pathway Topology Correction

### Goal

Make Afferent and Efferent Pathways explicit source-port + fixed middleware
sequence runtimes.

### Why

Motor needs to participate in both pathways. Current Core has concrete queues and
hard-coded consumers/sinks, but not a shared source-port / middleware decision
contract.

### Scope

Owned by Core / Stem pathway architecture.

Likely touched areas:

- `core/src/stem/afferent_pathway.rs`
- `core/src/stem/efferent_pathway.rs`
- `core/src/cortex/runtime/mod.rs`
- `core/src/main.rs`
- Product TDD coordination model.
- Core Unit TDD design / interfaces.

### Confirmed Decisions

- Public source-port APIs do not take source metadata.
- Middleware decision contract is `Accepted | Rejected | Continue(original/transformed)`.
- Middleware sequence is fixed and assembled in `main.rs`.
- There is no descriptor-selected routing for this sub-task.
- Tap is not a first-class pathway routing concept.
- Afferent deferral belongs to Cortex Afferent admission, not Afferent Pathway.
- Reflex-loop prevention is not an Afferent/Efferent responsibility.

See [core-topology-correction/SOLIDIFIED-CONTRACT.md](./core-topology-correction/SOLIDIFIED-CONTRACT.md).

### Acceptance Evidence

- Afferent and Efferent bus contracts are explicit in durable docs.
- Tests cover at least pass-through middleware and consumed-signal behavior.
- Tests cover multiple tx and multiple rx behavior.
- Existing Cortex -> Continuity -> Spine dispatch behavior remains covered.
- Existing endpoint Sense -> Cortex behavior remains covered.

## Subtask 2: Continuity Generic Store

### Goal

Evolve Continuity toward a generic durable store, with Cortex-authored routine
source definitions as the first forcing case.

### Why

Motor routines are learned procedural knowledge. If Cortex creates a routine,
the routine source must survive ordinary runtime cycles.

But Continuity should not become coupled to Motor routine semantics merely to
store routine source.

### Authority Change

This expands Continuity beyond current cognition-state persistence.

Current Continuity owns:

- `CognitionState`.
- goal forest validation.
- dispatch gate shape.

New proposed Continuity responsibility:

- generic durable records for learned cognition-adjacent state.

This must be documented explicitly; it should not be assumed from current code.

### Scope

Owned by Core / Continuity state architecture.

Likely touched areas:

- `core/src/continuity/state.rs`
- `core/src/continuity/engine.rs`
- `core/src/continuity/persistence.rs`
- `core/src/cortex/types.rs` or a new routine-definition type home.
- Product TDD state and authority docs.
- Core Unit TDD data/state and verification docs.

### Required Decisions

- Is the generic store a sibling to `CognitionState`, or does `CognitionState`
  become one store namespace?
- What generic record envelope does Continuity own?
- Does Continuity validate only durable record shape, or namespace-specific
  payloads too?
- What does Continuity care about for routine source:
  - identity.
  - namespace.
  - revision.
  - schema version.
  - payload envelope.
  - deterministic save/restore.
  - migration/load-time validation.
- What should Motor still own:
  - DSL syntax/semantic validation.
  - selector meaning.
  - activation state.
  - runtime behavior.
- Does Continuity persist activation state?
  - Current recommendation: no for MVP.
- How does Motor request persistence?
  - direct internal port?
  - Act-mediated pathway?
  - explicit Continuity service API?

### Acceptance Evidence

- Routine source definitions can be saved and restored deterministically.
- Invalid persisted routine source state is rejected on load.
- Existing cognition-state persistence remains compatible with the new state
  shape or migration policy.
- Motor can later read restored definitions without depending on Cortex context.
- Continuity does not depend on Motor routine execution semantics.

## Out Of Scope Until These Land

- Motor runtime implementation.
- Routine DSL choice.
- Routine lifecycle Act payload finalization.
- Agent Task Test success-rate comparison.

These depend on the topology and persistence boundaries above.
