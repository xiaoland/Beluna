# Documentation System Rules

Beluna uses a layered documentation system to preserve alignment without excessive maintenance cost.

## Document Families and Ownership

| Area | Purpose | Mode | Notes |
|---|---|---|---|
| `00-meta` | Definitions, document meanings, governance rules. | Constitutional | Start here for terminology and system rules. |
| `10-prd` | Pressure-driven product truth (`_drivers -> behavior -> domain-structure`). | Constitutional | Drivers are upstream; domain structure is derived. |
| `20-product-tdd` | System-level technical composition of units. | Constitutional | Defines unit topology, coordination, and system constraints. |
| `30-unit-tdd` | Local technical design per unit. | Operational-to-constitutional | Inherits Product TDD constraints and localizes implementation contracts. |
| `40-deployment` | Runtime and rollout truth. | Operational-to-constitutional | Environments, rollout, observability, recovery. |
| `AGENTS.md` | Agent governance (root + local refinements). | Constitutional | Hierarchical; nearest relevant file refines, not replaces, root rules. |
| `docs/task` | Procedural task plans and verification packets. | Operational | Non-authoritative; promote stable outcomes upward. |
| Codebase | Executable truth. | Operational | Tests, schemas, runtime behavior; must match governing layers. |

Constitutional structure (definitions and governance) changes slowly; operational structure (daily workflow, tasks, and verification) changes as needed but must respect constitutional anchors.

The `00-meta` baseline is:

- `concepts.md`
- `doc-system.md`
- `read-order.md`
- `intake-protocol.md`

## Stability Placement

| Stability | Layers | Examples |
|---|---|---|
| **Slow** | `00-meta`, `10-prd`, `20-product-tdd`, major `30-unit-tdd` boundaries | terminology, product claims, unit topology |
| **Medium** | unit interfaces, operations, `40-deployment` runbooks | interface contracts, deployment procedures |
| **Fast** | `docs/task` plans and in-progress notes | task L0-L3 files, scratch analysis |
| **Executable** | codebase | tests, schemas, runtime validation |

Slow-layer statements must survive multiple task cycles without meaningful change.
Medium-layer statements should remain stable within a release cycle.
Fast-layer content is ephemeral and expires with the task that produced it.
Executable truth must always match authoritative layer contracts; divergence is a defect.

## Operating Workflow

1. **Classify perturbation type**: `Intent`, `Constraint`, `Reality`, or `Artifact`.
2. **Contain volatility** in `docs/task/<task>/` with perturbation, impact hypothesis, assumptions, and negotiation triggers.
3. **Anchor** in the right durable layer first (update or add the smallest governing statement before coding).
4. **Execute** code/config changes under explicit acceptance criteria and guardrails.
5. **Verify** intent, design consistency, behavior correctness, and operational readiness with evidence.
6. **Decide outcome** explicitly (`promote`, `complete_without_promotion`, `defer`, `reject`, or `experiment`).
7. **Promote** only stable learning back into authoritative layers; keep one authoritative home per decision.

## Intake Classification (Required for Non-Trivial Changes)

- `Intent`: behavior/product request pressure; usually hits PRD first, then TDD.
- `Constraint`: budget/platform/performance/team limits; usually hits Product TDD, Unit TDD, or deployment.
- `Reality`: bug/incident/observed failure; usually hits code + verification first, then back-propagation.
- `Artifact`: code/schema/log/draft input; interpret first, then classify as one of the above.

## Layer Shapes (Essentials)

### PRD (Pressure-Driven, Claim-Centered)

- `10-prd/_drivers` defines upstream pressure field: market/user, business/service, hard constraints, operational realities.
- `10-prd/behavior` defines product claims, capabilities, user-observable workflows, and product rules/invariants.
- `10-prd/domain-structure` records derived semantic boundaries after drivers and behavior are stable.

Derived-domain rule:

- Domain boundaries are structuring outcomes of PRD, not upstream requirement sources.

PRD layer purity rule:

- PRD governs product truth only.
- PRD must not govern mechanism ordering, module ownership, transport internals, or local technical contracts.
- Mechanism contracts belong to `20-product-tdd`, `30-unit-tdd`, and `40-deployment`.

