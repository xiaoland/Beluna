# Gateway Stream Lifecycle Contract

## Boundary

`AIGateway` orchestrates normalization, routing, reliability, adapters, and emits canonical gateway events.

## Scenarios

### Scenario: Retry before first output succeeds

- Given: backend adapter fails once with retryable transient error before emitting output
- Given: retry policy allows retry before first output
- When: `infer_once` is called
- Then: request succeeds
- Then: adapter invocation count is `2`

### Scenario: No retry after output event

- Given: backend adapter emits output and then fails with retryable transient error
- When: `infer_once` is called
- Then: request fails with `BackendTransient`
- Then: adapter invocation count is `1`

### Scenario: Stream starts with Started and ends with one terminal event

- Given: backend adapter emits output and then fails
- When: `infer_stream` is consumed
- Then: first event is `Started`
- Then: exactly one terminal event is emitted (`Completed` or `Failed`)
- Then: in this failure path, terminal event is `Failed`

### Scenario: Usage over budget in post-check does not terminate active stream

- Given: backend emits a usage event whose total tokens exceed configured usage budget
- Given: backend then emits output and `Completed`
- When: `infer_once` is called
- Then: request still completes successfully
- Then: usage is observed as best-effort accounting only

### Scenario: Consumer drop cancels in-flight backend invocation

- Given: adapter invocation is active and exposes a cancellation handle
- Given: caller drops gateway stream before terminal event
- When: drop is observed by gateway runtime
- Then: adapter cancellation handle is called
- Then: acquired backend budget/concurrency resources are released
