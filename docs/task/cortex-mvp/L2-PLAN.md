# L2 Plan - Cortex MVP (Low-Level Design)
- Task Name: `cortex-mvp`
- Stage: `L2` (low-level design)
- Date: `2026-02-11`
- Status: `DRAFT_FOR_APPROVAL`

This L2 is split into focused files so interfaces, data models, algorithms, and test contracts can be reviewed independently.

## L2 File Index
1. `/Users/lanzhijiang/Development/Beluna/docs/task/cortex-mvp/L2-PLAN-01-module-and-boundary-map.md`
- source/test/doc change map
- ownership and dependency direction

2. `/Users/lanzhijiang/Development/Beluna/docs/task/cortex-mvp/L2-PLAN-02-domain-model-and-contracts.md`
- reaction input/output contracts
- distributed intent context model
- `IntentAttempt`/feedback correlation rules

3. `/Users/lanzhijiang/Development/Beluna/docs/task/cortex-mvp/L2-PLAN-03-ports-and-ai-gateway-adapters.md`
- cognition organ interfaces
- real `ai_gateway` adapter design
- mock strategy for deterministic tests

4. `/Users/lanzhijiang/Development/Beluna/docs/task/cortex-mvp/L2-PLAN-04-reactor-algorithms-and-state-machines.md`
- reactor loop algorithm
- clamp and one-repair pipeline
- backpressure and ingress assembly rules

5. `/Users/lanzhijiang/Development/Beluna/docs/task/cortex-mvp/L2-PLAN-05-test-contract-and-doc-plan.md`
- BDD contract-to-test matrix
- test commands and rollout checks
- docs/contract update plan

## L2 Objective
Define exact interfaces, data structures, and algorithms for a reactor-only stateless Cortex where:
1. Cortex runs continuously and advances only via inbox events.
2. Primary LLM emits prose IR and bounded sub-LLM calls compile IR into attempts.
3. Deterministic clamp is final authority before attempts leave Cortex.
4. `IntentAttempt` is world-relative and correlation-safe (`attempt_id`, `based_on`, feedback correlation).
5. Business flow outputs remain clean (no telemetry payload pollution).
6. Real `ai_gateway` is used in runtime, while tests run against mocks.

## L2 Completion Gate
L2 is complete when:
1. Reactor contracts and inbox progression semantics are unambiguous.
2. Stateless Cortex semantics are mechanically enforceable.
3. Capability-driven routing and clamp rules are fully specified.
4. One-primary/N-subcall/one-repair/noop fallback behavior is executable without reinterpretation.
5. L3 can implement directly with no architecture redesign.

Status: `READY_FOR_REVIEW`
