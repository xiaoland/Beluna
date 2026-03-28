# PRD Index

## Role in the system

`10-prd` is the authoritative layer for product intent.

It defines the pressure field Beluna must answer and the durable user-visible truth that technical realization must honor.
Read only the files relevant to the product question at hand.

## What this layer owns

1. Upstream product drivers in `_drivers` (market/user pressure, business/service objectives, hard constraints, operational realities).
2. Product behavior truth in `behavior` (claims, capabilities, workflows, rules/invariants, scope boundaries).
3. Canonical product/domain semantics in [`glossary.md`](./glossary.md).
4. Derived semantic structure in `domain-structure` (vocabulary, boundaries, lifecycle, cross-domain interactions) after drivers and behavior are stable.

## What must NOT appear here

1. Internal mechanism ordering and runtime scheduling internals.
2. Module ownership topology or technical unit decomposition policy.
3. Transport/wire internals and adapter implementation mechanics.
4. Unit-local interface/data contracts that do not define product truth.
5. Unit-to-container mapping decisions.

## How to read this layer

1. Start with the file that owns your question; do not read the whole layer by default.
2. For new or scope-shaping product work, begin with [`_drivers/index.md`](./_drivers/index.md) and [`behavior/index.md`](./behavior/index.md).
3. Use [`glossary.md`](./glossary.md) when term meaning matters.
4. Use [`domain-structure/index.md`](./domain-structure/index.md) only when boundary or lifecycle structure is part of the question.
5. Move to `20-product-tdd` only when you need system realization detail.

## How this layer connects to adjacent layers

1. PRD constrains Product TDD; `20-product-tdd` must explicitly realize PRD claims/workflows/rules.
2. Product and Unit TDD may refine technical realization but must not reinterpret upstream drivers.
3. Deployment docs operationalize runtime truth and must remain consistent with PRD behavior commitments.

## Common local mistakes

1. Starting from pre-selected domains instead of product pressure.
2. Treating `domain-structure` as an upstream requirement source.
3. Mixing technical mechanism contracts into PRD files.
4. Leaving product claims without evaluation dimensions and evidence expectations.
5. Updating user-visible behavior only in task/code without PRD promotion.
6. Leaving canonical product/domain semantics only in meta-layer docs instead of PRD.
