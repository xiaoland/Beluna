# AGENTS.md of Beluna Workspace

Beluna is now organized as a multi-component repository.

## Repository Layout (Crucial Only)

```
.
├── core/               # Core (Rust)
├── cli/                # CLI body endpoint client (Rust)
├── apple-universal/    # Apple Universal App (Swift)
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

> Note that documents are for communication only, code are the single source of truth.
> You are encouraged to add an AGENTS.md file under modules with significant complexity when needed.
