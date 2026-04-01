# Low-Level Design - Capability-First AI Gateway

## Status

Exploratory low-level design.
Non-authoritative.
No production code changes are implied by this file.

## Scope

This file narrows the architecture after the following decisions were made:

- `AI Gateway` is an AI capability runtime, not merely a thin provider gateway.
- Beluna canonical state remains the authority.
- `Cortex` keeps dialogue orchestration authority.
- `AI Gateway` must support multiple capabilities over time.
- Directory structure and dependency direction are capability-first.
- Capabilities and backends are added only when actually needed.

## Hard Conclusions

### 1. There is no valid top-level "unified capability surface"

`chat`, `asr`, and `tts` are not the same shape.

They may share:

- backend credentials
- transport helpers
- generic request execution policy
- error categories
- observability discipline

They must not be forced into one fake business API.

Therefore:

- `ai_gateway` may have one unified capability layer
- it must not have one fake unified capability request/response surface

### 2. Top-level organization must be capability-first

Target direction:

```text
core/src/ai_gateway/
  mod.rs
  shared/
  chat/
  asr/        # future, only when needed
  tts/        # future, only when needed
```

This means:

- top-level folders represent Beluna-owned semantic capabilities
- backend/provider concerns live below capability or inside shared infra
- backend additions are subordinate changes
- capability additions may define new canonical contracts

### 3. Beluna canonical state is capability-owned, not backend-owned

For every capability:

- Beluna owns canonical input and output contracts
- Beluna owns canonical state shape
- backend-native hosted state is only execution medium

For chat specifically:

- Beluna owns canonical conversation/thread/turn/tool state
- provider-native thread or prompt state may be used as cache/acceleration
- provider-native state must be reconstructable or discardable

### 4. `Cortex` must depend on capability contracts, not gateway internals

Current code violates this for chat:

- `Cortex` depends on `Chat`, `Thread`, `TurnInput`, `ToolOverride`, thread clone/reset behavior
- this means `Cortex` is coupled to the chat runtime's internal object model

Target rule:

- `Cortex` may depend on `ai_gateway::chat::contract`
- `Cortex` must not depend on provider transport details
- `Cortex` should also avoid depending on chat internal storage mechanics where possible

## Decisions After First LLD Review

### 1. Multiple capabilities use one shared provider inventory

Decision:

- chat / asr / tts and future capabilities should reference one shared provider inventory

Accepted, with one strict constraint:

- shared provider inventory must not become one giant mixed config object containing every capability's target fields

That would harm readability quickly.

Therefore the correct split is:

- shared provider inventory = connection/auth/provider-family facts
- capability-local binding config = capability target selection and feature mapping

### 2. `Cortex` calls high-level OOP-style chat operations

Decision:

- `Cortex` should call high-level operations that hide thread details
- API style should remain object-oriented

Accepted.

Important clarification:

- OOP style does not justify exposing low-level chat storage objects
- the object seen by `Cortex` should be a narrowed Beluna-owned chat thread handle, not raw provider or storage internals
- the ordinary write path should be thread-centric rather than turn-centric

### 3. Conversation state stays entirely inside AI Gateway

Decision:

- conversation state ownership stays inside AI Gateway

Accepted only in the following form:

- canonical chat state ownership is internal to AI Gateway
- but AI Gateway must expose read/export/restore surfaces for observability and recovery

Rejected interpretation:

- opaque black-box state with no canonical snapshot/export boundary

That interpretation would damage maintainability and would conflict with Beluna-owned authority and observability requirements.

## Proposed Module Shape

### Top level

```text
core/src/ai_gateway/
  mod.rs
  shared/
    mod.rs
    credentials.rs
    error.rs
    transport/
      mod.rs
      http.rs
      json_rpc.rs
      stream.rs
    execution/
      mod.rs
      request_policy.rs
      concurrency.rs
      circuit_breaker.rs
      rate_limit.rs
  chat/
    mod.rs
    contract.rs
    config.rs
    runtime.rs
    routing.rs
    observability.rs
    api.rs
    state/
      mod.rs
      thread.rs
      turn.rs
      tool.rs
    backends/
      mod.rs
      bridge.rs
      openai_responses/
      openai_chat_completions/
      ollama/
      github_copilot/
```

