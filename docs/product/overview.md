# Beluna PRD Overview

## System definition

BELUNA is an agent that:

- turns user intentions into environment changes through iteration and feedback.

## Core Design

### 1. Architecture

#### Layer 1 — AI Backend (Provider/Model)

- adapters: OpenAI-compatible / Ollama / Copilot SDK (MVP)
- uniform canonical request schema
- uniform canonical event stream schema

#### Layer 2 — Beluna Runtime (non-AI substrate)

- helpers (sub-agents)
  - dispatched by the mind
  - reduce cognitive load
  - task-scoped and temporary
  - not independent agents
- body abstraction (input/output adapters)
  - shell
  - filesystem
  - GUI automation
  - MCP / retrieval
  - runtime tools

#### Layer 3 — Mind (pluggable)

- orchestrates, evaluates, and evolves runtime behavior
- can call Layer 1 via AI Gateway
- one SOTA LLM now, non-LLM options possible later
- implemented as internal `src/mind/*` control core in MVP
- does not interact with Unix socket protocol/runtime directly

### 2. Feedback loop

```ascii
Goal -> Action -> Environment change -> Feedback -> Next action
```

### 3. Natural language as protocol

Natural language is the interface for:

- human <-> BELUNA
- LLM <-> helper processes

### 4. AI Gateway (MVP)

Beluna includes a minimal AI Gateway module with deterministic routing, strict normalization, canonical streaming, reliability controls, and budget enforcement.

Detailed feature PRD:

- `docs/features/ai-gateway/PRD.md`

Design docs:

- `docs/features/ai-gateway/HLD.md`
- `docs/features/ai-gateway/LLD.md`

Glossary note:

- top-level terms: `docs/product/glossary.md`
- AI Gateway domain terms: `docs/features/ai-gateway/glossary.md`

### 5. Mind Layer (MVP)

Mind MVP includes:

- explicit in-process `MindState` continuity model,
- `GoalManager` single-active-goal invariants,
- preemption dispositions (`pause`, `cancel`, `continue`, `merge`) with safe point/checkpoint data,
- trait-based delegation and memory policy ports,
- deterministic conflict resolution,
- proposal-only evolution decisions.

Design docs:

- `docs/features/mind/PRD.md`
- `docs/features/mind/HLD.md`
- `docs/features/mind/LLD.md`

### 6. Long-term direction

- framework self-modification and safe evolution remain long-term ideas, not MVP requirements.
