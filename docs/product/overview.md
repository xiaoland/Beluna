# Beluna PRD Overview

## System definition

BELUNA is an agent that:

- turns user intentions into environment changes through iteration and feedback.

## Core Design

> NEED FURTHER ADJUSTMENTS

### 1. Architecture

#### Layer 1 — AI Backend (Provider/Model)

- adapters: OpenAI/Anthropic/Gemini/local
- uniform request schema
- uniform event stream schema

#### Layer 2 — Beluna Runtime (non-AI substrate)

- helpers (sub-agents) (Exists since primary LLM is not powerful enough to drive the whole "BODY")
  - Dispatched by the mind
  - Reduce cognitive load
  - Task-scoped and temporary
  - Not independent agents
- The Body: Input ↔ Output, makes BELUNA stays environment-agnostic, connects mind ↔ tools ↔ environment.
  - shell
  - filesystem
  - GUI automation
  - MCP / retrieval
  - runtime tools

Not infrastructure, not the LLM itself.

#### Layer 3 — Mind (pluggable)

- a meta-level control system that orchestrates, evaluates, and evolves the whole intelligent runtime.
- can call Layer 1
- can be non-LLM later (one SOTA LLM now)

### 2. Feedback loop (core operating principle)

```ascii
Goal → Action → Environment change → Feedback → Next action
```

Or:

```ascii
BELUNA acts to generate feedback that improves understanding.
```

### 3. Natural language as protocol

Natural language is the interface for:

- human ↔ BELUNA
- LLM ↔ helper processes

Structured data remains execution-level artifacts.

### 4. AI Gateway (MVP)

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

### 5. Long-term idea (not required now)

BELUNA may eventually:

- modify its own framework
- test new versions
- migrate memory
- evolve safely
