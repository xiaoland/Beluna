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

## Decision Network & Workflow Layers

Beluna's documentation and development process forms a **decision network**: a set of interlocking layers where each layer's constraints and truths propagate downward and discoveries propagate upward.

### Layers (outer → inner)

| Layer | Folder | Governs |
|---|---|---|
| Product Intent | `10-prd` | What Beluna must do and why. |
| System Design | `20-product-tdd` | How units collaborate to realize product intent. |
| Unit Design | `30-unit-tdd` | Local design within one unit boundary. |
| Deployment | `40-deployment` | Runtime configuration and operational constraints. |
| Executable Truth | codebase | Tests, schemas, and runtime behavior. |

### Workflow Layers (per task)

- **L0 — Context**: understand the change scope and affected layers.
- **L1 — Strategy**: select the target layers and outline the approach.
- **L2 — Design**: specify interfaces, contracts, and cross-unit impact.
- **L3 — Plan**: concrete implementation steps and rollback criteria.
- **Execution**: implement, verify, and record stable outcomes.

### Decision Flow Rules

1. Decisions cascade **downward**: product intent constrains system design; system design constrains unit design.
2. Discoveries propagate **upward**: stable outcomes learned in tasks or code must be promoted to the appropriate layer.
3. A decision must live in **exactly one** authoritative layer; duplication creates drift.
4. When layers conflict, the outer layer wins. An ADR may record a rationale for a deliberate exception, but the ADR itself does not become an authoritative layer; any operative conclusion from an ADR must still be reflected in the appropriate `10–40` layer.

## Terminology Rules

1. Use one canonical term for one concept.
2. Avoid deprecated names in new docs.
3. When terminology changes, update this file first, then propagate.
4. Avoid mixing product intent and implementation detail in one statement.
