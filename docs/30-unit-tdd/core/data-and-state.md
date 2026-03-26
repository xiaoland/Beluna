# Core Data And State

## Owned State

1. Cognition continuity state and persistence metadata owned by `continuity`.
2. Runtime physical state and descriptor/proprioception state owned inside `stem` pathways.
3. Dispatch lifecycle state and terminal outcome production owned by `spine`.
4. Runtime configuration view after typed config validation at the core boundary.

## Consumed State

1. Endpoint-originated senses and registration payloads over the endpoint protocol contract.
2. AI-provider responses through `ai_gateway` integration boundaries.
3. Process/runtime environment signals used for startup, shutdown, and recovery behavior.

## Local Invariants

1. Continuity persistence and restore behavior remains deterministic and guardrailed.
2. Each dispatch attempt yields one explicit terminal outcome (`Acknowledged`, `Rejected`, or `Lost`).
3. Authoritative runtime state mutation remains inside `core`; external units interact only through contracts.

## Authority Boundaries

1. `core` is authoritative for cognition persistence, runtime physical state, dispatch routing/outcomes, and runtime observability export policy.
2. Endpoint units (`cli`, `apple-universal`) are authoritative only for endpoint-local UX/app state.
3. Any change to cross-unit authority ownership must escalate to `docs/20-product-tdd/system-state-and-authority.md`.

## Failure-Sensitive Assumptions

1. Persistence and recovery paths can fail; failure handling must remain bounded and explicit.
2. Endpoint disconnects and transport failures are expected; dispatch/outcome semantics must remain intact under failure.
3. Invalid config is expected at boundaries; startup validation must fail closed rather than continue with implicit fallback.
