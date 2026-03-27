# Promotion Rules

This file defines when and where transient findings become durable truth.

Task notes (`docs/task`) are non-authoritative by default. Promotion is deliberate.

## Promotion Gate (Required)

A statement can be promoted only if all checks pass:

1. It matches current implementation/runtime behavior (or the behavior introduced by the same change set).
2. It is stable truth, not temporary execution detail.
3. It belongs to exactly one authoritative layer.
4. It improves clarity and reduces ambiguity for future work.

## Promotion Targets

| Discovery | Promote To |
|---|---|
| New canonical term or concept | `docs/00-meta/concepts.md` |
| Product driver/claim/invariant/workflow truth | `docs/10-prd` |
| Cross-unit design decision | `docs/20-product-tdd` |
| Unit-local design decision | `docs/30-unit-tdd/<unit>` |
| Runtime or operational constraint | `docs/40-deployment` |
| Mechanically checkable invariant | code guardrail (test/schema/CI check) |
| One-off execution detail | keep in `docs/task` only |

## When Acceptance Criteria Become Contracts

Promote acceptance criteria into durable contracts when they are:

- recurring across tasks
- stable over time
- important enough to guide future design or review
- costly or risky to repeatedly rediscover

## When Contracts Need Guardrails

Add executable guardrails when a contract is:

- safety-critical
- frequently violated
- cheap enough to check mechanically
- too unreliable to enforce through human review alone

## What Must Not Drift Downward

The following truths should not remain implicit in code or task history:

- product drivers that materially shape behavior
- claim semantics and major workflows
- stabilized domain boundaries
- technical unit boundaries and decomposition rules
- authority boundaries and cross-unit contracts
- unit-to-container mapping rationale
- global ontology and promotion policy
