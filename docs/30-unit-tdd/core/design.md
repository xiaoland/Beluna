# Core Design

## Responsibility

Core owns runtime composition and the domain execution loop.

## Internal Boundaries

1. `stem`: tick authority, pathways, physical-state ownership.
2. `cortex`: cognition cycle execution and internal cognition tooling.
3. `continuity`: cognition persistence and dispatch gate.
4. `spine`: mechanical endpoint dispatch and adapter lifecycle.
5. `ledger`: resource accounting.
6. `ai_gateway`: provider-agnostic inference.
7. `body`: built-in inline endpoint implementations.

## Design Invariants

1. Core boundary invariants are established at startup and enforced at runtime boundaries.
2. Dispatch pipeline order remains deterministic (`Continuity -> Spine`).
3. Cognition persistence is deterministic and guardrailed.
4. Runtime topology is explicit; ownership and side effects remain localized.
