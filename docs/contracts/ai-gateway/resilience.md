# Resilience Engine Contract

## Boundary

`ResilienceEngine` decides retry eligibility, circuit-breaker transitions, timeout bounds, and per-backend concurrency/rate admission.

## Scenarios

### Scenario: Retry allowed before first output

- Given: a retryable transient backend error
- Given: `attempt < max_retries`
- Given: no output and no tool event emitted yet
- When: retry eligibility is evaluated
- Then: retry is allowed

### Scenario: Retry denied after output under default policy

- Given: a retryable transient backend error
- Given: at least one output event has been emitted
- Given: retry policy is `BeforeFirstEventOnly`
- When: retry eligibility is evaluated
- Then: retry is denied

### Scenario: Retry denied after tool event when adapter is not tool-safe

- Given: a retryable transient backend error
- Given: at least one tool event has been emitted
- Given: adapter does not support tool retry safety
- When: retry eligibility is evaluated
- Then: retry is denied

### Scenario: Circuit opens after threshold transient failures

- Given: breaker failure threshold is `N`
- Given: the same backend records `N` counted failures
- When: backend admission is checked
- Then: admission fails with `CircuitOpen`

### Scenario: Per-backend concurrency blocks parallel dispatch over limit

- Given: configured `max_concurrency_per_backend = 1`
- Given: first request acquires a lease for backend `b1`
- When: a second request for backend `b1` is dispatched before release
- Then: second request does not acquire a lease within the short timeout window
- Then: releasing the first lease unblocks further work
