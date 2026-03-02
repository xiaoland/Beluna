# L1 Plan 03 - Afferent Deferral Engine and Sidecar
- Task: `cortex-loop-architecture`
- Micro-task: `03-afferent-deferral-engine-and-sidecar`
- Stage: `L1`
- Date: `2026-03-01`
- Status: `DRAFT_FOR_APPROVAL`

## High-Level Strategy
1. Keep current MPSC ingress model and add a deterministic rule-driven deferral layer before delivery to Cortex.
2. Support only deferral (no drop/suppress).
3. Provide a rule-control port owned by the Afferent Pathway; Cortex Primary wraps it as a rule overwrite/reset tool.
4. Add observe-only sidecar streaming for out-of-band inspection.

## Architectural Design
1. Afferent pathway internal flow:
- ingress queue receives senses.
- active deferral rules evaluate each sense.
- matched senses move to deferred FIFO buffer.
- unmatched senses flow to Cortex consumer queue.
2. Rule selectors:
- `min_weight`
- `fq-sense-id` regex.
 - `min_weight` match semantics: defer when `sense.weight < min_weight`.
3. Rule lifecycle:
- `overwrite`: upsert exactly one rule by `rule_id` atomically.
- `reset`: clear all rules atomically.
4. Deferred buffer policy:
- FIFO retention while blocked.
- capacity `max_deferring_nums`.
- oldest-first eviction on overflow with warning telemetry.
5. Sidecar behavior:
- receives observation events (rule-hit, deferred, released, evicted).
- cannot mutate queues or rules.

## Key Technical Decisions
1. Rule operations are versioned atomic updates to avoid partially applied state.
2. Release behavior is deterministic after each overwrite/reset operation.
3. Regex compilation failures are rejected at control API boundary.
4. Eviction policy is explicit FIFO destruction, never silent discard.
5. Afferent pathway module is owned under Stem namespace (`stem::afferent_pathway`).

## Dependency Requirements
1. Micro-task `01` must provide runtime handles so Cortex can own afferent receiving independent of Stem loop.
2. Micro-task `02` should land first for `weight`-based rule matching.
3. Observability schema updates are required so sidecar events are queryable.

## L1 Exit Criteria
1. Deferral-only semantics are explicit and complete.
2. Single-rule overwrite + full reset control plane is defined.
3. Sidecar scope is fixed as observe-only.
