# Documentation System Boundary

Beluna uses a layered documentation system, but `00-meta` is intentionally small.
This file defines what the doc system itself owns and what it must not absorb.

## The Layers

| Area | Owns |
|---|---|
| `AGENTS.md` | Agent entry protocol and navigation rules. |
| `docs/00-meta` | Cross-layer coordination rules. |
| `docs/10-prd` | Product what/why, scope, user-visible behavior, glossary. |
| `docs/20-product-tdd` | Cross-unit technical realization. |
| `docs/30-unit-tdd` | Unit-local contracts and verification. |
| `docs/40-deployment` | Runtime environments, rollout, observability, recovery. |
| `docs/task` | Transient task reasoning only. |
| Code/tests | Executable truth and guardrails. |

## What `00-meta` Owns

`00-meta` exists only for cross-layer coordination:

- shared operational terms
- decision-network workflow rules
- loading guidance for ambiguous work
- intake rules for non-trivial changes
- promotion and demotion rules
- boundaries of the documentation system itself

## What `00-meta` Must Not Own

Do not place these here:

- canonical product/domain semantics
- system decomposition or runtime authority truth
- unit-local design contracts
- deployment/runbook truth
- task-specific reasoning that will expire

Those belong in `10/20/30/40` or `docs/task`, depending on stability and scope.

## Default Routing

Use this routing when deciding where truth belongs:

| Decision Type | Home |
|---|---|
| Product claim, workflow, invariant, glossary term | `docs/10-prd` |
| Cross-unit design, authority, coordination, realization trace | `docs/20-product-tdd` |
| Unit-local interface, data/state, operations, verification | `docs/30-unit-tdd/<unit>` |
| Runtime environment, rollout, observability, recovery | `docs/40-deployment` |
| Cross-layer operating term or doc-system rule | `docs/00-meta` |
| Temporary task reasoning | `docs/task` |
| Mechanically enforced contract | code/tests/schemas/CI |

## Rules

1. Keep one authoritative owner per decision.
2. Prefer the owning layer over a cross-layer summary.
3. If code/tests can enforce a rule better than prose, use an executable guardrail.
4. Demote or remove durable text when it no longer answers an expensive recurring question.
5. If layers conflict, fix the authoritative layer first instead of explaining the conflict in task history.
