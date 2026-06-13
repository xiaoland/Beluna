# Meta Engine Index

`docs/00-meta` keeps Beluna-local typed input protocols, mode SOPs, implementation taste, and a small framework ontology.
It is optional. Most work should start from `AGENTS.md` and the owning layer, not here.

Input route decides ownership.
Mode decides working posture.
They are related, but they are not the same thing.

## Use This Folder Only When

- changing the documentation system itself
- resolving ownership ambiguity between durable layers
- choosing the correct input route or mode SOP for non-trivial work
- shaping non-trivial implementation structure, boundaries, data shape, authority flow, naming, abstraction, or complexity budget
- framework terminology is unclear

## What Lives Here

- `input-*.md`: front-door routing rules for `Intent`, `Constraint`, `Reality`, and `Artifact`
- `mode-*.md`: SOP overlays for `Explore`, `Solidify`, `Execute`, and `Diagnose`
- `implementation-taste.md`: language- and tech-stack-neutral implementation judgment
- `concepts.md`: minimal workflow ontology for Beluna's documentation system

## Load Rules

1. Start from root `AGENTS.md`.
2. Read only the matching input route doc.
3. Read only the current mode SOP.
4. Load `implementation-taste.md` for non-trivial code design or implementation changes.
5. Load `concepts.md` only when boundary language is unclear.
6. Do not treat `00-meta` as a business or domain glossary.

## Source Search Defaults

For ordinary source and durable-doc search, exclude volatile and generated surfaces by default:

- `tasks/`
- `temp/`
- generated output such as `build/`, `dist/`, and coverage reports
- dependency folders such as `node_modules/`
- virtual environments and tool caches such as `.venv/`, `.tox/`, and `.pytest_cache/`

Search those surfaces only when the active question targets them, when recovering the active task packet, or when reviewing evidence deliberately stored there.

## Impact Handshake

Before mutating durable truth when the blast radius is not obviously local, pause and restate:

- Address and Object: the anchors or symbols that will be touched
- State Diff: `From -> To`
- Blast Radius Forecast: expected files, modules, or downstream surfaces affected
- Invariants Check: what is explicitly protected from change
- Verification: the concrete proof that will bound side effects

If evidence is missing or the owning layer is still unclear, return to `Explore` or `Diagnose` instead of handshaking guesses.

## Durable Ownership

- Product what, why, claims, workflows, scope, glossary: `docs/10-prd`
- Cross-unit technical truth: `docs/20-product-tdd`
- Hard-unit local design memory: `docs/30-unit-tdd`
- Runtime and operational truth: `docs/40-deployment`
- Volatile reasoning, diagnostics, collaboration state, evidence, and tactical artifacts: `tasks/`
- Mechanically enforced truth: code, tests, type systems, CI, runtime assertions

## Promotion

Promote a finding into durable docs only when it is stable, reusable, costly to rediscover, and not better enforced mechanically.

## Demotion

Delete, simplify, or merge durable docs when they stop answering expensive recurring questions or duplicate a clearer authoritative owner.

## Note

- [`concepts.md`](./concepts.md) keeps framework workflow terms only.
- [`implementation-taste.md`](./implementation-taste.md) keeps language- and tech-stack-neutral implementation taste.
- Product and domain semantics belong in [`docs/10-prd/glossary.md`](../10-prd/glossary.md).
