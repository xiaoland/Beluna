# Chat Error And Snapshot Contract Freeze Proposal

## Status

Proposed freeze only.
Exploratory, non-authoritative, no production code change is implied.

## Purpose

This file freezes three tightly related chat-capability contracts:

- `ChatError`
- `ThreadSnapshot`
- `TurnSnapshot`

These three should be frozen together.
If they are designed independently, the boundary between caller error, canonical state, and backend failure will stay blurry.

## Design Goals

1. Public chat contracts must be capability-first.
2. Backend / transport detail must remain attached for diagnostics, but must not become the public taxonomy.
3. Snapshot must export only committed Beluna-canonical state.
4. Restore must succeed from canonical state alone.
5. Snapshot must not duplicate derived fields or hidden runtime state.

## Non-goals

Do not:

- expose raw `GatewayError` as the primary public chat error contract
- export provider-native thread ids or hosted prompt state in snapshots
- export half-complete tool continuation state
- duplicate obviously derived snapshot fields such as `message_count`, `has_tool_calls`, or `completed`

Those all increase surface area without improving clarity.

## 1. `ChatError`

## Exact proposed shape

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatError {
    pub kind: ChatErrorKind,
    pub operation: ChatOperation,
    pub message: String,
    pub retryable: bool,
    pub thread_id: Option<ThreadId>,
    pub turn_id: Option<u64>,
    pub route_ref: Option<ChatRouteRef>,
    pub route_key: Option<ChatRouteKey>,
    pub tool_name: Option<String>,
    pub tool_call_id: Option<String>,
    pub backend: Option<BackendFailureInfo>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChatErrorKind {
    InvalidInput,
    UnsupportedFeature,
    ToolExecutionFailed,
    InvariantViolation,
    BackendFailure,
    Internal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChatOperation {
    OpenThread,
    RestoreThread,
    Append,
    RewriteContext,
    InspectTurns,
    Snapshot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendFailureInfo {
    pub gateway_kind: GatewayErrorKind,
    pub backend_id: Option<String>,
    pub provider_code: Option<String>,
    pub provider_http_status: Option<u16>,
}
```

## Why this shape is the right minimum

It is intentionally narrow:

- one capability-first `kind`
- one operation label
- one human message
- one retry hint
- a small amount of context for thread / turn / tool attribution
- optional backend diagnostics

This is enough for callers, logs, and observability.
Anything larger starts drifting toward transport leakage or ad hoc debug payloads.

## Variant semantics

### `InvalidInput`

Use when the caller violates the public chat contract.

Examples:

- empty append batch
- invalid route syntax
- snapshot restore input is malformed
- caller asks rewrite to keep duplicate or unordered turn ids

Rule:

- retryable is always `false`

### `UnsupportedFeature`

Use when the request is semantically valid chat, but the selected route/binding cannot satisfy it.

Examples:

- tools requested on a binding without tool support
- vision content sent to a route without vision support
- `JsonSchema` output requested on a binding that only supports plain text

Rule:

- retryable is usually `false`
- switching route/binding is a caller policy decision, not automatic retry

### `ToolExecutionFailed`

Use when AI Gateway cannot obtain a canonical tool result message needed to finish the turn transaction.

Examples:

- scheduler crashed
- local tool executor returned infrastructure failure
- required tool result could not be produced in canonical form

Important boundary:

- a business-level tool failure encoded as a normal `ToolCallResultMessage` payload is not `ChatError`
- this error is only for failure to obtain the canonical result message itself

Rule:

- default retryable is `false`
- if a future tool transport supports safe retry, that should be an explicit policy choice

### `InvariantViolation`

Use when Beluna canonical chat state is internally inconsistent.

Examples:

- dangling `ToolCallResultMessage`
- `ToolCallMessage` without matching result in committed history
- duplicate `turn_id`
- snapshot contains forbidden message ordering

Rule:

- retryable is always `false`
- this kind means runtime integrity is broken or restore input claims impossible canonical state

### `BackendFailure`

Use when the operation failed because the selected backend / transport / resilience layer failed.

Examples:

- auth rejected
- provider timeout
- circuit open
- rate limited
- provider returned invalid protocol for the current bridge

Rule:

- `backend` must be populated
- `retryable` is inherited from the translated gateway failure

### `Internal`

Use for unexpected runtime bugs that do not fit a stable semantic category.

Rule:

- retryable is always `false` by default

## Mapping rule from `GatewayError`

Public callers should branch on `ChatError.kind`, not on `GatewayErrorKind`.

Recommended translation:

```text
GatewayError::InvalidRequest
  -> ChatError::InvalidInput

GatewayError::UnsupportedCapability
  -> ChatError::UnsupportedFeature

GatewayError::{Authentication, Authorization, RateLimited, Timeout, CircuitOpen, BudgetExceeded, BackendTransient, BackendPermanent}
  -> ChatError::BackendFailure

GatewayError::ProtocolViolation
  -> ChatError::BackendFailure or ChatError::InvariantViolation
     depending on whether the violation originated in backend normalization
     or in Beluna canonical state

GatewayError::Internal
  -> ChatError::Internal unless the runtime can prove this is actually a canonical-state invariant failure
```

This is deliberately not a one-to-one mirror.
If it were, chat would still be backend-first in public semantics.

## Recommended constructors

```rust
impl ChatError {
    pub fn invalid_input(operation: ChatOperation, message: impl Into<String>) -> Self;
    pub fn unsupported_feature(operation: ChatOperation, message: impl Into<String>) -> Self;
    pub fn tool_execution_failed(
        operation: ChatOperation,
        tool_name: impl Into<String>,
        tool_call_id: impl Into<String>,
        message: impl Into<String>,
    ) -> Self;
    pub fn invariant_violation(operation: ChatOperation, message: impl Into<String>) -> Self;
    pub fn from_gateway(
        operation: ChatOperation,
        route_ref: Option<ChatRouteRef>,
        route_key: Option<ChatRouteKey>,
        thread_id: Option<ThreadId>,
        turn_id: Option<u64>,
        source: GatewayError,
    ) -> Self;
}
```

## Rejected alternatives

### Reject: expose `GatewayError` directly

That would make transport detail the primary public taxonomy again.

### Reject: create one huge nested debug payload

That would produce unstable surface area and weak caller discipline.

### Reject: split every backend condition into its own `ChatErrorKind`

That duplicates shared error vocabulary and bloats the capability contract.

## 2. `ThreadSnapshot`

## Exact proposed shape

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadSnapshot {
    pub snapshot_version: u32,
    pub thread_id: ThreadId,
    pub route_key: ChatRouteKey,
    pub system_prompt: Option<String>,
    pub tools: Vec<ChatToolDefinition>,
    pub defaults: ThreadExecutionDefaults,
    pub turns: Vec<TurnSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadExecutionDefaults {
    pub output_mode: OutputMode,
    pub limits: TurnLimits,
    pub enable_thinking: bool,
}
```

## Why this shape is the right minimum

`ThreadSnapshot` should carry only what is required to reconstruct Beluna-canonical thread state:

- stable identity
- resolved route
- thread-level system prompt
- baseline tools
- default execution policy
- committed turn history

Anything else is either:

- provider-native hidden state
- per-operation metadata
- derived state
- transient runtime cache

Those do not belong in the canonical snapshot.

## Thread-level rules

### `snapshot_version`

Rule:

- start at `1`
- version bump only when serialized meaning changes, not for internal refactors

### `thread_id`

Rule:

- stable thread identity
- restore preserves it unless the caller explicitly requests clone semantics elsewhere

### `route_key`

Rule:

- must be the resolved canonical route key
- must represent stable runtime identity (capability + binding key)
- restore must not depend on config fallback after the snapshot is already materialized

This is important.
Allowing route ambiguity to re-enter during restore would weaken snapshot determinism.

### `system_prompt`

Rule:

- thread-level canonical system instructions live here
- they do not also appear as committed `SystemMessage` inside turns

### `tools`

Rule:

- this is the baseline tool inventory attached to the thread
- per-append overrides are not baked into the thread snapshot as separate state

### `defaults`

Rule:

- only durable per-thread defaults belong here
- one-append metadata and request ids do not

### `turns`

Rule:

- ordered by ascending `turn_id`
- contains only committed canonical turns
- may contain gaps in ids after rewrite operations

Important implication:

- `next_turn_id` is derived as `max(turn_id) + 1`
- dense reindexing is not part of the canonical model

## Rejected fields

Do not include:

- provider-native thread ids
- provider-native prompt ids
- provider session ttl
- open continuation state
- thread-level metadata bag used only for observability
- `next_turn_id`
- derived counters

Each of these either leaks implementation detail or duplicates truth.

## 3. `TurnSnapshot`

## Exact proposed shape

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnSnapshot {
    pub turn_id: u64,
    pub messages: Vec<Message>,
    pub metadata: BTreeMap<String, String>,
    pub usage: Option<UsageStats>,
    pub finish_reason: FinishReason,
}
```

## Why this shape is the right minimum

`TurnSnapshot` should capture one committed semantic turn and nothing more:

- stable turn identity
- canonical message sequence
- observational metadata
- usage when available
- terminal finish reason

Fields intentionally omitted:

- `completed`: always true for committed snapshots
- `message_count`: derived
- `has_tool_calls`: derived
- `source_turn_id`: obsolete if kept turns preserve identity

## Turn-level invariants

### `turn_id`

Rule:

- unique within thread
- strictly increasing across `turns`
- gaps are allowed

### `messages`

Rule:

- must not be empty
- must not contain `SystemMessage`
- must satisfy canonical message ordering

Canonical ordering minimum:

- explicit `ToolCallMessage` must be immediately followed by matching `ToolCallResultMessage`
- `ToolCallResultMessage` must never appear without its preceding `ToolCallMessage`
- trailing unpaired `ToolCallMessage` is invalid

Important transitional rule:

- if `AssistantMessage.tool_calls` still exists in the code during migration, committed snapshots must require it to be empty
- canonical tool history is represented by explicit `ToolCallMessage` / `ToolCallResultMessage`

This rule is necessary to avoid dual committed representations.

### `metadata`

Rule:

- observational only
- restore preserves it but must not require it for canonical semantics

This means keys such as `tick`, `organ_id`, `request_id`, or `parent_span_id` may be useful for logs, but thread restore must not depend on them.

### `usage`

Rule:

- optional because some backends will not provide it
- when present, it describes the committed turn, not the entire thread

### `finish_reason`

Rule:

- required on committed turns
- if the runtime cannot determine a committed turn finish reason, the turn is not ready for snapshot

## Restore contract

`restore_thread(snapshot)` must validate the full snapshot before admitting it as canonical runtime state.

Validation steps:

1. validate `snapshot_version`
2. validate route key shape and capability ownership
3. validate turn id ordering and uniqueness
4. validate each turn's message invariants
5. derive runtime-local state such as next turn id
6. reject the whole restore if any invariant fails

Recommended failure mapping:

- malformed snapshot from caller -> `ChatErrorKind::InvalidInput`
- internally produced snapshot that fails canonical validation -> `ChatErrorKind::InvariantViolation`

## Strong recommendation about system messages

The snapshot contract should freeze one rule now:

- system prompt is thread-level state
- committed turns do not contain `SystemMessage`

If this rule is not frozen now, future rewrite/restore logic will stay unnecessarily ambiguous.

## Strong recommendation about partial continuation state

The snapshot contract should also freeze one negative rule now:

- no open tool continuation state is public snapshot data

If AI Gateway needs private continuation caches later, they must be disposable and reconstructable.
They are not canonical state.

## Summary

The proposed freeze is intentionally strict:

- `ChatError` is capability-first with optional backend diagnostics
- `ThreadSnapshot` exports only durable thread semantics
- `TurnSnapshot` exports only committed canonical turns

This keeps the public chat contract small while still giving `Cortex`, observability, and restore/replay enough stable structure.
