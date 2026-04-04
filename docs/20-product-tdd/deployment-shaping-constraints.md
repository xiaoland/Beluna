# Deployment-Shaping Constraints

These constraints shape system design choices and unit contracts.

## Constraint Set

1. Core observability ownership
- Core exports runtime logs/metrics/traces.
- Non-core units must not duplicate core runtime observability control surfaces.

2. Configuration authority
- Typed Rust config boundary defines runtime config shape/defaults/validation.
- Runtime startup is not gated by an independent hand-authored schema authority.

3. Reliability and safety constraints
- Shutdown must be bounded and explicit.
- Dispatch outcomes must remain explicit terminal classes.
- Hidden fallback behavior is disallowed in routing and config interpretation.

4. Local control-plane constraints
- A first-party local control plane may prepare artifacts, supervise core, and consume runtime observability.
- That control plane must not become the authority for runtime behavior, config shape, or observability emission policy.

5. Local observability constraints
- Local first-party storage/query targets logs as the primary signal.
- Metrics and traces may surface exporter status and handoff destinations without becoming first-party locally stored datasets.

6. Release packaging constraints
- Release workflow output must follow the cross-unit packaging contract before Moira release-consumer automation is considered correct.
- Workflow implementation may not redefine archive naming, checksum naming, or the first supported target matrix ad hoc.
- The initial producer contract is:
  - `beluna-core-<rust-target-triple>.tar.gz`
  - release-level `SHA256SUMS`
  - current first supported Moira consumer target `aarch64-apple-darwin`

## Boundary To Deployment Docs

Product TDD defines deployment-shaping design constraints.
`docs/40-deployment` defines concrete runtime operational procedures and environment details.
