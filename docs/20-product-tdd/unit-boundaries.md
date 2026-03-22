# Unit Boundaries

## Unit: `core`

Responsibility:

- Own runtime loop orchestration, cognition execution, routing, persistence, and resource control.

Contained subsystem boundaries:

- `cortex`: cognition execution boundary.
- `stem`: tick/pathway/physical-state orchestration.
- `continuity`: cognition persistence + dispatch gate.
- `spine`: endpoint routing and dispatch outcomes.
- `ledger`: resource accounting.
- `ai_gateway`: model/backend inference abstraction.
- `body`: built-in inline body endpoints.
- `observability`: telemetry and runtime tracing setup.

## Unit: `cli`

Responsibility:

- Provide a minimal Unix-socket body endpoint for terminal interaction and smoke usage.

## Unit: `apple-universal`

Responsibility:

- Provide Apple ecosystem body endpoint UX while keeping domain logic in Core.

Boundary Rules:

1. Body endpoints do not re-implement Core domain logic.
2. Runtime observability export ownership belongs to Core.
3. Cross-unit protocols are explicit and typed where possible.
