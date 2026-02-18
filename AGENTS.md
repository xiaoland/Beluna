# AGENTS.md of Beluna Workspace

Beluna is now organized as a multi-component repository.

## Repository Layout (Crucial Only)

```
.
├── core/               # Beluna (Rust)
├── cli/                # Beluna Body Endpoint, CLI (Rust)
├── apple-universal/    # Beluna Body Endpoint, Apple Universal App (Swift)
└── docs/               # Product, BDT contracts, modules, ADR and tasks
```

## Component Guides

When editing a component, treat its local `AGENTS.md` as authoritative for that scope.

## Documents

Read following documents if needed, and keep them current:

- [Overview](./docs/overview.md): Product overview.
- [Glossary](./docs/glossary.md): Product top-level glossary.
- [Feature Document](./docs/features/README.md): Feature document, answers "What to do", each feature has its PRD, HLD and LLD.
- [Modules](./docs/modules/README.md)
- [BDT Contract](./docs/contracts/README.md)
- [ADR](./docs/descisions/README.md)
- [Core's AGENTS.md](./core/AGENTS.md)
- [Apple Universal App's AGENTS.md](./apple-universal/AGENTS.md)

Note that documents are for communication only, code are the single source of truth.

You are encouraged to add an AGENTS.md file under modules with significant complexity when needed.
Whenever new understanding emerges during implementation, you should decide whether that knowledge is reusable. If yes, it must be promoted from working memory to documentation.

## Coding Guidelines

- Prefer abstraction only when duplication or patterns become clear.

### Naming Conventions

- Directory names: Omit "Beluna" prefix. Use short, descriptive names.
  - ✅ `cli/`, `core/`, `apple-universal/`
  - ❌ `beluna-cli/`, `beluna-core/`

- Package names: Include "Beluna" for user clarity and discoverability.
  - ✅ `beluna-cli` (in Cargo.toml)
  - ❌ `cli` (too generic for package registries)

- User-facing documentation: Include "Beluna" branding for clarity.
  - ✅ "Beluna CLI", "Beluna Core" (in README.md, user guides)
  - ❌ "CLI", "Core" (too vague for users)

- Internal documentation: Use shorter names for agent efficiency.
  - ✅ "Core", "CLI" (in AGENTS.md, architecture docs)
  - Purpose: Keeps agent context concise
