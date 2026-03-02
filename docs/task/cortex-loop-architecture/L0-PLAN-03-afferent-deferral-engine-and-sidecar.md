# L0 Plan 03 - Afferent Deferral Engine and Sidecar
- Task: `cortex-loop-architecture`
- Micro-task: `03-afferent-deferral-engine-and-sidecar`
- Stage: `L0`
- Date: `2026-03-01`
- Status: `DRAFT_FOR_APPROVAL`

## Objective
Introduce a deferral-only afferent scheduling engine with observe-only sidecar, controlled by Cortex Primary tools.

## Scope
1. Afferent remains MPSC at ingress.
2. Deferral-only rules:
- selector by `min_weight`
- selector by `fq-sense-id` regex.
3. Rule actions are `overwrite` and `reset` only (no remove API).
4. Deferred queue is FIFO and held until rule reset/overwrite permits release.
5. `max_deferring_nums` cap with oldest-first eviction and warning logs.
6. Observe-only sidecar stream for out-of-band observability.
7. Cortex Primary tool calls afferent control API directly.

## Current State
1. Afferent is sender gate only; no scheduler/rule engine/sidecar exists.
2. No runtime control surface from Primary to pathway.

## Target State
1. Afferent is policy-aware and deterministic.
2. Deferral backlog behavior is bounded and observable.
3. Rule lifecycle is tool-driven from Primary.

## Key Gaps
1. Rule storage, matching, and overwrite/reset semantics are missing.
2. Deferred FIFO buffer and eviction counters/logging are missing.
3. Sidecar channel abstraction and subscription lifecycle are missing.

## Risks
1. Regex-heavy matching can affect throughput without guardrails.
2. Large rule-driven deferral can cause bursty release behavior.

## L0 Exit Criteria
1. Rule precedence and overwrite/reset semantics are explicit.
2. Deferred FIFO + eviction behavior is deterministic and testable.
3. Sidecar stream purpose is strictly observe-only.
