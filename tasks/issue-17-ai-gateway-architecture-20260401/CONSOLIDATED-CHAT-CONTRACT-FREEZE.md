# Consolidated Chat Contract Freeze

## Status

Broader-mode consolidated freeze.
Exploratory, non-authoritative, but this is now the active contract baseline for follow-up
implementation planning.

## Why this file exists

The broader working set already produced strong local drafts, but they were split across:

- `LOW-LEVEL-DESIGN.md`
- `CONFIG-AND-CHAT-CONTRACT.md`
- `ERROR-AND-SNAPSHOT-CONTRACT.md`
- `FOUR-QUESTION-FREEZE.md`
- `PROVIDER-CONTEXT-AND-RETRY-GROUNDING.md`

This file consolidates the stable parts into one readable contract story.

## Product And Observability Constraints

This freeze is constrained by the following upstream truths:

### From PRD

1. Runtime observability and recovery are first-class product commitments.
2. Incident response depends on high-quality telemetry and explicit failure semantics.
3. Operators and integrators must be able to diagnose failures from Beluna-native telemetry
   surfaces without code archaeology.

### From Product TDD

1. AI capability observability must remain split into:
   - capability-neutral gateway transport records
   - chat-capability records that own thread/turn/message/tool semantics
2. Chat-capability records must own:
   - committed conversation state
   - authoritative thread snapshots
   - tool activity
   - thinking payload when present
   - full chat payloads needed for source-grounded investigation
3. Observability enhancements should extend real ownership boundaries rather than introduce
   decorative new seams.

## Consolidated Decisions

## 1. Capability And Config Boundary

1. `AI Gateway` is a capability runtime, not merely a transport gateway.
2. Top-level organization is capability-first, not backend-first.
3. Shared provider inventory and capability-local binding config are the right split.
4. Public chat contracts are capability-local and Beluna-owned.

### Shared provider inventory

Shared provider inventory owns only provider/bootstrap facts such as:

- provider identity
- provider family / dialect
- endpoint / SDK bootstrap information
- credential reference
- coarse capability-family declaration

It does not own chat-specific route or model semantics.

### Chat-local config

Chat-local config owns:

- route aliases
- binding identity
- target model or other chat-native target selector
- native feature assumptions
- Beluna fallback policy

## 2. Route Reference And Resolved Key Contract

`alias` and `route key` are not the same concept.
This freeze separates them explicitly.

### Alias (human-facing selector)

The canonical alias reference grammar is:

- `<capability>.<alias>`

Examples:

- `chat.default`
- `chat.cortex_primary`
- `chat.cortex_helper`

Directionally:

```rust
pub struct ChatRouteAlias {
    pub capability: String,
    pub alias: String,
}
```

### Resolved route key (stable runtime identity)

The route key is the resolved runtime identity of one capability binding.
It is not a caller alias.

Directionally:

```rust
pub struct ChatRouteKey {
    pub capability: String,
    pub binding_id: String,
}
```

### Public route reference type

Public APIs that accept route selection should use a route reference type:

```rust
pub enum ChatRouteRef {
    Alias(ChatRouteAlias),
    Key(ChatRouteKey),
}
```

Rules:

1. alias input is for caller-friendly selection
2. key is for resolved stable runtime identity
3. multiple aliases may map to one route key
4. snapshots and canonical persisted thread state store route key, not alias
5. route references should not remain raw strings once they cross nontrivial internal boundaries

## 3. Public Object Model

The canonical chat abstraction stack remains:

- `ChatCapability`
- `Thread`
- `Turn`
- `Message`

No extra `ChatDialogue` wrapper is needed.

### Ownership rule

1. `Thread` is the main long-lived write-path object.
2. `Turn` is the semantic control unit for committed chat progression and context retention.
3. `Message` is the finest canonical payload unit inside a turn.

Important clarification:

- `Turn` is the control unit for context-management operations
- `Message` remains inspectable and snapshot-visible
- this does **not** justify arbitrary public message-splice surgery APIs

If future message-level transforms are required, they must be introduced as named semantic
operations, not as free-form edit hooks.

## 4. Public Thread Surface

### `ChatCapability`

Minimum public responsibilities:

- open a new thread from explicit thread semantics
- restore a thread from canonical snapshot state

Directionally:

```rust
pub trait ChatCapability: Send + Sync {
    async fn open_thread(&self, spec: ThreadSpec) -> Result<Thread, ChatError>;
    async fn restore_thread(&self, snapshot: ThreadSnapshot) -> Result<Thread, ChatError>;
}
```

### `Thread`

Ordinary public operations are:

- `append(...)`
- `append_message(...)`
- `append_messages(...)`
- `rewrite_context(...)`
- `derive_context(...)`
- `inspect_turns(...)`
- `snapshot()`

