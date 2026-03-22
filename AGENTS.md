# AGENTS.md of Beluna Workspace

Respond to user in English.
If a user decision degrades readability or maintainability, pause and request a trade-off discussion.

Beluna is organized as a multi-component repository.

## Repository Layout (Crucial Only)

```text
.
├── core/               # Beluna runtime (Rust)
├── cli/                # Beluna body endpoint, CLI client (Rust)
├── apple-universal/    # Beluna body endpoint, Apple Universal app (Swift)
└── docs/               # Authoritative layered docs + ADRs + task workspace
```

## Authoritative Documentation Map

Read and keep these current:

- [Meta](./docs/00-meta/index.md): terminology and doc-system rules.
- [PRD](./docs/10-prd/index.md): product intent and product invariants.
- [Product TDD](./docs/20-product-tdd/index.md): system-level technical realization.
- [Unit TDD](./docs/30-unit-tdd/index.md): unit-level technical realization.
- [Deployment](./docs/40-deployment/index.md): runtime/deployment operational truth.
- [ADR](./docs/90-decisions/README.md): decision history and rationale.
- [Core AGENTS](./core/AGENTS.md)
- [Apple Universal AGENTS](./apple-universal/AGENTS.md)

### `docs/task` Rule

- `docs/task` is procedural and non-authoritative.
- Promote stable outcomes from tasks into authoritative layers above.

## Development Workflow

- Add local `AGENTS.md` under complex modules when local constraints are needed.
- When implementation reveals reusable knowledge, promote it into durable docs.
- Less is more: quality over quantity; high cohesion and low coupling.
- No backward compatibility is required unless explicitly requested.
- Establish invariants at system boundaries and rely on them internally.
- Inspect logs with `jq`.

## Coding Guidelines

- Prefer abstraction only when duplication or patterns become clear.
- Source files should stay under 300 lines where practical.

## Naming Conventions

- Omit `Beluna` prefix in directory names, file names, and internal docs.
- Keep `Beluna` in user-facing package names/documentation for discoverability.
