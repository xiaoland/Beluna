# Issue 17 - AI Gateway Architecture Discussion

## Context

- GitHub issue: `#17 Simplify AI Gateway adapter abstraction and resilience model`
- User concern:
  - AI Gateway module has serious code-organization defects.
  - There is no clear balance between AI Capability and Backend Adapter.
  - Discussion should stay in exploration mode and avoid product code changes.

## Intended Change

- Build a grounded discussion baseline from product docs, Cortex needs, AI Gateway code, and historical task notes.
- Identify which abstractions are real ownership boundaries and which are naming-only abstractions.
- Prepare a maintainability-first direction without prematurely locking implementation details.
- Split emerging low-level design into a separate working file once the discussion moves beyond high-level architecture.

## Working Notes

### User clarification after first round

User clarified the intended direction:

- `AI Gateway` should be understood more like an AI SDK / AI capability runtime.
- `Cortex` should continue to own dialogue orchestration.
- `AI Gateway` should provide reusable building blocks rather than owning the whole orchestration.
- `Backend Adapter` is not only API-dialect mapping, because some backends expose higher-level managed capabilities such as tools, MCP, prompt management, or similar hosted runtime features.

This clarification changes the discussion in one important way:

- The main problem is no longer "why is AI Gateway thicker than a transport gateway?"
- The main problem becomes "how thick can AI Gateway be before it starts stealing orchestration authority from Cortex?"

### User decision on authority

User selected:

- `A`: Beluna canonical conversation state remains the authority.
- Backend-native conversation / prompt / tool state is only an execution medium, never the ultimate source of truth.

This decision is maintainability-favorable, but only if enforced rigorously.
If not enforced, the codebase will drift into "Beluna claims authority while provider state is the real authority in practice."

### Additional clarification: AI Gateway is multi-capability, but there is no unified capability layer yet

User explicitly clarified:

- `AI Gateway` should be understood as a home for multiple AI capabilities.
- Examples include chat, ASR, TTS, and potentially others later.
- The current problem is that there is no unified capability surface / layer.

Current repo evidence:

- `core/src/ai_gateway/mod.rs` exposes one wide module bucket.
- The only actual capability subtree today is `chat/`.
- There is no sibling capability subtree such as `asr/`, `tts/`, or any capability-neutral abstraction above `chat/`.
- Historical task notes mention future ASR/TTS expansion only as a possibility, not as implemented structure.

Implication:

- Today's `ai_gateway` is not "multi-capability runtime with one missing polish layer".
- It is effectively "chat capability runtime inside a prematurely generic top-level module name".

### User decision on module direction

User decided:

- use capability-first directory structure and dependency direction
- add capabilities and backends only when actually needed
- do not pursue backend-first organization

This is the correct simplifying bias for the current repo stage.

Immediate architectural consequence:

- the primary top-level axis inside `ai_gateway` should be capability, not backend dialect
- backend implementations become subordinate implementation details under capability-owned or bridge-owned layers
- adding a new backend should never force redefining the system's capability model
- adding a new capability may define new canonical contracts without forcing fake unification with chat

### Issue statement (source-grounded)

Issue `#17` currently says:

- `attempt` is still an unclear concept in the AI Gateway resilience model.
- retry / budget / reliability concerns are not yet cleanly consolidated.
- a Tool or adapter seam may help observability only if it removes ambiguity rather than adding another layer.

### Authoritative product / system constraints

From docs:

- Product capabilities are implementation-agnostic and must not be confused with module decomposition.
- Inside `core`, `ai_gateway` is described as `provider-agnostic inference`.
- Observability requires a hard split between:
  - capability-neutral gateway transport (`ai-gateway.request`)
  - chat-capability semantics (`ai-gateway.chat.*`)

Implication:

- Any design that collapses backend transport and chat/thread/tool semantics into one indistinct layer is already fighting the observability contract.

### Current AI Gateway structure

Current `core/src/ai_gateway` effectively contains three different concerns:

1. Backend transport and backend policy
- router
- credentials
- resilience
- backend adapters
- transport telemetry

2. Chat capability runtime
- `chat/runtime.rs`
- `chat/thread.rs`
- `chat/turn.rs`
- `chat/tool_scheduler.rs`
- `chat/executor.rs`

3. In-memory chat state ownership
- `Chat` owns a thread registry
- `Thread` owns turn/message accumulation
- clone / query / snapshot behavior lives inside the gateway

This is not just a provider gateway anymore. It is already a chat-capability subsystem.

### Current Cortex dependency shape

`Cortex` does not treat AI Gateway as a small inference port.

Instead, `core/src/cortex/runtime/primary.rs` directly depends on:

- `Chat`
- `Thread`
- `ThreadOptions`
- `CloneThreadOptions`
- `TurnInput`
- `ToolExecutor`
- `ToolOverride`
- chat message and turn types

And Cortex directly orchestrates:

- opening the primary thread
- cloning selected turns into a new thread
- passing dynamic tool overrides
- acting as the tool executor
- resetting primary thread state when message history becomes invalid

Implication:

- The boundary is not `Cortex -> AI service`.
- The real boundary today is closer to `Cortex -> Chat thread engine inside AI Gateway`.

### Important challenge to the framing itself

`AI Capability` and `Backend Adapter` are not naturally symmetrical concepts.

- Capability is a semantic boundary.
- Adapter is an integration boundary.

So the goal should not be to make them feel equally weighted or structurally parallel.
The goal should be to make the dependency direction explicit:

- capability layer depends on transport/adapters
- adapters do not define capability semantics

If the design tries to "balance" them as peers, it will usually produce one of two failures:

- capability becomes too implicit
- adapters absorb semantics they should never own