### Product TDD (System Composition)

- Define unit topology and why it is shaped that way.
- Describe coordination models (requests, events, ordering, failure behavior) that realize PRD behavior.
- Capture inherited system constraints and deployment-shaping constraints for Unit TDD.
- Keep governance and trace artifacts explicit:
  - `system-objective`
  - `unit-topology`
  - `unit-boundary-rules`
  - `unit-to-container-mapping`
  - `coordination-model`
  - `cross-unit-contracts`
  - `system-state-and-authority`
  - `claim-realization-matrix`
  - `failure-and-recovery-model`
  - `deployment-shaping-constraints`

### Unit TDD (Local Realization)

- Localize ownership: responsibilities, consumed/produced interfaces, data/state, and dependencies.
- Record local design assumptions, operational rules, and verification/guardrails.
- Inherit Product TDD constraints rather than redefining system boundaries.

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
| Product driver/claim/invariant/workflow truth | `10-prd` |
| Cross-unit design decision | `20-product-tdd` |
| Unit-local design decision | `30-unit-tdd/<unit>` |
| Runtime or operational constraint | `40-deployment` |
| Historical rationale that remains useful | Decision-memory section in the owning authoritative doc |

### Promotion Rules

- Promote acceptance criteria into contracts when they recur, are stable across tasks, and are important enough to guide future work.
- Add guardrails to contracts when the rule is safety-critical, frequently violated, cheap to check mechanically, or when human review has proven unreliable.

### QA Cross-Cutting Guidance

When validating a change that spans multiple layers:

1. Confirm PRD claims/rules are still satisfied.
2. Confirm Product/Unit TDD contracts remain consistent.
3. Confirm executable truth (tests, schema) enforces contracts.
4. If any layer is inconsistent, fix the authoritative layer first, then propagate downward.

### Task Verification Packet

For non-trivial tasks, carry a lightweight packet:

- Perturbation: raw user/reality signal.
- Input Type: `Intent` / `Constraint` / `Reality` / `Artifact`.
- Governing Anchors: stable docs this task depends on.
- Intended Change: what is being changed.
- Impact Hypothesis: primary hit, likely secondary hits, confidence, unknowns.
- Temporary Assumptions: reversible assumptions made to proceed.
- Negotiation Triggers: what requires explicit user decision before continuing.
- Acceptance Criteria: what must be true for completion.
- Guardrails Touched: tests, schemas, CI checks, rollout checks involved.
- Evidence Expected: proof expected before closure.
- Outcome: `promote` / `complete_without_promotion` / `defer` / `reject` / `experiment`.
- Promotion Candidates: recurring truths to promote back into durable docs.

## Task Quarantine (`docs/task`)

- `docs/task` is allowed and active, but non-authoritative.
- Durable docs may reference a task as evidence, not as governing truth.
- If a task conclusion becomes repeatedly relevant, promote it to `10/20/30/40` layers.

## Decision Memory

- Beluna does not maintain a separate ADR family.
- Important rationale should be captured inline in the authoritative doc that owns the live rule.
- Reviewers should understand current state and rationale from current layer docs without replaying detached history folders.

## Deployment Boundary

Distinguish between deployment-shaping constraints in Product TDD and runtime truth in `40-deployment` (environments, rollout, observability, recovery).

## Split Rules

Split a file only when it becomes hard to read as one concept, sections change at different rates, contributors repeatedly touch unrelated parts, or misunderstandings stem from mixed abstraction levels. Do not split solely because a theoretical category exists.

## Anti-Patterns

- Treating chat history as source of truth.
- Treating derived domain structure as upstream requirement source.
- Mixing product truth with implementation mechanisms in PRD.
- Mixing constitutional rules and daily workflow without acknowledging the difference.
- Re-discovering coordination, verification, or runtime realities from code instead of capturing them in governing layers.

## Removed Legacy Families

The following legacy families are removed from the authoritative map:

- `docs/features`
- `docs/modules`
- `docs/contracts`
- `docs/overview.md`
- `docs/glossary.md`
- `docs/descisions`
- `docs/90-decisions`

Do not reintroduce these as parallel authoritative systems.
