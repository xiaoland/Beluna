# Operational Constraints

## Observability Ownership

1. Core exports logs, metrics, and traces via OTLP.
2. Core keeps local JSON runtime logs as baseline while supporting OTLP logs export.
3. Prometheus pull endpoint is not part of current Core runtime contract.
4. Apple body endpoint does not own runtime observability control surfaces.

Decision Source: ADR-001.

## Configuration Contract

1. Typed Rust config structs are the single source of truth for config shape/defaults.
2. Validation is performed at config boundary through typed validation rules.
3. `core/beluna.schema.json` is generated from code, not hand-authored as an independent source.
4. Runtime startup does not depend on schema-file validation path.

Decision Source: ADR-002.

## Reliability Constraints

1. Runtime shutdown must close ingress, cancel tasks, and perform bounded efferent drain.
2. Dispatch outcomes remain explicit (`Acknowledged`, `Rejected`, `Lost`).
3. Hidden fallback behavior is disallowed in routing and config interpretation.