Notes:

- `asr/` and `tts/` do not exist until needed.
- `shared/` contains only capability-neutral infrastructure.
- `chat/backends/` is capability-specific backend bridging, not the universal home of every provider concern in the whole gateway.

## Dependency Rules

### Allowed direction

```text
Cortex
  -> ai_gateway::chat::contract
  -> ai_gateway::chat::runtime   # only if contract alone is insufficient during transition

ai_gateway::chat::runtime
  -> ai_gateway::chat::state
  -> ai_gateway::chat::routing
  -> ai_gateway::chat::backends
  -> ai_gateway::shared

ai_gateway::chat::backends::*
  -> ai_gateway::chat::contract
  -> ai_gateway::shared

ai_gateway::shared
  -> no capability modules
```

### Forbidden direction

- `shared` must not depend on `chat`
- future `asr` or `tts` must not depend on `chat`
- `Cortex` must not depend directly on `chat/backends/*`
- provider-specific SDK code must not define Beluna canonical semantics

## Configuration Shape

The current model-centric backend config is too chat-specific to serve as the long-term top-level shape.

The next config direction should be:

```text
ai_gateway/
  providers/            # shared provider inventory
  chat/                 # capability-local config
  asr/                  # future
  tts/                  # future
```

### Shared provider inventory

Shared provider inventory should contain only provider-level facts such as:

- provider id
- provider family / dialect
- endpoint or SDK bootstrap details
- credential reference
- coarse declared capability families

It should not directly become the home for:

- chat model routing
- tts voice selection
- asr decoding policy
- provider-native prompt object ids used only by one capability

### Capability-local binding config

Each capability defines its own binding surface.

For chat, that means:

- route aliases
- provider reference
- capability target such as model
- capability-native feature assumptions
- Beluna fallback policy where backend-native support is partial

This means the answer to the user's parenthetical question is:

- yes, config must change
- yes, the top-level shared provider inventory should stop being model-centric
- but capability-local bindings may still be model-centric where chat actually needs models

That distinction preserves readability.

## What Moves Out of the Current Top Level

The current top-level `ai_gateway` contains several things that are only "generic" by name.

### `router.rs`

Current problem:

- route selection is backend/model oriented
- there is no capability dimension
- current design assumes one global routing space

Target:

- routing belongs first to each capability
- shared backend inventory may exist later, but capability chooses how it routes

So:

- global router should not remain the primary semantic routing owner
- chat gets its own routing module first

### `telemetry.rs`

Current problem:

- top-level request telemetry is valid only for capability-neutral transport
- chat-specific turn/thread telemetry already exists elsewhere

Target:

- transport request lifecycle helpers may live in `shared`
- capability telemetry belongs in the capability namespace

### `types.rs`

Current problem:

- this file mixes provider/dialect concepts with chat-specific config and feature flags

Examples of chat-biased types currently pretending to be generic:

- `ChatConfig`
- `BackendCapabilities` fields such as `tool_calls`, `parallel_tool_calls`, `json_mode`, `json_schema_mode`, `vision`

Those are not future-proof generic gateway features.
They are mostly chat capability feature flags.

Target:

- capability-neutral backend identity/config types may stay shared
- chat-specific feature flags move under `chat`
- future `asr` or `tts` define their own backend feature contracts when needed

## Critical Low-Level Correction: Current "Generic" Resilience Is Not Fully Generic

`ResilienceEngine` is only partially generic today.

Problem:

- `can_retry(...)` already knows about:
  - emitted output
  - emitted tool activity
  - resumable streaming
  - tool retry safety

These are chat/runtime semantics, not universally valid gateway semantics.

Therefore the correct split is:

- shared execution policy owns:
  - timeout shaping
  - concurrency permits
  - circuit breaker
  - rate smoothing
  - generic retry budget and backoff

- capability/backend bridge owns:
  - what counts as irreversible progress
  - whether partial output can be retried
  - whether tool activity can be retried
  - provider-specific resumability semantics

This matters because otherwise future `asr` and `tts` will be forced through chat-derived retry concepts.

## Chat Capability Contract

Chat needs a capability-local public contract.

Minimal public responsibility:

