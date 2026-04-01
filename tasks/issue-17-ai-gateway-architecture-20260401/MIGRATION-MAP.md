# Migration Map Draft

## Status

Exploratory only.
No production refactor is implied by this file.

## Purpose

This file maps the current `core/src/ai_gateway` layout to the target capability-first shape.

It exists to answer one concrete question:

- which files are merely misplaced
- which files are modeling the wrong ownership boundary

## Migration Rules

1. Behavior-preserving moves come before behavior changes.
2. Do not add new capabilities while chat contracts are still moving.
3. Do not keep pretending that model-centric routing types are capability-neutral.
4. Avoid introducing compatibility wrappers unless they reduce migration risk materially.

## Current Top-Level Problem

Current top-level layout:

```text
ai_gateway/
  adapters/
  chat/
  credentials.rs
  error.rs
  resilience.rs
  router.rs
  telemetry.rs
  types.rs
```

This mixes three ownership levels:

- shared capability-neutral infra
- chat capability runtime
- chat-specific backend routing hidden behind generic names

That is the naming/ownership drift the refactor must correct.

## Recommended Target Shape

```text
ai_gateway/
  mod.rs
  shared/
    mod.rs
    credentials.rs
    error.rs
    config.rs
    telemetry.rs
    transport/
    execution/
  chat/
    mod.rs
    contract.rs
    config.rs
    api.rs
    runtime.rs
    routing.rs
    state/
    tooling/
    backends/
```

## File Mapping

### Shared / capability-neutral

`/Users/lanzhijiang/Development/Beluna/core/src/ai_gateway/credentials.rs`
-> `ai_gateway/shared/credentials.rs`

Reason:

- provider credential resolution is not chat-specific

`/Users/lanzhijiang/Development/Beluna/core/src/ai_gateway/error.rs`
-> `ai_gateway/shared/error.rs`

Reason:

- transport/provider error details remain shared infra
- chat should wrap rather than redefine provider diagnostics

`/Users/lanzhijiang/Development/Beluna/core/src/ai_gateway/resilience.rs`
-> split into `ai_gateway/shared/execution/*`

Recommended split:

- retry / backoff policy
- breaker state
- concurrency leasing
- rate smoothing

Reason:

- current file is capability-neutral but too bundled
- issue `#17` already questions this bundling

`/Users/lanzhijiang/Development/Beluna/core/src/ai_gateway/telemetry.rs`
-> `ai_gateway/shared/telemetry.rs`

Reason:

- this file is transport/request observability, not chat conversation observability

`/Users/lanzhijiang/Development/Beluna/core/src/ai_gateway/adapters/http_errors.rs`
-> `ai_gateway/shared/transport/http_errors.rs`

`/Users/lanzhijiang/Development/Beluna/core/src/ai_gateway/adapters/http_stream.rs`
-> `ai_gateway/shared/transport/http_stream.rs`

`/Users/lanzhijiang/Development/Beluna/core/src/ai_gateway/adapters/wire.rs`
-> `ai_gateway/shared/transport/wire.rs`

Reason:

- these are transport helpers, not chat capability semantics

### Shared files that should be split, not merely moved

`/Users/lanzhijiang/Development/Beluna/core/src/ai_gateway/types.rs`
-> split across `ai_gateway/shared/config.rs` and `ai_gateway/chat/config.rs`

Shared candidates:

- credential reference
- provider bootstrap config
- shared resilience config
- provider family / dialect enum

Chat-local candidates:

- route key
- chat binding config
- model target
- chat default-route policy
- chat default tool-round policy

Reason:

- current `types.rs` is model-centric and therefore not truly shared
- leaving it intact would preserve the current false genericity

`/Users/lanzhijiang/Development/Beluna/core/src/ai_gateway/router.rs`
-> split between `ai_gateway/chat/routing.rs` and possibly `ai_gateway/shared/config.rs`

Reason:

- current router resolves aliases directly to backend/model targets
- that is chat binding resolution, not universal gateway routing
- keeping it top-level would continue to privilege chat's current model-centric config shape

## Chat capability runtime

`/Users/lanzhijiang/Development/Beluna/core/src/ai_gateway/chat/api_chat.rs`
-> `ai_gateway/chat/api.rs`

Expected contract changes:

- `open_thread(...)` remains
- `clone_thread_with_turns(...)` should disappear from the public surface
- query/rewrite/snapshot responsibilities should be re-expressed in thread-centric terms

`/Users/lanzhijiang/Development/Beluna/core/src/ai_gateway/chat/runtime.rs`
-> `ai_gateway/chat/runtime.rs`

Reason:

- file is already capability-local
- but it should depend on `chat/routing.rs`, `chat/backends/*`, and `shared/*`

`/Users/lanzhijiang/Development/Beluna/core/src/ai_gateway/chat/thread.rs`
-> `ai_gateway/chat/state/thread.rs`

Reason:

- this file is canonical chat state ownership plus runtime orchestration

Critical change later:

- `complete(TurnInput)` should converge toward thread-centric append APIs

`/Users/lanzhijiang/Development/Beluna/core/src/ai_gateway/chat/turn.rs`
-> `ai_gateway/chat/state/turn.rs`

Reason:

