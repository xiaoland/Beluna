# AGENTS.md of Beluna

Beluna is organized as a multi-component repository.

## Repository Layout (Crucial Only)

```text
.
├── core/               # Beluna core (Rust)
├── cli/                # Beluna body endpoint - CLI client (Rust)
├── apple-universal/    # Beluna body endpoint - Apple Universal app (Swift)
├── monitor/            # Beluna local observability monitor (Web)
└── docs/               # Authoritative layered docs + task workspace
```

## Documentation System

Beluna uses a layered documentation profile. Use the smallest relevant slice for each task.

Typical references:

- [Meta](./docs/00-meta/index.md): terminology and doc-system rules.
- [Read Order](./docs/00-meta/read-order.md): advisory cross-layer reading strategy for humans and agents.
- [Intake Protocol](./docs/00-meta/intake-protocol.md): perturbation classification and containment workflow.
- [Promotion Rules](./docs/00-meta/promotion-rules.md): promotion gates and authoritative target rules.
- [PRD](./docs/10-prd/index.md): pressure-driven product truth (`_drivers -> behavior -> glossary -> domain-structure`).
- [Product TDD](./docs/20-product-tdd/index.md): system-level technical realization.
- [Unit TDD](./docs/30-unit-tdd/index.md): unit-level technical realization.
- [Deployment](./docs/40-deployment/index.md): runtime/deployment operational truth.
- [Core AGENTS](./core/AGENTS.md)
- [Apple Universal AGENTS](./apple-universal/AGENTS.md)

> Add local `AGENTS.md` under complex modules when local constraints are needed.
> When implementation reveals reusable knowledge, promote it into durable docs.

### Pre-Execution Restatement (Risky or Reference-Sensitive Changes)

Before implementation, restate:

- Target.
- Target path/anchor.
- State/context.
- Operation type.
- Scope boundaries.
- Invariants that must remain unchanged.
- Likely affected files.
- Uncertainty.

### `docs/task` Rule

- `docs/task` is procedural and non-authoritative.
- Promote stable outcomes from tasks into authoritative layers above.

## Development Workflow

- Less is more: quality over quantity; high cohesion and low coupling.
- No backward compatibility is required unless explicitly requested.
- Establish invariants at system boundaries and rely on them internally.
- Tooling: `jq`, `gh`, `rg`.
- Make use of sub-agents.

## Coding Guidelines

- Prefer abstraction only when duplication or patterns become clear.
- Source files should stay under 300 lines where practical.

## Naming Conventions

- Omit `Beluna` prefix in directory names, file names, and internal docs.
- Keep `Beluna` in user-facing package names/documentation for discoverability.