- canonical chat request and response types
- canonical conversation state model
- canonical tool contract model
- capability-facing route selection entry
- capability-facing execution entry used by `Cortex`

This contract must be:

- Beluna-owned
- provider-agnostic
- reconstructable from Beluna state without requiring provider-native hidden state

### Route contract note

The canonical route syntax should be globally uniform even if some call sites are already inside one capability scope.

Use:

- `<capability>.<alias>`

Reason:

- one route grammar is simpler than maintaining local-vs-global route exceptions
- the small redundancy cost is acceptable
- config, logs, telemetry, and debugging all benefit from one stable syntax

### OOP-facing chat API

After review of the current code, a new `ChatDialogue` layer is probably unnecessary.

Reason:

- `Thread / Turn / Message` is already a meaningful semantic stack for chat
- adding `ChatDialogue` on top would likely be a naming-only wrapper unless it carries genuinely different semantics

Therefore the better direction is:

- keep `Thread / Turn / Message` as the canonical chat abstraction stack
- narrow the public `Thread` API so it behaves like a capability object rather than a storage surgery surface

What `Cortex` should see:

- `ChatCapability`
- `Thread`
- `Thread::advance(...)`
- `Thread::rewrite_context(...)`
- `Thread::snapshot()`

What `Cortex` should not see:

- direct thread storage mutation helpers
- direct external turn cloning / append surgery
- backend/provider-native thread handles

The important design point is:

- object-oriented style is fine
- extra naming layers without new semantics are not

## Chat Backend Bridge Contract

`chat/backends/*` should not be pure wire adapters only.

They should be capability-aware bridges that answer:

- can this backend execute canonical chat turn requests?
- which chat features does it support natively?
- which chat features require Beluna-local fallback or simulation?
- which provider-managed states may be used as execution media?
- what provider-native state must remain non-authoritative?

This is the correct place for backend-native hosted capability handling.
It is more precise than calling everything simply "adapter".

## Canonical vs Provider-Native State Rules

For chat:

Beluna canonical state includes:

- thread identity as Beluna understands it
- turn ordering
- committed message history
- tool call and tool result semantics
- reset and rewrite semantics

Provider-native state may include:

- hosted thread ids
- hosted prompt objects
- provider-side tool/mcp sessions
- provider-side truncation or caching aids

Rule:

- provider-native state may accelerate execution
- provider-native state must never be required to explain Beluna's own semantics

### Internal ownership rule

Because conversation state is owned inside AI Gateway:

- chat state store lives inside the chat capability runtime
- `Cortex` does not directly mutate that store
- observability and recovery use exported canonical snapshots / replayable state

This is the only form of "internal ownership" that remains compatible with authority choice `A`.

## Capability Admission Rule

A new capability may be added only when all of the following are explicit:

1. canonical request and response contract
2. canonical state model, if the capability is stateful
3. capability-local observability requirements
4. backend bridge rules
5. what stays shared versus what stays capability-local

If these are not explicit, the capability should not be added yet.

## Backend Admission Rule

A new backend for one capability may be added only when all of the following are explicit:

1. which capability it serves
2. which canonical features it supports natively
3. which features require Beluna-local fallback
4. what provider-native state exists
5. why that provider-native state remains non-authoritative

If these are not explicit, the backend should not be added yet.

## Immediate Refactoring Target for Existing Code

Without changing behavior yet, the current `chat` implementation should become visibly capability-owned.

Minimum target shape:

- make `chat` the clearly primary implemented capability
- remove the illusion that current top-level gateway types are all capability-neutral
- stop letting `chat` semantics hide inside "generic" top-level types and policy code

## Remaining Open Questions

### 1. How much canonical chat state must be restorable through Continuity?

Conversation state is now considered AI-Gateway-internal.
Still open:

- whether all canonical chat state participates in broader persistence
- or whether some chat state is intentionally wake-local and reconstructable from other sources

### 2. Where exactly is the boundary between capability runtime and backend bridge?

Still open:

- which fallback logic belongs in `chat/runtime`
- which provider-specific feature-mapping belongs in `chat/backends/*`

### 3. How should shared provider capability-family declaration be represented?

Still open:

- whether the provider inventory declares coarse supported capability families only
- or also declares provider-native feature bundles that capability bindings may reference
