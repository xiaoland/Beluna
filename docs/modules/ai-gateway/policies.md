# Resilience and Usage Policies

## Retry Policy

Default policy: retry only before first output/tool event.

- Retry requires retryable canonical error and remaining attempts.
- No retry after output/tool events under default policy.
- Post-start retry requires explicit policy/capability support.

## Circuit Breaker

- Per-backend breaker state.
- Opens after configured counted transient failures.
- `CircuitOpen` returned while open window is active.

## Resilience Admission Policy

- Timeout bound (effective per request)
- Per-backend concurrency limit
- Per-backend rate smoothing

## Usage Policy

- Gateway does not enforce token budget rejection.
- Gateway returns usage stats as output metadata.
- Caller-owned policy may consume usage for accounting/admission decisions.

## Cancellation Semantics

- Consumer stream drop triggers adapter cancellation.
- Cancellation releases resilience lease resources.
- Consumer cancellation does not count as backend failure for breaker accounting.
