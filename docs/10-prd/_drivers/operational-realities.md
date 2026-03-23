# Operational Realities

## Runtime Reality Inputs

1. Inference backends and network paths are variable; failures and latency spikes are normal operating conditions.
2. Endpoint transport lifecycles are imperfect; disconnect/reconnect behavior must be expected.
3. Incident response depends on high-quality telemetry and explicit failure semantics.
4. Recovery paths must work under bounded time and partial-service conditions.

## PRD Consequence

Product claims must be evaluable under non-ideal runtime conditions, not only in happy-path development scenarios.
