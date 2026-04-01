# Minimal Adapter Contract Boundary For Issue #17

## Status

Exploratory.
In scope for issue `#17`.
No public chat capability surface change is implied.

## Why this file exists

After scope realignment, the next useful question is:

- what is the minimal internal adapter contract that clarifies ownership
- without changing `Chat`, `Thread`, `TurnInput`, or the current Cortex-facing surface

This file answers that question from current code first, then proposes the smallest contract correction.

## Current Code-Grounded Seam

Today the real adapter seam is:

```rust
BackendAdapter::complete(AdapterContext, &TurnPayload) -> Result<BackendCompleteResponse, GatewayError>
```

Current relevant types:

- `AdapterContext`
  - `backend_id`
  - `model`
  - `profile`
  - `credential`
  - `timeout`
  - `request_id`
- `TurnPayload`
  - canonical messages
  - tools
  - output mode
  - limits
  - enable thinking
  - free-form metadata bag
- `BackendCompleteResponse`
  - output text
  - tool calls
  - usage
  - finish reason

Current runtime ownership around that seam:

1. `Thread::complete(...)` builds canonical dispatch messages and turn metadata.
2. `ChatRuntime::dispatch_complete(...)` performs capability checks, resilience leasing, retry loop, and request observability.
3. The adapter performs provider-specific request encoding / transport / response parsing.
4. Runtime turns the adapter result into current `TurnResponse` and commits canonical history.

This means the issue is not lacking a seam.
The issue is that the seam is not explicit enough about ownership classes.

## What Is Blurry In The Current Seam

### 1. `TurnPayload.metadata` is an unbounded mixed-ownership bag

The payload metadata currently carries runtime and observability information such as:

- `tick`
- `parent_span_id`
- `organ_id`
- `request_id`
- `thread_id`
- `turn_id`

That is useful for runtime and observability, but it is not the same thing as provider-inheritable context.

If provider-thread inheritance is added or expanded later, this bag is the easiest place for accidental leakage.

### 2. Retry semantics are split across three weak signals

Current retry-related signals are:

- `GatewayError.retryable`
- `BackendAdapter::supports_tool_retry()`
- `ResilienceEngine::can_retry(...)`

This is too coarse.

In particular:

- `retryable` does not explain retryability scope
- `supports_tool_retry()` is only one boolean
- `can_retry(...)` still needs caller-supplied booleans like `emitted_output` and `emitted_tool`

The ownership line between shared execution policy and adapter-local safety knowledge is therefore still blurry.

### 3. Provider-thread inheritance has no dedicated boundary object

Current seam has:

- canonical request payload
- execution control context

But it has no explicit slot for:

- provider-thread context input
- provider-thread context output
- inherited-context filtering rules

So issue `#17` is correct to call this out.

### 4. `attempt` exists in runtime telemetry, not in business semantics

Code observation:

- `attempt` is generated and consumed inside `ChatRuntime::dispatch_complete(...)`
- adapters do not receive a first-class `attempt` domain object
- callers do not receive an `attempt` domain object

This strongly suggests `attempt` is transport lifecycle terminology, not a stable cross-backend chat concept.

### 5. Clone lineage is not part of the adapter seam

That is correct.
Clone lineage belongs to chat/runtime observability and canonical thread semantics, not to provider transport.

However, because provider-thread inherited context is under-specified, clone-related persistence boundaries are still ambiguous.

## Minimal Contract Correction

The smallest useful correction is to make the internal seam explicitly three-layered:

1. canonical chat dispatch input
2. execution control context
3. optional provider-context channel

Without changing the current public chat surface, the runtime should conceptually separate:

```rust
struct AdapterDispatchInput {
    backend: BackendBinding,
    request: CanonicalChatDispatch,
    execution: ExecutionControl,
    provider_context: Option<ProviderContextInput>,
}
```

This does not mean these exact names must become public types.
It means the ownership classes must stop being mixed implicitly.

## Recommended Ownership Split

### 1. Canonical chat dispatch

Owned by chat runtime.
This is the Beluna-owned semantic request:

- canonical message sequence
- tools
- output mode
- limits
- thinking flag

This is what adapters translate into provider-native wire calls.

This is the semantic heart of the request.

### 2. Execution control context

Owned by runtime / shared execution layer.
This includes:

- `request_id`
- timeout / deadline
- backend identity
- credential material
- observability correlation values

Important rule:

- execution control context may be used for transport, telemetry, budgeting, and correlation
- it must not silently become provider-thread state

### 3. Provider-context channel

Owned jointly by chat runtime policy and adapter bridge, but only through an explicit dedicated object.

This channel is the right place for:

- provider-native thread ids
- provider-native continuation handles
- provider-native resumable execution hints
- other backend-managed context that may be inherited across derived operations

Important rule:

- if something enters this channel, it must be intentionally admitted
- free-form runtime metadata must never arrive here by accident

## Retry / Reliability Contract

Issue `#17` does not require moving all resilience into adapters.
It requires clarifying what adapters actually own.

### Shared layer should continue to own

- concurrency limits
- circuit breaker state
- generic backoff timing
- global request timeout shaping
- top-level retry loop orchestration

### Adapter should explicitly own

- whether one specific provider failure is retryable
- whether retry is safe after provider partial output
- whether retry is safe after provider tool activity
- whether provider-native context can resume safely

### Therefore the missing signal is not "more layers"

The missing signal is a more precise adapter retry-safety classification than:

- `GatewayError.retryable`
- one `supports_tool_retry()` boolean

The smallest correction would be an internal adapter-facing retry-safety description such as:

- retryable before any provider output
- retryable after partial output
- retryable after tool emission
- resumable with provider context

The exact type can stay internal.
What matters is that the adapter stops compressing all of that into one coarse boolean.

## Inherited-Context Admission Rule

For issue `#17`, the most important negative rule should be frozen now:

- runtime-only metadata is not provider context

Default-deny examples:

- `tick`
- `organ_id`
- `parent_span_id`
- `request_id`
- local budgeting state
- retry counters
- thread-local observability metadata

If a backend truly needs a provider-thread context object, it should receive only:

- a dedicated admitted provider-context structure
- built intentionally from canonical state or explicit backend-owned continuation state

Not the entire metadata map.

## Clone Semantics Boundary

Clone semantics should be clarified at chat/runtime level, not adapter level.

For issue `#17`, the practical rule is:

- clone lineage and selected-turn provenance belong in chat observability and canonical thread policy
- adapters should only see provider context that the runtime explicitly chooses to inherit

This keeps clone semantics from leaking downward into transport abstractions.

## Recommended Next Design Questions

1. What is the minimal internal type that represents provider-context admission without exposing it publicly?

2. What retry-safety classification replaces the current `supports_tool_retry()` boolean?

3. Which current `TurnPayload.metadata` fields are strictly observability-only, and therefore must never enter provider inheritance?

4. Does clone lineage require explicit `source_thread_id` in chat observability, or is current parentage sufficient?

## Working Conclusion

Issue `#17` does not need a new public chat API to make progress.

It needs a clearer internal statement that:

- chat runtime owns canonical request semantics
- shared execution owns generic transport policy
- adapters own provider-specific translation and retry-safety knowledge
- provider-thread inheritance is a separate admitted channel, not an accident of free-form metadata
