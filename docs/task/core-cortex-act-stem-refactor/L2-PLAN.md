# L2 Plan - Core Cortex Act Stem Refactor (Low-Level Design)
- Task Name: `core-cortex-act-stem-refactor`
- Stage: `L2` (low-level design)
- Date: `2026-02-13`
- Status: `DRAFT_FOR_APPROVAL`

This L2 is split into focused files so interfaces, data models, algorithms, queue/shutdown behavior, and migration/tests can be reviewed independently.

## L2 File Index
1. `/Users/lanzhijiang/Development/Beluna/docs/task/core-cortex-act-stem-refactor/L2-PLAN-01-module-and-file-map.md`
- source/test/doc delta map
- ownership and dependency boundaries

2. `/Users/lanzhijiang/Development/Beluna/docs/task/core-cortex-act-stem-refactor/L2-PLAN-02-domain-model-and-interfaces.md`
- canonical `Sense`/`PhysicalState`/`CognitionState`/`Act` contracts
- stage interfaces for Ledger/Continuity/Spine
- Cortex function boundary contract

3. `/Users/lanzhijiang/Development/Beluna/docs/task/core-cortex-act-stem-refactor/L2-PLAN-03-stem-and-dispatch-algorithms.md`
- Stem loop algorithm
- control-sense interception rules
- serial per-act dispatch with `Continue | Break`

4. `/Users/lanzhijiang/Development/Beluna/docs/task/core-cortex-act-stem-refactor/L2-PLAN-04-queue-shutdown-and-wire-protocol.md`
- bounded MPSC/backpressure mechanics
- ingress gating and blocking `sleep` enqueue shutdown sequence
- wire/protocol changes for capability patch senses

5. `/Users/lanzhijiang/Development/Beluna/docs/task/core-cortex-act-stem-refactor/L2-PLAN-05-test-contract-and-doc-migration.md`
- contract-to-test matrix
- delete/replace test plan
- docs/features/contracts/modules/glossary migration plan

## L2 Objective
Define exact interfaces, data structures, and algorithms for runtime flow:
1. `Sense -> Cortex -> Act[] -> Ledger -> Continuity -> Spine` (serial, inline in Stem).
2. Admission removed completely.
3. Cortex is stateless and side-effect free at component boundary.
4. `CognitionState` is persisted in Continuity.
5. bounded MPSC is the only sense queue, with blocking senders.
6. shutdown injects `sleep` after ingress gating and blocks until enqueue succeeds.

## L2 Completion Gate
L2 is complete when:
1. all replacement interfaces are unambiguous,
2. serial dispatch/break semantics are executable without reinterpretation,
3. ingress gating + shutdown mechanics are precise,
4. admission deletion blast radius is fully mapped,
5. L3 can execute directly with no architecture redesign.

Status: `READY_FOR_REVIEW`
