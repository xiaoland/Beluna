# Concepts

This file defines Beluna's cross-layer operating terms for humans and coding agents.

Canonical product/domain semantics belong in PRD (`docs/10-prd`), especially [`glossary.md`](../10-prd/glossary.md).
Beluna runtime terms such as `Cortex`, `Stem`, `Continuity`, `Ledger`, and `Spine` belong in Product TDD or the relevant Unit TDD, not in `00-meta`.

## Core Operating Terms

- Decision Network: the way Beluna's durable layers and executable truth constrain one another without forming a waterfall.
- Perturbation: any incoming request, incident, constraint, or artifact that may require a change.
- Governing Anchor: the smallest authoritative doc or executable guardrail that currently owns the decision being changed.
- Unit: a technical ownership boundary. Current units are `core`, `cli`, `apple-universal`, and `monitor`.
- Product TDD: the authoritative cross-unit technical design layer.
- Unit TDD: the authoritative unit-local design and verification layer.
- Contract: a stable rule that should guide implementation and review.
- Guardrail: executable or procedural enforcement of a contract.
- Acceptance Criteria: task-scoped completion targets that may later be promoted into contracts.
- Stable Truth: information that should survive beyond a single task cycle.
- Task Truth: temporary reasoning or execution detail that belongs only in `docs/task`.
- Promotion: moving stable truth into its authoritative layer.
- Demotion: simplifying or removing durable text that no longer deserves preservation.

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
3. For product/domain semantic terms, update PRD glossary first, then propagate.
4. For cross-layer operational/governance terms, update this file first, then propagate.
5. Avoid mixing product intent and implementation detail in one statement.