With the user's clarification, a refined version is:

- capability runtime may legitimately be the center of gravity
- but backend adapters still must not become the place where Beluna-level orchestration semantics are defined

### Why the current organization feels wrong

#### 1. Name / ownership drift

Docs say `ai_gateway` is provider-agnostic inference.
Code says the public surface is `Chat`, with thread and turn lifecycle as first-class runtime behavior.

This mismatch increases cognitive load:

- readers expect a thin provider gateway
- they instead find a capability runtime with memory and tool semantics

#### 2. False symmetry between capability and adapter

Backend adapter is a real abstraction:

- one adapter per dialect/provider wire contract
- owns transport encoding/decoding and provider-specific behavior

AI capability is not modeled with equal clarity.

Instead, capability is split awkwardly across:

- `chat/*`
- `CapabilityGuard` boolean checks
- caller-side route choice
- backend capability flags

This creates a bad middle state:

- capability is too important to stay implicit
- adapter is too low-level to carry capability ownership

#### 3. Late capability failure

Route selection happens first.
Capability mismatch is checked later by `CapabilityGuard`.

That means the system has:

- backend selection
- then capability rejection

instead of a cleaner model such as:

- ask for one capability contract
- capability layer resolves an eligible backend or fails deterministically

This is a major reason the "balance" between capability and adapter feels off.

#### 4. Partially implemented surfaces

There are signs that the current abstraction stack is not yet coherent:

- `Thread::stream()` exists but is not implemented.
- `default_session_ttl_seconds` exists in config but appears unused.
- adapters support streaming while the active thread API is effectively complete-only.

These are not isolated TODOs. They indicate the top-level capability model is still undecided.

#### 5. Routing ownership is split across layers

Route selection is not owned in one place.

Today it is split across:

- `ai_gateway.chat.default_route`
- backend model aliases
- `cortex.helper_routes.*`
- organ-specific `resolve_route(...)` logic in Cortex

This means "which capability uses which backend/model" is not a single explicit policy surface.
It is fragmented across caller config and gateway config.

That fragmentation is another concrete reason the capability/adapter balance feels wrong.

#### 6. Cortex and Gateway duplicate state concerns

Gateway owns thread state.
Cortex also owns:

- `primary_thread_state`
- `primary_continuation_state`

This is not automatically wrong, but it means the current design stores chat-lifecycle decisions across both sides of the boundary.
That is exactly where readability and maintainability decay begin.

### First-principles interpretation of `attempt`

`attempt` should not be treated as a domain concept.

It is only meaningful as:

- one transport invocation attempt inside one logical gateway request

It should stay:

- internal to transport/reliability logic
- visible in low-level telemetry

It should not leak upward as if it were a user-meaningful chat/thread/cortex concept.

### Where retry / budget / reliability should live

These do not all belong in the same place.

Reasonable ownership split:

- Gateway transport layer owns:
  - request timeout shaping
  - backend concurrency limits
  - circuit breaker
  - rate smoothing
  - generic retry loop

- Adapter owns:
  - provider-specific retryability details
  - protocol parse failures
  - whether retry after partial output or tool activity is safe

- Capability layer owns:
  - tool/message/thread invariants
  - continuation semantics
  - capability-specific observability payloads

So a full move of retry / budget / reliability into adapters would likely reduce clarity, not improve it.
It would duplicate cross-backend policy and blur the line between transport policy and provider mechanics.

### Observability seam: what not to do

Do not add a `Tool` trait or another seam only because observability wants cleaner hooks.

A seam is justified only if it matches a real ownership boundary.

For observability, the stronger existing boundary is:

- transport request lifecycle
- chat capability lifecycle

That is already consistent with Product TDD.

### Likely maintainable direction

The maintainable direction is not "more abstraction".
It is "make the existing real layers explicit".

Candidate shape:

1. Keep one internal `ai_gateway` module inside `core`.
2. Inside it, separate by ownership:
   - transport / backend policy
   - chat capability
3. Make the public naming honest:
   - if the public surface is chat-first, expose that explicitly rather than pretending the whole module is generic inference.
4. Stop injecting raw `Chat` / `Thread` machinery directly into Cortex.
5. Give Cortex a smaller capability-facing port that expresses what Cortex actually needs.

One likely boundary:

- Cortex depends on a `PrimaryChatPort` or `ChatCapabilityPort`
- the chat capability implementation may still use gateway thread internals underneath
- backend adapters remain strictly transport/dialect implementations

### Core challenge to the current design instinct

If the intended future includes more AI capabilities beyond chat, then the current structure is underspecified.

If the intended future is actually chat-first for the foreseeable horizon, then calling the whole thing a generic provider-agnostic inference gateway is misleading.

It should be one or the other in naming and in boundary design.
Trying to keep both stories alive at once is what currently damages readability.

## Verification

Evidence used in this exploration:

- GitHub issue `#17`
- `docs/10-prd/behavior/capabilities.md`
- `docs/20-product-tdd/unit-topology.md`
- `docs/20-product-tdd/unit-boundary-rules.md`
- `docs/20-product-tdd/observability-contract.md`
- `docs/30-unit-tdd/core/design.md`
- `docs/30-unit-tdd/core/interfaces.md`
- `docs/30-unit-tdd/core/observability.md`
- `core/src/ai_gateway/*`
- `core/src/cortex/runtime/primary.rs`
- `tasks/refactor-ai-gateway-2/CURRENT.md`
- `tasks/refactor-ai-gateway-3/BAD-SMELL.md`

## Promotion

Do not promote yet.

This note is still volatile because the repo has not yet decided:

- whether AI Gateway is fundamentally transport-first or capability-first
- whether Cortex should own chat orchestration directly or through a narrower capability port