- internal turn invariants belong under state ownership

`/Users/lanzhijiang/Development/Beluna/core/src/ai_gateway/chat/message.rs`
-> `ai_gateway/chat/state/message.rs` or `ai_gateway/chat/contract.rs`

Bias:

- public message types probably belong in `chat/contract.rs`
- internal helpers stay in `state/` only if they are not part of the external capability contract

`/Users/lanzhijiang/Development/Beluna/core/src/ai_gateway/chat/message_codec.rs`
-> split between `ai_gateway/chat/backends/bridge.rs` and `ai_gateway/chat/state/message_normalization.rs`

Reason:

- provider-wire normalization should not look like a generic message utility
- this file currently hides provider-shape leakage inside a neutral-sounding codec

`/Users/lanzhijiang/Development/Beluna/core/src/ai_gateway/chat/thread_types.rs`
-> split between `ai_gateway/chat/contract.rs` and `ai_gateway/chat/state/snapshots.rs`

Reason:

- it currently mixes public API, storage-facing helper types, and legacy turn-centric naming

`/Users/lanzhijiang/Development/Beluna/core/src/ai_gateway/chat/types.rs`
-> split between `ai_gateway/chat/contract.rs` and `ai_gateway/chat/backends/bridge.rs`

Public contract candidates:

- `ContentPart`
- `OutputMode`
- `UsageStats`
- `FinishReason`
- public stream event types if streaming remains public

Internal-only candidates:

- backend invocation payload
- backend raw event types
- adapter invocation internals

Reason:

- current file mixes contract and adapter bridge concerns

### Chat tooling

`/Users/lanzhijiang/Development/Beluna/core/src/ai_gateway/chat/tool.rs`
-> `ai_gateway/chat/tooling/tool_definitions.rs`

`/Users/lanzhijiang/Development/Beluna/core/src/ai_gateway/chat/executor.rs`
-> `ai_gateway/chat/tooling/executor.rs`

`/Users/lanzhijiang/Development/Beluna/core/src/ai_gateway/chat/tool_scheduler.rs`
-> `ai_gateway/chat/tooling/scheduler.rs`

Reason:

- tool definitions, execution hooks, and scheduling are related but distinct
- today they are capability-local, not shared gateway abstractions

Important contract change later:

- `ToolScheduler` should become a `Thread` concern, not a `Turn` concern

### Chat backend bridges

`/Users/lanzhijiang/Development/Beluna/core/src/ai_gateway/adapters/openai_compatible/*`
-> `ai_gateway/chat/backends/openai_compatible/*`

`/Users/lanzhijiang/Development/Beluna/core/src/ai_gateway/adapters/ollama/*`
-> `ai_gateway/chat/backends/ollama/*`

`/Users/lanzhijiang/Development/Beluna/core/src/ai_gateway/adapters/github_copilot/*`
-> `ai_gateway/chat/backends/github_copilot/*`

`/Users/lanzhijiang/Development/Beluna/core/src/ai_gateway/adapters/mod.rs`
-> split between `ai_gateway/chat/backends/mod.rs` and `ai_gateway/shared/transport/*`

Reason:

- these are not generic backend adapters for all future AI capabilities
- they are current chat backend bridges

`/Users/lanzhijiang/Development/Beluna/core/src/ai_gateway/chat/capabilities.rs`
-> `ai_gateway/chat/backends/capability_guard.rs` or merge into `chat/routing.rs`

Reason:

- this file is actually backend feature validation for chat requests
- the current name `capabilities.rs` is too vague and sounds more general than it is

## `Cortex` Boundary Cleanup

Current problematic imports in `/Users/lanzhijiang/Development/Beluna/core/src/cortex/runtime/primary.rs`:

- `CloneThreadOptions`
- `TurnInput`
- `TurnResponse`
- `TurnQuery`
- direct dependence on thread clone/reset behavior

Target dependency direction:

- `Cortex` depends on `chat/contract.rs`
- `Cortex` uses `Thread`, `ThreadSpec`, `AppendRequest`, `ThreadRewriteRequest`
- `Cortex` does not depend on raw turn-construction or turn-cloning primitives

Important note:

- `TurnResponse` is especially revealing
- it means `Cortex` is still shaped around a single backend completion step, not a thread transaction result

## Recommended Sequencing

### Phase 1

Rename and split files without changing semantics:

- move obvious shared files
- move chat backend adapters under `chat/backends`
- create `chat/contract.rs` and re-export existing public types through it

### Phase 2

Replace misleading public names:

- `ThreadOptions` -> `ThreadSpec`
- `TurnInput` -> `AppendRequest`
- `TurnOutput` / `TurnResponse` public dependency -> `AppendMessagesResult`

### Phase 3

Move responsibility boundaries:

- make `Thread` own tool continuation orchestration
- make `Turn` invariant-only
- eliminate public `clone_thread_with_turns(...)`

### Phase 4

Only after the above:

- simplify config schema
- simplify router into capability-local binding resolution
- revisit resilience terminology with the new ownership model in place

## Anti-goals

- do not add `asr/` or `tts/` folders yet just to prove generality
- do not create one mega `provider.rs` that mixes all capability-level settings
- do not keep a top-level `router.rs` that is secretly chat model routing
