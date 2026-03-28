# Product TDD Index

## Role in the system

`20-product-tdd` is the authoritative layer for system-level technical composition.

It translates PRD intent into technical units, contracts, authority boundaries, and realization traces.

## What this layer owns

1. Technical unit topology and decomposition policy.
2. Cross-unit coordination model and contract surfaces (APIs/events/schemas/compatibility/failure semantics).
3. Authority ownership and system state boundaries across units.
4. Claim/workflow realization trace from PRD behavior to technical units and verification homes.
5. Unit-to-container mapping rationale and deployment-shaping constraints inherited by Unit TDD.

## What must NOT appear here

1. New product drivers/claims/rules (belongs to `10-prd`).
2. Unit-internal implementation detail that does not affect cross-unit behavior (belongs to `30-unit-tdd/<unit>`).
3. Runtime runbook procedures and environment operations detail (belongs to `40-deployment`).

## How to read this layer

1. Read [System Objective](./system-objective.md) first.
2. Read [Unit Topology](./unit-topology.md) and [Unit Boundary Rules](./unit-boundary-rules.md) to understand decomposition policy.
3. Read [Unit-To-Container Mapping](./unit-to-container-mapping.md) to distinguish technical units from code containers.
4. Read [System State And Authority](./system-state-and-authority.md) and [Cross-Unit Contracts](./cross-unit-contracts.md) for authority and boundary contracts.
5. Read [Coordination Model](./coordination-model.md), [Failure And Recovery Model](./failure-and-recovery-model.md), and [Deployment-Shaping Constraints](./deployment-shaping-constraints.md).
6. Read [Claim Realization Matrix](./claim-realization-matrix.md) to trace PRD claims/workflows into unit behavior and verification.

## How this layer connects to adjacent layers

1. Inherits product truth from PRD and turns it into system-level technical contracts.
2. Constrains Unit TDD by defining what units may decide locally versus what must be escalated, including when full unit-local contract memory is required.
3. Provides deployment-shaping constraints to `40-deployment`, while runtime execution truth remains in deployment docs.

## Common local mistakes

1. Treating repository layout as a substitute for technical unit boundaries.
2. Leaving authority boundaries implicit or scattered in unit-local docs.
3. Leaving claim-to-unit realization implicit.
4. Allowing Unit TDD docs to redefine cross-unit contracts.
5. Mixing rollout/runbook procedures into Product TDD instead of deployment docs.
6. Requiring full Unit TDD ceremony for straightforward units without hard-unit signals.

## Product TDD Catalog

- [System Objective](./system-objective.md)
- [Unit Topology](./unit-topology.md)
- [Unit Boundary Rules](./unit-boundary-rules.md)
- [Unit-To-Container Mapping](./unit-to-container-mapping.md)
- [Coordination Model](./coordination-model.md)
- [Cross-Unit Contracts](./cross-unit-contracts.md)
- [System State And Authority](./system-state-and-authority.md)
- [Claim Realization Matrix](./claim-realization-matrix.md)
- [Failure And Recovery Model](./failure-and-recovery-model.md)
- [Deployment-Shaping Constraints](./deployment-shaping-constraints.md)
