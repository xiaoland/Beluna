# Reliability Layer Contract

## Boundary

`ReliabilityLayer` decides retry eligibility and circuit-breaker state transitions.

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

### Scenario: Non-counted failures do not open circuit

- Given: breaker failure threshold is `1`
- Given: a backend records a failure with `count_toward_breaker = false`
- When: backend admission is checked
- Then: admission succeeds
- Then: circuit remains closed for that backend
