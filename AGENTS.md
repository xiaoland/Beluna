# AGENTS.md of Beluna

Beluna is organized as a multi-component repository.

## Repository Layout

```text
.
├── core/               # Beluna core (Rust)
├── cli/                # Beluna body endpoint - CLI client (Rust)
├── apple-universal/    # Beluna body endpoint - Apple Universal app (Swift)
├── monitor/            # Beluna local observability monitor (Web)
└── docs/               # Authoritative layered docs + task workspace
```

## Working Mode

Treat documentation as a decision network, not a linear read ritual.

For any non-trivial change:

1. Classify the incoming signal:
   - `Intent`: new behavior, UX direction, or policy.
   - `Constraint`: platform, budget, performance, or team limit.
   - `Reality`: bug, incident, runtime failure, or observed drift.
   - `Artifact`: code, schema, logs, screenshots, or draft docs that must be interpreted first.
2. Choose the governing layer before editing:
   - `docs/10-prd`: product what/why, user-visible behavior, scope, glossary.
   - `docs/20-product-tdd`: cross-unit design, authority, coordination, realization.
   - `docs/30-unit-tdd`: unit-local contracts and verification.
   - `docs/40-deployment`: environments, rollout, observability, recovery.
   - Code/tests: executable truth and guardrails.
3. Read the smallest relevant slice:
   - nearest relevant `AGENTS.md`
   - owning layer `index.md`
   - exact authoritative files for the decision being changed
   - adjacent layers only when scope crosses boundaries
   - `docs/00-meta` only when the task touches documentation policy, cross-layer ambiguity, or promotion/demotion rules
4. Work through task depth only as needed:
   - `L0`: context and scope
   - `L1`: strategy
   - `L2`: design/contracts
   - `L3`: implementation plan and rollback
5. Execute and verify under explicit invariants and guardrails.
6. Promote stable truth upward; keep one authoritative owner per decision.

## Documentation System

- [Meta](./docs/00-meta/index.md): Documents' coordination rules for cross-layer work; usually not the first stop.
- [PRD](./docs/10-prd/index.md): product intent, scope, workflows, rules, glossary.
- [Product TDD](./docs/20-product-tdd/index.md): cross-unit technical realization.
- [Unit TDD](./docs/30-unit-tdd/index.md): unit-local contracts and verification.
- [Deployment](./docs/40-deployment/index.md): runtime and operational truth.
- [Core AGENTS](./core/AGENTS.md)
- [Apple Universal AGENTS](./apple-universal/AGENTS.md)
- [Monitor AGENTS](./monitor/AGENTS.md)

> Add local `AGENTS.md` under complex modules when local constraints are needed.
> When implementation reveals reusable knowledge, promote it into durable docs.
> `docs/task` is procedural and non-authoritative.

## Development Workflow

- Less is more: quality over quantity; high cohesion and low coupling.
- No backward compatibility is required unless explicitly requested.
- Establish invariants at system boundaries and rely on them internally.
- Tooling: `jq`, `gh`, `rg`.
- Use delegation only when the active environment and task policy allow it.

## Coding Guidelines

- Prefer abstraction only when duplication or patterns become clear.
- Source files should stay under 300 lines where practical.

## Naming Conventions

- Omit `Beluna` prefix in directory names, file names, and internal docs.
- Keep `Beluna` in user-facing package names/documentation for discoverability.
