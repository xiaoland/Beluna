# Concepts

This file defines shared terminology for humans and coding agents.

## Ontology

- Product: user and business value Beluna must realize, including operational realities.
- Product Driver: market, business, constraint, or operational pressure that shapes product truth.
- Product Claim: durable product promise used to evaluate whether Beluna is delivering intended value.
- Domain: derived semantic structure that stabilizes meaning boundaries after product drivers and behavior claims are defined.
- Capability: something the product or system can do; may span domains or units.
- Workflow: ordered behavior path across actors, states, and system steps.
- Unit: technical planning and ownership boundary; current units are `core`, `cli`, and `apple-universal`.
- Product TDD: system-level technical design that composes units to realize product intent.
- Unit TDD: local technical design within one unit that inherits Product TDD constraints.
- Contract: stable normative statement that should guide implementation and verification.
- Guardrail: executable or procedural enforcement of a contract (tests, schema validation, CI checks).
- Acceptance Criteria: task-scoped verification targets; recurring stable ones may be promoted into contracts.
- Stable Truth: information useful beyond one task cycle.
- Task Truth: procedural or temporary decisions specific to a task iteration.

## Structural Modes

- **Constitutional structure**: definitions, document meanings, and governance rules (e.g., this file, `doc-system.md`).
- **Operational structure**: day-to-day workflow for initiating, verifying, promoting, and propagating work.

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

Beluna's documentation and development process forms a **decision network**, not a waterfall. Layers constrain one another and are updated iteratively.

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
4. When layers conflict, the outer layer wins and the conflict must be resolved in authoritative layer docs rather than task history.

## Terminology Rules

1. Use one canonical term for one concept.
2. Avoid deprecated names in new docs.
3. When terminology changes, update this file first, then propagate.
4. Avoid mixing product intent and implementation detail in one statement.
