# PRD Index

## Role in the system

`10-prd` is the authoritative layer for product intent.

It defines the pressure field Beluna must answer and the durable user-visible truth that technical realization must honor.
Read only the files relevant to the product question at hand.

## What this layer owns

1. Upstream product drivers in `_drivers` (market and user pressure, business and service objectives, hard constraints, operational realities).
2. Product behavior truth in `behavior` (claims, capabilities, workflows, rules and invariants, scope boundaries).
3. Canonical product and domain semantics in [`glossary.md`](./glossary.md).
4. Derived semantic structure in `domain-structure` after glossary terms, drivers, and behavior are stable.

## What must NOT appear here

1. Internal mechanism ordering and runtime scheduling internals.
2. Module ownership topology or technical unit decomposition policy.
3. Transport and wire internals plus adapter implementation mechanics.
4. Unit-local interface and data contracts that do not define product truth.
5. Unit-to-container mapping decisions.

## How to read this layer

1. Start with the file that owns your question; do not read the whole layer by default.
2. For new or scope-shaping product work, begin with [`_drivers/index.md`](./_drivers/index.md) and [`behavior/index.md`](./behavior/index.md).
3. Use [`glossary.md`](./glossary.md) when term meaning matters.
4. Use [`domain-structure/index.md`](./domain-structure/index.md) only when derived boundary or lifecycle structure is part of the question.
5. Move to `20-product-tdd` only when you need system realization detail.

## How this layer connects to adjacent layers

1. PRD constrains Product TDD; `20-product-tdd` must explicitly realize PRD claims, workflows, and rules.
2. Product and Unit TDD may refine technical realization but must not reinterpret upstream drivers.
3. Deployment docs operationalize runtime truth and must remain consistent with PRD behavior commitments.

## Common local mistakes

1. Starting from pre-selected domains instead of product pressure.
2. Treating `domain-structure` as an upstream requirement source.
3. Using `domain-structure` files as a glossary replacement.
4. Mixing technical mechanism contracts into PRD files.
5. Leaving product claims without evaluation dimensions and evidence expectations.
6. Updating user-visible behavior only in task notes or code without PRD promotion.
7. Leaving canonical product or domain semantics only in meta docs instead of PRD.
