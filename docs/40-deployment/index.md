# Deployment Index

## Role in the system

`40-deployment` is the authoritative runtime truth layer.

It defines how Beluna is operated, observed, rolled out, and recovered in real environments.

## What this layer owns

1. Runtime environment profiles, operational prerequisites, and environment-specific constraints.
2. Rollout/change management procedures, rollback paths, and recovery operations.
3. Observability contracts for logs/metrics/traces and incident investigation evidence paths.
4. Operational guardrails and failure handling procedures validated in deployment contexts.

## What must NOT appear here

1. Product drivers/claims/workflow semantics (belongs to `10-prd`).
2. Technical unit decomposition policy, authority ownership design, or cross-unit contract definitions (belongs to `20-product-tdd`).
3. Unit-local internal implementation design that does not affect runtime operations (belongs to `30-unit-tdd/<unit>`).

## How to read this layer

1. Read [Environments](./environments.md) for runtime context and constraints.
2. Read [Rollout And Recovery](./rollout-and-recovery.md) for release, rollback, and incident handling procedures.
3. Read [Observability](./observability.md) for telemetry and debugging expectations.
4. Cross-check [Deployment-Shaping Constraints](../20-product-tdd/deployment-shaping-constraints.md) if runtime practice and system design diverge.

## How this layer connects to adjacent layers

1. Inherits deployment-shaping constraints from Product TDD and operationalizes them.
2. Feeds incident and runtime learning back to `20-product-tdd` and `30-unit-tdd` when stable technical truths emerge.
3. Must remain aligned with PRD user-visible reliability and operational trust expectations.

## Common local mistakes

1. Recording recurring deployment constraints only in task notes and not promoting them.
2. Treating rollback/recovery behavior as ad-hoc operator memory instead of explicit runbook truth.
3. Redefining cross-unit contracts in operational docs.
4. Leaving observability and incident evidence expectations implicit.

## Deployment Catalog

- [Environments](./environments.md)
- [Rollout And Recovery](./rollout-and-recovery.md)
- [Observability](./observability.md)