Important note:

- this freeze defines semantic mutation, not the exact Rust receiver syntax
- implementations may use interior synchronization
- the public contract is that these operations mutate or derive canonical thread state

### Canonical write-path naming

Freeze the ordinary write-path names as:

- `append`
- `append_message`
- `append_messages`

Do not freeze `advance(...)` as the public write-path name.

Reason:

- `append` matches the thread-centric conversation model
- `advance` is more metaphorical and less explicit about history growth

## 5. ThreadSpec And Execution Defaults

Freeze the durable thread-creation concepts as:

```rust
pub struct ThreadSpec {
    pub thread_id: Option<ThreadId>,
    pub route_ref: ChatRouteRef,
    pub system_prompt: Option<String>,
    pub tools: Vec<ChatToolDefinition>,
    pub defaults: ThreadExecutionDefaults,
}

pub struct ThreadExecutionDefaults {
    pub output_mode: OutputMode,
    pub limits: TurnLimits,
    pub enable_thinking: bool,
}
```

Rules:

1. `ThreadSpec` owns stable thread semantics.
2. `ThreadSpec.route_ref` is route selection input, not persisted canonical identity.
3. The stored/exported route is the resolved canonical route key.
4. One-append metadata is not durable thread identity.
5. Baseline tools belong to the thread; one-append tool changes are temporary overrides.

## 6. Ordinary Append Contract

### Input

Ordinary write-path input is:

- `UserMessage`

Not:

- arbitrary `Message`

Reason:

- ordinary caller writes should express user input only
- assistant/tool/system/tool-result messages are runtime-owned on the normal write path

### Per-append controls

Per-append controls remain thread-centric and belong in:

```rust
pub struct AppendOptions {
    pub tool_overrides: Vec<ToolOverride>,
    pub output_mode: Option<OutputMode>,
    pub limits: Option<TurnLimits>,
    pub enable_thinking: Option<bool>,
    pub metadata: BTreeMap<String, String>,
}
```

### Result

Freeze the ordinary append result envelope as:

```rust
pub struct AppendMessagesResult {
    pub new_messages: Vec<Message>,
    pub finish_reason: Option<FinishReason>,
    pub usage: Option<UsageStats>,
}
```

Do not return `Turn` from the ordinary write path.

## 7. Append Transaction Semantics

One append call equals:

- one caller-visible committed thread transaction
- at most one committed turn

Rules:

1. Caller sees either one valid committed turn outcome or failure with no partial canonical commit.
2. Internal continuation/tool orchestration remains runtime-owned.
3. No public `pending_tool_call_continuation` protocol is exposed upward.
4. If a tool cycle cannot be completed into valid canonical history, the append fails and commits
   nothing.

## 8. Higher-Level Context Control

This freeze accepts the new direction that context control will become stronger over time.

However, that control must remain readable.
So the contract is:

- use a higher-level context-control request family
- keep derive and rewrite as semantic sibling operations
- do not collapse them into arbitrary storage or message surgery

### Shared request family

Freeze a shared request family:

```rust
pub struct ThreadContextRequest {
    pub retention: TurnRetentionPolicy,
    pub system_prompt: SystemPromptAction,
    pub drop_unfinished_continuation: bool,
    pub reason: ContextControlReason,
}
```

With:

```rust
pub enum TurnRetentionPolicy {
    KeepAll,
    KeepLastTurns { count: usize },
    KeepSelectedTurnIds { turn_ids: Vec<u64> },
    DropAll,
}
```

The exact Rust shape may vary, but these semantics are frozen:

1. retention works on committed turn identities
2. system prompt change is explicit
3. continuation dropping is explicit
4. a structured reason exists for observability and diagnosis

### Rewrite semantics

`rewrite_context(...)` means:

- mutate the current thread's canonical context
- preserve surviving `turn_id` values
- produce one new canonical thread state

### Derive semantics

`derive_context(...)` means:

- create one new thread from selected committed context of the source thread
- preserve kept `turn_id` values from the source
- create explicit lineage rather than storage-level clone semantics

### Explicit rejection

Do not keep public:

- `clone_thread_with_turns(...)`
- raw append of already-built turns
- caller-managed reindexing
- arbitrary message splice/edit operations

## 9. Canonical Message And Turn Rules

### System prompt

Freeze:

- system prompt is thread-level canonical state
- committed turns do not contain `SystemMessage`

### Tool activity

Freeze one canonical representation for committed tool activity:

- `ToolCallMessage`
- `ToolCallResultMessage`

Not two equal-status committed representations.

If provider output arrives in embedded/provider-native form, normalization happens before
canonical commitment.

### Turn identity

Freeze:

- `turn_id` is stable once committed
- surviving turns keep their ids across rewrite/derive operations
- dense reindexing is not part of the canonical model

