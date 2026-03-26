# Deployment-Shaping Constraints

These constraints shape system design choices and unit contracts.

## Constraint Set

1. Core observability ownership
- Core exports runtime logs/metrics/traces.
- Endpoint units must not duplicate core runtime observability control surfaces.

2. Configuration authority
- Typed Rust config boundary defines runtime config shape/defaults/validation.
- Runtime startup is not gated by an independent hand-authored schema authority.

3. Reliability and safety constraints
- Shutdown must be bounded and explicit.
- Dispatch outcomes must remain explicit terminal classes.
- Hidden fallback behavior is disallowed in routing and config interpretation.

## Boundary To Deployment Docs

Product TDD defines deployment-shaping design constraints.
`docs/40-deployment` defines concrete runtime operational procedures and environment details.
