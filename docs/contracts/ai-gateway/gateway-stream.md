# Gateway Stream Lifecycle Contract

## Boundary

`ChatRuntime` orchestrates capability checks, backend dispatch, resilience controls, adapters, and canonical gateway events.

## Scenarios

### Scenario: Retry before first output succeeds

- Given: backend adapter fails once with retryable transient error before emitting output
- Given: retry policy allows retry before first output
- When: `dispatch_complete` is called
- Then: request succeeds
- Then: adapter invocation count is `2`

### Scenario: No retry after output event

- Given: backend adapter emits output and then fails with retryable transient error
- When: streaming dispatch is consumed
- Then: request fails with `BackendTransient`
- Then: adapter invocation count is `1`

### Scenario: Stream starts with Started and ends with one terminal event

- Given: backend adapter emits output and then fails
- When: stream is consumed
- Then: first event is `Started`
- Then: exactly one terminal event is emitted (`Completed` or `Failed`)
- Then: in this failure path, terminal event is `Failed`

### Scenario: Usage reporting does not enforce gateway budget rejection

- Given: backend emits usage data
- Given: caller-side budget policy would consider it over budget
- When: response is finalized
- Then: gateway still returns output + usage
- Then: caller decides subsequent budget policy

### Scenario: Consumer drop cancels in-flight backend invocation

- Given: adapter invocation is active and exposes a cancellation handle
- Given: caller drops gateway stream before terminal event
- When: drop is observed by gateway runtime
- Then: adapter cancellation handle is called
- Then: acquired backend resilience lease resources are released
