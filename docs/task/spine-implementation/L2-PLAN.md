# L2 Plan - Spine Implementation (Low-Level Design)
- Task Name: `spine-implementation`
- Stage: `L2` (low-level design)
- Date: `2026-02-12`
- Status: `DRAFT_FOR_APPROVAL`

This L2 is split into focused files so interfaces, data models, algorithms, adapter behavior, and tests can be reviewed independently.

## L2 File Index
1. `/Users/lanzhijiang/Development/Beluna/docs/task/spine-implementation/L2-PLAN-01-module-and-file-map.md`
- source/test/doc change map
- dependency and ownership boundaries

2. `/Users/lanzhijiang/Development/Beluna/docs/task/spine-implementation/L2-PLAN-02-core-interfaces-and-data-model.md`
- async spine interfaces
- route key, endpoint abstraction, capability catalog model
- affordance/capability canonical semantics

3. `/Users/lanzhijiang/Development/Beluna/docs/task/spine-implementation/L2-PLAN-03-routing-catalog-and-dispatch-algorithms.md`
- registry and table-lookup algorithms
- catalog derivation and cortex bridge
- execution/report assembly algorithms

4. `/Users/lanzhijiang/Development/Beluna/docs/task/spine-implementation/L2-PLAN-04-adapter-shells-unixsocket-websocket.md`
- UnixSocket adapter migration from `core/src/server.rs`
- WebSocket adapter low-level API
- runtime orchestration and backpressure model

5. `/Users/lanzhijiang/Development/Beluna/docs/task/spine-implementation/L2-PLAN-05-test-contract-and-doc-delta.md`
- contract-to-test matrix
- async migration test impact
- docs/contracts/modules/features update plan

## L2 Objective
Define exact interfaces, data structures, and algorithms for an async Spine where:
1. Spine core is transport-ignorant.
2. Routing is pure table lookup by (`affordance_key`, `capability_handle`).
3. Body endpoints register capabilities into Spine-owned catalog.
4. Cortex consumes catalog derived from Spine registration state.
5. UnixSocket and WebSocket adapters act as shells and isolate transport complexity.
6. Body Endpoint -> Spine ingress data is canonically named `Sense`.

## L2 Completion Gate
L2 is complete when:
1. async port signatures are unambiguous,
2. route/capability data contracts are complete,
3. routing/dispatch and failure semantics are deterministic,
4. adapter boundaries and server migration are explicit,
5. L3 can implement without architecture reinterpretation.

Status: `READY_FOR_REVIEW`