## 10. Snapshot And Restore Contract

Freeze the canonical snapshot direction as:

```rust
pub struct ThreadSnapshot {
    pub snapshot_version: u32,
    pub thread_id: ThreadId,
    pub route_key: ChatRouteKey,
    pub system_prompt: Option<String>,
    pub tools: Vec<ChatToolDefinition>,
    pub defaults: ThreadExecutionDefaults,
    pub turns: Vec<TurnSnapshot>,
}

pub struct TurnSnapshot {
    pub turn_id: u64,
    pub messages: Vec<Message>,
    pub metadata: BTreeMap<String, String>,
    pub usage: Option<UsageStats>,
    pub finish_reason: FinishReason,
}
```

Rules:

1. Snapshot exports only committed canonical state.
2. Snapshot contains no provider-native hidden state.
3. Snapshot contains no open continuation state.
4. Restore must validate snapshot invariants before admitting it as canonical runtime state.
5. Restore derives runtime-local state such as next turn id rather than storing derived counters.

### Metadata rule

Turn metadata is observational and preservable, but restore must not require it for canonical
semantics.

## 11. Public Error Contract

Freeze the public error direction as capability-first `ChatError`, not direct `GatewayError`.

Directionally:

```rust
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
```

Freeze the capability-first error kinds:

- `InvalidInput`
- `UnsupportedFeature`
- `ToolExecutionFailed`
- `InvariantViolation`
- `BackendFailure`
- `Internal`

And operations:

- `OpenThread`
- `RestoreThread`
- `Append`
- `RewriteContext`
- `DeriveContext`
- `InspectTurns`
- `Snapshot`

Rules:

1. callers branch on `ChatErrorKind`, not `GatewayErrorKind`
2. backend detail remains attached only as diagnostics
3. invariant failure remains distinct from backend failure

## 12. Transport / Retry / Provider-Context Constraints

These remain frozen constraints from the strict issue-17 analysis:

1. `attempt` remains transport/request terminology, not chat semantics.
2. transport request lifecycle remains in `ai-gateway.request`.
3. provider context remains explicit and default-deny.
4. runtime metadata must not silently become provider context.
5. retry semantics must not pretend to be richer than implemented reality.

This means the broader public contract must not force a fake generic provider-context or retry
story upward.

## 13. Observability Enhancements For The New Chat Contract

The new AI Gateway should gain observability strength, but only on real ownership boundaries.

### Family rule

Do not invent a decorative new generic family.

Keep:

- `ai-gateway.request` for capability-neutral transport
- `ai-gateway.chat.turn` for turn semantics
- `ai-gateway.chat.thread` for thread semantics

### Thread-level enhancement

Enhance `ai-gateway.chat.thread` so thread-level context control is source-grounded.

For open / derive / rewrite / turn-commit snapshots, the thread records should expose enough
structured semantics to answer:

1. what operation occurred
2. which thread is the source, if any
3. which turns were kept
4. which turns were dropped
5. whether continuation state was dropped
6. whether system prompt state changed
7. what the resulting authoritative thread snapshot is
8. why the context operation was requested

Directionally, this means extending thread-level structured fields such as:

- `kind`
- `source_thread_id?`
- `source_turn_ids?`
- `kept_turn_ids?`
- `dropped_turn_ids?`
- `continuation_dropped?`
- `context_reason?`

The exact unit-TDD field names may still be refined, but these semantics are now frozen.

### Turn-level enhancement

`ai-gateway.chat.turn` remains the owner of:

- committed turn payload
- committed tool activity
- finish reason
- usage
- turn failure boundary
- thinking payload when present

This follows the Product TDD rule that full chat payloads needed for source-grounded
investigation must remain in chat-capability records rather than being pushed down into generic
transport telemetry.

### Correlation rule

When Cortex drives context control:

- related AI Gateway records must preserve `organ_id`
- transport records still correlate through `request_id` and parentage
- thread/turn records must remain sufficient for drilldown without parsing prose

### Explicit non-goal

Do not create a separate "context-control observability layer" unless the runtime later grows a
truly separate ownership boundary.

For now, derive/rewrite semantics belong to thread lifecycle observability.

## 14. Explicit Non-Freezes

This consolidated freeze still does **not** freeze:

1. exact Rust receiver syntax for mutating methods
2. rich provider-context channel design
3. rich phase-aware retry contract tied to streaming/resume
4. future `asr` / `tts` public contracts
5. exact file migration order in production code

## Working Conclusion

The broader design is now strong enough for one coherent follow-up implementation baseline.

The next step should be:

1. treat this file as the active contract baseline
2. slice the first implementation against it
3. defer provider-context/retry overdesign until the runtime actually needs it
