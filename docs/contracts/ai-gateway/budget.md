# Budget Enforcer Contract

## Boundary

`BudgetEnforcer` performs pre-dispatch budget checks and enforces per-backend concurrency/rate limits.

## Scenarios

### Scenario: Pre-dispatch rejects request token budget overflow

- Given: configured `max_usage_tokens_per_request = M`
- Given: request `max_output_tokens = M + 1`
- When: `pre_dispatch` is called
- Then: it fails with `BudgetExceeded`

### Scenario: Per-backend concurrency blocks parallel dispatch over limit

- Given: configured `max_concurrency_per_backend = 1`
- Given: first request acquires a lease for backend `b1`
- When: a second request for backend `b1` is dispatched before release
- Then: second request does not acquire a lease within the short timeout window
- Then: releasing the first lease unblocks further work
