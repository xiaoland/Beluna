# Provider-Context And Retry Grounding

## Status

Exploratory.
Bridges strict issue `#17` scope and the broader four-freeze discussion.
No public chat capability surface change is implied.

## Why this file exists

`FOUR-QUESTION-FREEZE.md` is directionally correct, but `provider-context admission` and
`retry-safety contract` are still abstract there.

Current code now gives enough evidence to narrow those freezes without jumping into speculative
public API redesign.

This file answers:

- what provider-context boundary actually exists today
- what retry-safety contract actually exists today
- which abstractions are real versus only nominal

## Code-Grounded Observations

### 1. There is no active provider-context channel today

Current adapter seam:

```rust
BackendAdapter::complete(AdapterContext, &TurnPayload)
    -> Result<BackendCompleteResponse, GatewayError>
```

Current `AdapterContext` carries only transport/execution data:

- backend identity
- model
- backend profile
- credential
- timeout
- request id

Current `TurnPayload` carries:

- canonical messages
- tools
- output mode
- limits
- `enable_thinking`
- `metadata`

Important code fact:

- current adapters do not read `payload.metadata`

Current runtime uses `metadata` for:

- `tick`
- `parent_span_id`
- `organ_id`
- chat/thread/turn identifiers
- turn/request observability decoration
- debug serialization through `turn_payload_json(...)`

Implication:

- provider-thread inheritance is not a real runtime path today
- default-deny is currently achieved accidentally because nothing consumes the bag
- if future work reuses this bag as provider context, ownership classes will mix immediately

### 2. `TurnPayload.metadata` is a runtime bag, not a semantic request contract

Because adapters ignore it and runtime emits it into observability/debug payloads, current
semantics are:

- runtime bookkeeping
- correlation
- local observability decoration

Not:

- Beluna canonical chat state
- provider-inheritable context

This is good enough for current code, but the name and placement invite later misuse.

### 3. Retry safety is currently one active boolean plus several dormant hooks

Current active retry signal:

- `GatewayError.retryable`

Current dormant or only partially connected hooks:

- `RetryPolicy::AdapterResumable`
- `BackendCapabilities.resumable_streaming`
- `BackendAdapter::supports_tool_retry()`

Code facts:

- no adapter overrides `supports_tool_retry()`
- `Thread::stream()` is not implemented
- `ChatRuntime::dispatch_complete(...)` always calls
  `can_retry(err, attempt, false, false, ...)`

Implication:

- current complete path does not distinguish before-output, after-output, or after-tool retry
  states in practice
- current code effectively supports only "retry the whole backend request before any committed
  semantic effect exists"
- the richer retry state machine is aspirational scaffolding, not active behavior

### 4. `attempt` is transport lifecycle language, not chat semantics

Current code creates and consumes `attempt` inside the runtime retry loop and request telemetry.

It is not part of:

- `TurnInput`
- `TurnOutput`
- canonical thread state
- adapter-facing semantic request content

Implication:

- issue `#17` is correct to treat `attempt` as removable or renameable transport terminology
- it should not be promoted into chat, thread, or clone semantics

### 5. Clone lineage remains outside the adapter boundary

Current clone entry is still:

- `Chat::clone_thread_with_turns(...)`

Current observability already emits thread snapshots with:

- `source_turn_ids`

So clone lineage belongs to:

- chat runtime
- thread observability
- canonical history policy

Not to:

- adapter transport contract

The missing piece is not adapter support for clone.
It is explicit policy for what, if anything, derived chats may inherit as provider-managed
context.

## First-Principles Conclusion

Current code supports a stricter and smaller truth than some earlier notes implied:

1. Beluna owns canonical chat history and commit semantics.
2. Runtime owns observability metadata and request control context.
3. Provider-context inheritance is not implemented yet and should remain default-deny until one
   backend proves a real need.
4. Retry safety is only trustworthy for replaying one whole backend request before canonical
   commit. Anything richer is future work unless streaming or provider-resume becomes real.

## Freeze Refinement

### Freeze 3 refined: provider context is absent today, and that is a feature

The correct near-term rule is not "design a rich generic provider-context model now."
It is:

- keep provider context absent by default
- keep runtime metadata out of adapter inheritance paths
- introduce a dedicated provider-context object only when one backend truly needs resumable or
  thread-native inheritance

Therefore:

- `TurnPayload.metadata` must remain runtime-owned
- it must not be reinterpreted as provider context
- any future provider context must be allowlisted and shaped separately

### Freeze 4 refined: remove false genericity before adding richer retry semantics

The correct near-term rule is not "finish the full retry matrix immediately."
It is:

- acknowledge that current complete path only knows full-request replay
- stop pretending `supports_tool_retry()` already carries real contract weight
- only introduce richer retry classification when an implemented path can use it honestly

Therefore the issue-17-safe bias should be:

- either delete `supports_tool_retry()` and similar dormant hooks
- or replace them with a richer internal type only when at least one adapter can populate it

### Consequence for `RetryPolicy::AdapterResumable`

`AdapterResumable` and `resumable_streaming` should not drive the refactor narrative today.
With `Thread::stream()` still unimplemented and no provider-context channel present, these are
future hooks, not current architecture centerpieces.

They may stay as placeholders.
But they should not be mistaken for an already-real ownership boundary.

## Issue-17-Safe Internal Cleanup Candidates

1. Keep `attempt` strictly inside transport/request observability and retry-loop terminology.
2. Treat `TurnPayload.metadata` as runtime metadata by policy, even if the field name stays
   temporarily unchanged.
3. Do not add a generic provider-context channel until one backend requires it.
4. Trim or quarantine dormant retry abstractions that no adapter actually implements.
5. Preserve clone lineage work at chat/runtime observability level, not in adapter types.

## Broader Follow-up Candidates, Not Required For Issue #17

1. Introduce an explicit `ProviderContextInput/Output` internal channel when provider-native
   thread inheritance becomes real.
2. Replace boolean retryability with a phase-aware internal classification once streaming or
   resumable dispatch is actually implemented.
3. Revisit clone-derived provider-context policy only after 1 and 2 exist.

## What This Note Deliberately Rejects

1. Do not design public append/snapshot/error APIs from this evidence.
2. Do not invent multi-capability scaffolding from retry/provider-context gaps.
3. Do not widen adapter abstractions just to make capability and backend look symmetrical.
4. Do not let dormant hooks masquerade as stable architecture.
