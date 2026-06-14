# Core Topology Correction

> Last Updated: 2026-06-14
> Status: implemented in Core; durable docs pending
> Scope: Core Afferent/Efferent pathway topology only

## Objective

Correct Afferent/Efferent Pathways from hard-coded single-consumer / fixed sink
runtime paths into source-port + fixed middleware sequence runtimes.

This is a prerequisite for Motor because Motor must participate in both
pathways without becoming a Cortex helper and without making Cortex call private
Motor APIs.

## Solidified Contract

See [SOLIDIFIED-CONTRACT.md](./SOLIDIFIED-CONTRACT.md).

## Implementation Result

See [IMPLEMENTATION-RESULT.md](./IMPLEMENTATION-RESULT.md).

## Supporting Notes

- [CORE-TOPOLOGY-REALITY.md](./CORE-TOPOLOGY-REALITY.md): current Core runtime
  topology and authority grounding.
- [PATHWAY-BUS-MODEL.md](./PATHWAY-BUS-MODEL.md): source-port + fixed
  middleware sequence model.
- [PATHWAY-BUS-IMPLEMENTATION.md](./PATHWAY-BUS-IMPLEMENTATION.md): Tokio
  implementation shape.
- [PATHWAY-MIDDLEWARE-CONTRACT.md](./PATHWAY-MIDDLEWARE-CONTRACT.md):
  middleware decision contract.
- [PATHWAY-SOURCE-MIDDLEWARE-MODEL.md](./PATHWAY-SOURCE-MIDDLEWARE-MODEL.md):
  earlier vocabulary exploration.
- [ARCHITECTURE-EXPLORATION.md](./ARCHITECTURE-EXPLORATION.md): repo-grounded
  exploration notes.

## Implementation Posture

Implemented the topology correction directly in Core:

- Afferent source ports expose `emit_sense` and `emit_sense_and_wait`.
- Efferent source ports expose `emit_act` and `emit_act_and_wait`.
- Afferent runtime executes a fixed middleware sequence.
- Efferent runtime executes a fixed middleware sequence.
- `main.rs` composes the sequences.
- Afferent deferral moves out of Afferent Pathway and into Cortex's
  Afferent-facing admission component.

Durable docs should be updated after implementation verifies the exact shape.
