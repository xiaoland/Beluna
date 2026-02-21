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

## Documents

Read following documents if needed, and keep them current:

- [Overview](./docs/overview.md): Product overview.
- [Glossary](./docs/glossary.md): Product top-level glossary.
- [Feature Document](./docs/features/README.md): Feature document, answers "What to do", each feature has its PRD, HLD and LLD.
- [Modules](./docs/modules/README.md)
- [ADR](./docs/descisions/README.md)
- [Core's AGENTS.md](./core/AGENTS.md)
- [Apple Universal App's AGENTS.md](./apple-universal/AGENTS.md)
- [CLI's AGENTS.md](./core/AGENTS.md)

## Development Workflow

- You are encouraged to add an AGENTS.md file under modules with significant complexity when needed.
- Whenever new understanding emerges during implementation, you should decide whether that knowledge is reusable. If yes, it must be promoted from working memory to documentation.
- Stop maintaining/running tests for all projects, just make sure the build passes.

## Coding Guidelines

- Prefer abstraction only when duplication or patterns become clear.

### Naming Conventions

- Omit "Beluna" prefix in directory name, file name and internal documents. Use short, descriptive names. eg. use `cli/`, instead of `beluna-cli`.
- Include "Beluna" in package name (eg. `Cargo.toml`), user-facing documentation for user clarity and discoverability. eg. use `Beluna CLI` instead of `CLI`.
