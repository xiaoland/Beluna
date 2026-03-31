# Docs Policy

`docs/00-meta` is a small Beluna-local note about the documentation system itself.
It is optional. Most work should start from `AGENTS.md` and the owning layer, not here.
Beluna keeps authoritative memory in `docs/` and volatile work in `tasks/`.

## Use This Folder Only When

- changing the documentation system itself
- resolving ownership ambiguity between durable layers
- deciding whether a truth belongs in docs, code/tests, or a task note

## Routing

- Product what/why, claims, workflows, scope, glossary: `docs/10-prd`
- Cross-unit technical truth: `docs/20-product-tdd`
- Hard-unit local design memory: `docs/30-unit-tdd`
- Runtime and operational truth: `docs/40-deployment`
- Volatile reasoning and plans: `tasks/`
- Mechanically enforced truth: code/tests/CI

## Promotion

Promote a finding into durable docs only when it is stable, reusable, costly to rediscover, and not better enforced mechanically.

## Demotion

Delete or simplify durable docs when they stop answering expensive recurring questions or duplicate a clearer authoritative owner.

## Note

- [`concepts.md`](./concepts.md) keeps a few repo-level workflow terms. Product and domain semantics belong in [`docs/10-prd/glossary.md`](../10-prd/glossary.md).
