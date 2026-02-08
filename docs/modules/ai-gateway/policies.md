# Reliability and Budget Policies

## Retry Policy

Default policy: retry only before first output/tool event.

- Retry requires retryable canonical error and remaining attempts.
- No retry after output/tool events under default policy.
- Post-start retry requires explicit policy/capability support.

## Circuit Breaker

- Per-backend breaker state.
- Opens after configured counted transient failures.
- `CircuitOpen` returned while open window is active.

## Budget Policy

- Timeout bound (effective per request)
- Per-backend concurrency limit
- Per-backend rate smoothing
- Usage token post-check is best-effort accounting only
  - may influence future admission/telemetry
  - does not terminate in-flight stream

## Cancellation Semantics

- Consumer stream drop triggers adapter cancellation.
- Cancellation releases budget/concurrency resources.
- Consumer cancellation does not count as backend failure for breaker accounting.
