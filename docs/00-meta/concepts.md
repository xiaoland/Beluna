# Concepts

This file defines shared terminology for humans and coding agents.

## Documentation Concepts

- Product: the user and operational value Beluna must realize.
- Product TDD: system-level technical design that realizes product intent.
- Unit TDD: local technical design within one technical management boundary.
- Unit: a technical planning and ownership boundary. In Beluna, initial units are `core`, `cli`, and `apple-universal`.
- Contract: stable normative behavior statement inside TDD/interface docs.
- Guardrail: executable or procedural enforcement of contracts (tests, schema validation, runtime checks).
- Stable Truth: information that remains useful and valid beyond one task.
- Task Truth: procedural or temporary decisions specific to one task iteration.

## Runtime Concepts

- Cortex: deliberative cognition runtime boundary.
- Stem: runtime orchestrator that owns tick grants, physical state mutation, and pathways.
- Continuity: deterministic cognition persistence and dispatch gate owner.
- Ledger: resource accounting and settlement owner.
- Spine: endpoint routing and act dispatch substrate.
- Body Endpoint: world-facing endpoint that publishes senses and executes acts.
- Afferent Pathway: bounded ingress pathway from endpoint/domain senses into Cortex runtime.
- Efferent Pathway: ordered act dispatch pathway (`Continuity -> Spine`).
- Physical State: ledger snapshot + descriptor catalog + proprioception snapshot visible to Cortex.
- Cognition State: persisted cognitive state (goal-forest and memory structures).
- Fully Qualified Signal ID: `endpoint_id/neural_signal_descriptor_id`.

## Terminology Rules

1. Use one canonical term for one concept.
2. Avoid deprecated names in new docs.
3. When terminology changes, update this file first, then propagate.
4. Avoid mixing product intent and implementation detail in one statement.
