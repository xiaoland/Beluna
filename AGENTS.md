# AGENTS.md of Beluna

Beluna is organized as a multi-component repository.
Documentation is selective memory, not a parallel runtime.

## Repository Layout

```text
.
├── core/               # Beluna core (Rust)
├── cli/                # Beluna body endpoint - CLI client (Rust)
├── apple-universal/    # Beluna body endpoint - Apple Universal app (Swift)
├── monitor/            # Beluna local observability monitor (Web)
├── docs/               # Authoritative layered docs
└── tasks/              # Volatile task workspace
```

## Minimal Cheat Sheet

- Unit: a logical technical boundary, not the same thing as a folder.
- PRD (`docs/10-prd/`): owns product intent, user-visible behavior, scope, and glossary.
- Product TDD (`docs/20-product-tdd/`): owns cross-unit technical contracts, topology, and authority boundaries.
- Unit TDD (`docs/30-unit-tdd/`): owns hard-unit local design memory and verification.
- Local `AGENTS.md`: owns tactical hazards and recurrence tripwires closest to code.
- `tasks/`: owns volatile exploration, diagnostics, and tactical artifacts.

## Documentation

- Start with this root `AGENTS.md`.
- Before changing files in a subtree, recursively inspect local `AGENTS.md` files from the target directory upward to the repo root.
- Use `docs/00-meta/` only when you need typed input protocols, mode SOPs, or framework ontology.
- Read `docs/00-meta/concepts.md` only when boundary language is unclear.
- Read only the owning slice of `docs/10-prd/`, `docs/20-product-tdd/`, `docs/30-unit-tdd/`, or `docs/40-deployment/`.
- Treat `tasks/` as procedural and non-authoritative.
- When implementation reveals reusable knowledge, promote it into durable docs.

## Operating Model

1. Classify the incoming request as `Intent`, `Constraint`, `Reality`, or `Artifact`.
2. Identify the durable owner and likely blast radius before acting.
3. For non-trivial work, open a task packet with the three MVT anchors:
   - `Objective & Hypothesis`
   - `Guardrails Touched`
   - `Verification`
4. Choose the active mode for the current slice of work: `Explore`, `Solidify`, `Execute`, or `Diagnose`.
5. Load only the matching route doc, mode SOP, and governing anchors needed for this step.
6. Execute and verify.
7. Switch modes when evidence or uncertainty changes.
8. Promote only stable truths after verification.

### Typed Input Guide

- `Intent`: the business wants new behavior, scope, or policy. Update PRD first.
- `Constraint`: product behavior stays the same, but technical or environment boundaries changed. Update Product TDD, Unit TDD, or Deployment docs as appropriate.
- `Reality`: observed behavior diverges from expectation. Collect evidence first, then fix the cause and add recurrence tripwires near code when warranted.
- `Artifact`: produce a bounded intermediate deliverable. Keep it tactical unless reuse is proven.

### Mode Guide

- `Explore`: map unknowns, alternatives, and assumptions.
- `Solidify`: restate findings into explicit claims, contracts, or decisions.
- `Execute`: implement a clear, verified change.
- `Diagnose`: investigate mismatches between expected and observed reality.

Mode guidance:

- do not assume one task equals one mode
- switch modes when evidence or clarity changes
- mode selection never overrides durable ownership

### Beluna Gates

- `Explore`: do not update durable docs or production code during open-ended exploration.
- `Solidify`: durable-doc updates are confirmation-gated; perform the restatement first and await human confirmation.
- `Execute`: straightforward low-risk localized changes may proceed directly; risky, reference-sensitive, or logic-altering changes are confirmation-gated.
- `Diagnose`: no evidence, no modification.

### Restatement Rule

For risky or reference-sensitive changes, restate:

- Target.
- Target path or anchor.
- State and context.
- Operation type.
- Scope boundaries.
- Invariants that must remain unchanged.
- Likely affected files.
- Uncertainty.

### Negotiation Triggers

Pause and ask the human when any of these happen:

- the requested change conflicts with an existing product claim or technical contract
- blast radius crosses multiple durable owners and the correct owner is unclear
- a shortcut would damage readability, maintainability, or an explicit guardrail
- evidence is insufficient for a bug fix or architectural decision

## Documentation Index

- [Meta Engine](./docs/00-meta/index.md): typed routes, mode SOPs, and framework ontology.
- [PRD](./docs/10-prd/index.md): product intent, behavior, scope, and glossary.
- [Product TDD](./docs/20-product-tdd/index.md): cross-unit technical realization.
- [Unit TDD](./docs/30-unit-tdd/index.md): unit-local contracts and verification.
- [Deployment](./docs/40-deployment/index.md): runtime and operational truth.
- [Tasks](./tasks/README.md): volatile planning, diagnostics, and artifact workspace.
- [Core AGENTS](./core/AGENTS.md)
- [Apple Universal AGENTS](./apple-universal/AGENTS.md)
- [Monitor AGENTS](./monitor/AGENTS.md)

> Add local `AGENTS.md` under complex modules when local constraints are needed.
> `tasks/` is procedural and non-authoritative.
> When implementation reveals reusable knowledge, promote it into durable docs.

## Engineering Constraints

- Less is more: quality over quantity; high cohesion and low coupling.
- No backward compatibility is required unless explicitly requested.
- Establish invariants at system boundaries and rely on them internally.
- Tooling: `jq`, `gh`, `rg`.
- Use delegation only when the active environment and task policy allow it.

## Coding Guidelines

- Prefer abstraction only when duplication or patterns become clear.
- Source files should stay under 300 lines where practical.

### Naming Conventions

- Omit `Beluna` prefix in directory names, file names, and internal docs.
- Keep `Beluna` in user-facing package names and documentation for discoverability.
