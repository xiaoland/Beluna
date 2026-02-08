# Architecture

## High-Level Components

- `AIGateway`: facade entry point (`infer_stream`, `infer_once`)
- `RequestNormalizer`: strict input validation + canonical mapping
- `BackendRouter`: deterministic backend/model selection
- `CredentialProvider`: secret resolution boundary
- `CapabilityGuard`: requested feature validation against backend capabilities
- `BudgetEnforcer`: timeout/concurrency/rate policy
- `ReliabilityLayer`: retry/backoff + per-backend circuit breaker
- `BackendAdapter`: transport + dialect mapping
- `ResponseNormalizer`: backend raw event -> canonical event
- `TelemetrySink`: lifecycle/attempt/outcome events

## Layering Rule

- Gateway core is backend-neutral.
- Backend-specific protocol and transport logic lives only in adapters.
- Contracts and tests validate boundary behavior independent of provider SDKs.
