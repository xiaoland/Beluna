# Usage Reporting Contract

## Boundary

AI Gateway returns usage data from adapters but does not reject requests by token budget policy.

## Scenarios

### Scenario: Missing usage does not fail request

- Given: backend response has no usage object
- When: gateway normalizes response
- Then: request can still complete successfully
- Then: usage field is `None`

### Scenario: Usage is returned when backend provides it

- Given: backend response includes usage details
- When: gateway normalizes response
- Then: usage appears in `TurnResponse`
- Then: caller can consume usage for external budget/accounting policy

### Scenario: Usage over caller budget does not auto-abort in gateway

- Given: caller has stricter external budget policy
- Given: gateway receives usage that exceeds caller policy
- When: gateway finalizes response
- Then: gateway still returns response + usage
- Then: caller decides subsequent admission or rejection behavior
