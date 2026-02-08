# AGENTS.md of Beluna

Beluna is an agent.

## Tech Stacks

- Language: Rust (2024 edition)

## Documentation System

Read following documents if needed, and keep them current:

- [Product Documents](./docs/product/overview.md)

> You are encouraged to add an AGENTS.md file under modules with significant complexity when needed.

## Coding Guidelines

- Avoid Loose protocol design.

## Current State

> Last Updated At: 2026-02-08T23:10Z+08:00

### Live Capabilities

- Load config (jsonc, with JSONSchema support)
- Start the core loop listening on an Unix Socket (NDJSON), exit on SIGTERM or exit message.
- AI Gateway module with canonical request/event model and strict normalization.
- Gateway adapters:
  - `openai_compatible` (`chat/completions`-like)
  - `ollama` (`/api/chat`)
  - `github_copilot_sdk` (Copilot language server over stdio JSON-RPC)
- Cross-cutting gateway concerns:
  - deterministic backend routing (no fallback)
  - capability guard
  - retry + exponential backoff + minimal circuit breaker
  - budget enforcement (timeout/concurrency/rate smoothing + best-effort usage accounting)
  - cancellation propagation on stream drop

### Known Limitations & Mocks

- Gateway is implemented as a module boundary and is not yet integrated into the Unix socket runtime protocol.
- Copilot adapter targets SDK/LSP flow with conservative assumptions; method/shape drift across SDK versions may require follow-up updates.
- No live provider-network integration tests in CI.

### Immediate Next Focus

- An interactive shell that bridges human and Beluna (use the Unix Socket), the very first UI of Beluna.
- Runtime protocol integration: expose AI Gateway inference through Beluna socket protocol and shell UI.
