# Testing and Contracts

## Test Surfaces

- Unit tests under `tests/ai_gateway/*`
- Runtime dialect protocol tests under `src/spine/adapters/unix_socket.rs`

## Contract Set

Behavioral contracts:

- `docs/contracts/ai-gateway/chat-invariants.md`
- `docs/contracts/ai-gateway/router.md`
- `docs/contracts/ai-gateway/resilience.md`
- `docs/contracts/ai-gateway/usage.md`
- `docs/contracts/ai-gateway/adapters.md`
- `docs/contracts/ai-gateway/gateway-stream.md`

## Contract Intent

Contracts define testable boundaries and deterministic behaviors for gateway components independent of specific provider availability.
