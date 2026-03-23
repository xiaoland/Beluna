# Documentation System Rules

Beluna uses a layered documentation system to preserve alignment without excessive maintenance cost.

## Document Families and Ownership

| Area | Purpose | Mode | Notes |
|---|---|---|---|
| `00-meta` | Definitions, document meanings, governance rules. | Constitutional | Start here for terminology and system rules. |
| `10-prd` | Product intent, domains, workflows, and invariants. | Constitutional | Domain map is first-class; keep product discussion domain-oriented. |
| `20-product-tdd` | System-level technical composition of units. | Constitutional | Defines unit topology, coordination, and domain-to-unit realization. |
| `30-unit-tdd` | Local technical design per unit. | Operational-to-constitutional | Inherits Product TDD constraints; captures local interfaces, state, and verification. |
| `40-deployment` | Runtime and rollout truth. | Operational-to-constitutional | Environments, rollout, observability, recovery. |
| `90-decisions` | ADRs for decision memory and rationale. | Constitutional | Operative conclusions must still land in the TDD layers. |
| `AGENTS.md` | Agent governance (root + local refinements). | Constitutional | Hierarchical; nearest relevant file refines, not replaces, root rules. |
| `docs/task` | Procedural task plans and verification packets. | Operational | Non-authoritative; promote stable outcomes upward. |
| Codebase | Executable truth. | Operational | Tests, schemas, runtime behavior; must match the governing layers. |

Constitutional structure (definitions and governance) changes slowly; operational structure (daily workflow, tasks, and verification) changes as needed but must respect the constitutional anchors.

## Stability Placement

| Stability | Layers | Examples |
|---|---|---|
| **Slow** | `00-meta`, `10-prd`, `20-product-tdd`, major `30-unit-tdd` boundaries | terminology, product invariants, unit topology |
| **Medium** | unit interfaces, operations, `40-deployment` runbooks | interface contracts, deployment steps |
| **Fast** | `docs/task` plans and in-progress notes | task L0–L3 files, scratch analysis |
| **Executable** | codebase | tests, schemas, runtime validation |

Slow-layer statements must survive multiple task cycles without meaningful change.
Medium-layer statements should remain stable within a release cycle.
Fast-layer content is ephemeral and expires with the task that produced it.
Executable truth must always match authoritative layer contracts; divergence is a defect.

## Operating Workflow

1. **Classify** the change: terminology/meta, product intent, system design, unit-local design, deployment/runtime, or task-only.
2. **Anchor** in the right layer first (update or add the smallest governing statement before coding).
3. **Execute** the change in code/config with the governing anchor in view.
4. **Verify** using a task verification packet (intent, design consistency, behavior, operational readiness, evidence).
5. **Promote** only stable learning back into authoritative layers; keep one authoritative home per decision.

## Layer Shapes (Essentials)

### PRD (Domain-Oriented)

- Keep domain map explicit: boundaries, cross-domain workflows, and ownership rationale.
- Partition by business semantics, not by routes/screens. Include rules, invariants, edge cases, and acceptance hints per domain.

### Product TDD (System Composition)

- Define unit topology and why it is shaped that way.
- Describe coordination models (requests, events, ordering, failure behavior) that let units jointly realize product workflows.
- Bridge domains to units (who owns authoritative state, who participates, which coordination pattern applies).
- Capture inherited system constraints and deployment-shaping constraints that downstream Unit TDDs must respect.

### Unit TDD (Local Realization)

- Localize ownership: responsibilities, consumed/produced interfaces, data/state, and dependencies.
- Record local design assumptions, operational rules, and verification/guardrails.
- Inherit Product TDD constraints rather than redefining boundaries.

## Promotion Gate (Required)

A statement can be promoted from `docs/task` or from code into an authoritative layer only if all checks pass:

1. It matches current implementation/runtime behavior.
2. It is stable truth, not temporary execution detail.
3. It belongs to exactly one target layer.
4. It improves clarity and lowers ambiguity.

### Promotion Targets

| Discovery | Promote To |
|---|---|
| New canonical term or concept | `00-meta/concepts.md` |
| Product intent or invariant | `10-prd` |
| Cross-unit design decision | `20-product-tdd` |
| Unit-local design decision | `30-unit-tdd/<unit>` |
| Runtime or operational constraint | `40-deployment` |
| Significant design decision with rationale | `90-decisions` (ADR) |

### Promotion Rules

- Promote acceptance criteria into contracts when they recur, are stable across tasks, and are important enough to guide future work.
- Add guardrails to contracts when the rule is safety-critical, frequently violated, cheap to check mechanically, or when human review has proven unreliable.

### QA Cross-Cutting Guidance

When validating a change that spans multiple layers:

1. Confirm the outer-layer statement (PRD invariant or product-tdd contract) is satisfied.
2. Confirm unit-level contracts (interfaces, operations) remain consistent.
3. Confirm executable truth (tests, schema) enforces the contract.
4. If any layer is inconsistent, fix the authoritative layer first, then propagate downward.

### Task Verification Packet

For non-trivial tasks, carry a lightweight packet:

- Governing Anchors: stable docs this task depends on.
- Intended Change: what is being changed.
- Acceptance Criteria: what must be true for completion.
- Guardrails Touched: tests, schemas, CI checks, rollout checks involved.
- Evidence Expected: proof expected before closure.
- Promotion Candidates: recurring truths to promote back into durable docs.

## Task Quarantine (`docs/task`)

- `docs/task` is allowed and active, but non-authoritative.
- Durable docs may reference a task as evidence, not as governing truth.
- If a task conclusion becomes repeatedly relevant, promote it to `10/20/30/40` layers.

## ADR Usage

- ADRs preserve decision memory and rationale.
- Current operative conclusions must be reflected in `20-product-tdd` or `30-unit-tdd`.
- Reviewers should understand current system state without reading full ADR history.

## Deployment Boundary

Distinguish between deployment-shaping constraints captured in Product TDD and runtime truth in `40-deployment` (environments, rollout, observability, recovery).

## Split Rules

Split a file only when it becomes hard to read as one concept, sections change at different rates, contributors repeatedly touch unrelated parts, or misunderstandings stem from mixed abstraction levels. Do not split solely because a theoretical category exists.

## Anti-Patterns

- Treating chat history as source of truth.
- Mixing constitutional rules and daily workflow without acknowledging the difference.
- Leaving domain or unit boundaries implicit.
- Re-discovering coordination, verification, or runtime realities from code instead of capturing them in the governing layers.

## Removed Legacy Families

The following legacy families were removed from the authoritative map:

- `docs/features`
- `docs/modules`
- `docs/contracts`
- `docs/overview.md`
- `docs/glossary.md`
- `docs/descisions` (replaced by `docs/90-decisions`)

Do not reintroduce these as parallel authoritative systems.
