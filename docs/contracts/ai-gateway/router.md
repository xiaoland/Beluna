# Backend Router Contract

## Boundary

`BackendRouter` selects exactly one backend for a `CanonicalRequest`.
No multi-backend fallback is allowed in MVP.

## Scenarios

### Scenario: Default backend is selected deterministically

- Given: gateway config with `default_backend = primary`
- Given: request has no `backend_hint`
- When: selection is performed
- Then: selection succeeds
- Then: selected backend id is `primary`

### Scenario: Explicit backend hint is selected without fallback

- Given: gateway config contains backend `secondary`
- Given: request has `backend_hint = secondary`
- When: selection is performed
- Then: selection succeeds
- Then: selected backend id is `secondary`

### Scenario: Unknown backend hint fails instead of falling back

- Given: gateway config does not contain backend `unknown`
- Given: request has `backend_hint = unknown`
- When: selection is performed
- Then: selection fails with `InvalidRequest`
- Then: the error message contains `no fallback`
