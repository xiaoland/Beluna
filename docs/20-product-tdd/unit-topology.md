# Unit Topology

Beluna currently has three technical units.

## Units

1. `core`
- Runtime authority and composition root.
- Owns cognition execution, routing, persistence, resource control, and observability export.

2. `cli`
- Minimal terminal-oriented external body endpoint.
- Uses core endpoint protocol; does not own core domain authority.

3. `apple-universal`
- Apple ecosystem external body endpoint UX.
- Uses core endpoint protocol; does not own core domain authority.

## Core Internal Subsystems (Inside `core` Unit)

- `cortex`
- `stem`
- `continuity`
- `spine`
- `ledger`
- `ai_gateway`
- `body`
- `observability`

These remain internal module boundaries inside the `core` technical unit.
