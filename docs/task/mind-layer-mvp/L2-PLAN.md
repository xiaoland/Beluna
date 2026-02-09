# L2 Plan - Mind Layer MVP (Low-Level Design)

- Task Name: `mind-layer-mvp`
- Stage: `L2` (low-level design)
- Date: 2026-02-08
- Status: `DRAFT_FOR_APPROVAL`

This L2 is intentionally split into multiple files to keep each design surface isolated and reviewable.

## L2 File Index

1. `docs/task/mind-layer-mvp/L2-PLAN-01-module-and-boundary-map.md`
- concrete source file map
- dependency-direction and coupling boundaries

2. `docs/task/mind-layer-mvp/L2-PLAN-02-domain-model-and-invariants.md`
- core types (`MindState`, goals, safe point, decisions, events)
- invariant set and state machine

3. `docs/task/mind-layer-mvp/L2-PLAN-03-ports-and-policy-contracts.md`
- trait contracts for delegation and memory policy
- preemption/evaluation/conflict/evolution policy interfaces

4. `docs/task/mind-layer-mvp/L2-PLAN-04-deterministic-loop-and-algorithms.md`
- deterministic `MindFacade` loop
- preemption, merge, conflict, and evolution algorithms

5. `docs/task/mind-layer-mvp/L2-PLAN-05-test-contract-and-doc-plan.md`
- BDD contracts
- test matrix and document updates

## L2 Objective

Specify exact interfaces, data structures, and algorithms for a strict Mind core that:

- never depends on Unix socket protocol/runtime,
- enforces single-active-goal invariants,
- performs policy-bounded preemption with explicit dispositions,
- uses trait-based helper and memory policy ports,
- emits deterministic typed decisions/events,
- keeps evolution proposal-only.

## L2 Completion Gate

L2 is complete when:

- all referenced L2 files are approved,
- interface signatures and ownership boundaries are unambiguous,
- deterministic loop and policy behavior are reviewable and testable,
- L3 can be written as a direct execution roadmap without redefining design.

Status: `READY_FOR_REVIEW`
