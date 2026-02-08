# Beluna PRD Overview

## System definition

BELUNA is an agent that:

- turns user intentions into environment changes through iteration and feedback.

## Core structure

### 1. One mind

- One primary LLM
- Memory is central
- Responsible for reasoning, decisions, and iteration
- No multi-mind architecture.

### 2. Helpers (sub-agents)

> Exists since primary LLM is not powerful enough to drive the whole "BODY"

- Dispatched by the mind
- Reduce cognitive load
- Task-scoped and temporary
- Not independent agents

### 3. BELUNA framework (the “circulatory system”)

BELUNA’s code:

- routes data and control
- handles automation pipelines
- validates structured data (e.g., JSON)
- connects mind ↔ tools ↔ environment

Not infrastructure, not the LLM itself.

### 4. Body abstraction

The body is simply:

- Input ↔ Output
- Adapters implement this:
- shell
- filesystem
- GUI automation
- MCP / retrieval
- runtime tools

BELUNA stays environment-agnostic.

### 5. Feedback loop (core operating principle)

```ascii
Goal → Action → Environment change → Feedback → Next action
```

Or:

```ascii
BELUNA acts to generate feedback that improves understanding.
```

### 6. Natural language as protocol

Natural language is the interface for:

- human ↔ BELUNA
- LLM ↔ helper processes

Structured data remains execution-level artifacts.

### 7. Long-term idea (not required now)

BELUNA may eventually:

- modify its own framework
- test new versions
- migrate memory
- evolve safely

### 8. AI Gateway (MVP)

Beluna now includes a minimal AI Gateway module that standardizes inference calls across multiple backend dialects:

- `openai_compatible` (`chat/completions`-like protocol, graceful degradation on provider divergence)
- `ollama` (`/api/chat`)
- `github_copilot_sdk` (Copilot language server over stdio JSON-RPC)

Gateway guarantees and behavior in MVP:

- Deterministic backend routing (no multi-backend fallback)
- Strict request normalization and validation before adapter dispatch
- Canonical event stream (`Started` -> deltas/tool events/usage -> terminal event)
- Retry with exponential backoff under safe boundaries
  - default: retry only before first output/tool event
- Minimal per-backend circuit breaker
- Budget enforcement
  - timeout bound
  - per-backend concurrency limit
  - rate smoothing
  - token usage post-check is best-effort accounting only
- Cancellation propagation when stream consumer drops
