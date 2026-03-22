# Scope

## Goals

1. Deliver a deterministic runtime loop with clear module boundaries.
2. Keep cognition state durable and guardrailed.
3. Support pluggable body endpoints with stable routing and protocol behavior.
4. Keep inference backend integration provider-agnostic through AI Gateway.
5. Keep deployment and runtime operations legible and observable.

## Non-Goals

1. No hidden multi-backend fallback for inference routing.
2. No duplication of Core runtime observability surfaces in body endpoints.
3. No backward-compatibility shims for stale config keys.
4. No task-plan documents as authoritative product/design truth.
