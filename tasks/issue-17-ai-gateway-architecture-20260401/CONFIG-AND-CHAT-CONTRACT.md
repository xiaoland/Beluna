# Config And Chat Contract Draft

## Purpose

This file refines two specific low-level design areas:

- shared provider inventory config
- OOP-style chat contract for `Cortex`

It exists to prevent these decisions from being re-expanded into vague architecture language.

## 1. Config Direction

## Non-goal

Do not create one mega provider object that mixes:

- provider connection data
- chat model routing
- asr decoding settings
- tts voice settings
- provider-native prompt ids
- provider-native mcp/session details for every capability

That shape is superficially centralized but actually unreadable.

## Recommended split

```text
ai_gateway:
  providers:
    - id
      family
      endpoint
      credential
      declared_capability_families
      transport

  chat:
    routes:
      - alias
        binding
    bindings:
      - id
        provider
        target
        native_features
        fallback_policy

  asr:
    ...

  tts:
    ...
```

## Provider inventory example

```json
{
  "ai_gateway": {
    "providers": [
      {
        "id": "openai",
        "family": "openai_responses",
        "endpoint": "https://api.openai.com/v1",
        "credential": { "type": "env", "var": "OPENAI_API_KEY" },
        "declared_capability_families": ["chat", "asr", "tts"],
        "transport": {
          "kind": "http"
        }
      }
    ]
  }
}
```

What belongs here:

- provider identity
- bootstrap connection facts
- auth facts
- coarse capability-family declaration

What does not belong here:

- `model`
- `voice`
- chat route alias
- prompt object ids used only by chat

## Chat binding example

```json
{
  "ai_gateway": {
    "chat": {
      "routes": [
        { "alias": "default", "binding": "primary" },
        { "alias": "cortex_primary", "binding": "primary" },
        { "alias": "cortex_helper", "binding": "helper" }
      ],
      "bindings": [
        {
          "id": "primary",
          "provider": "openai",
          "target": {
            "kind": "model",
            "model": "gpt-5"
          },
          "native_features": {
            "hosted_tools": true,
            "remote_mcp": true,
            "prompt_objects": true
          },
          "fallback_policy": {
            "canonical_thread_authority": "beluna",
            "allow_provider_thread_cache": true
          }
        }
      ]
    }
  }
}
```

Key point:

- the shared inventory is no longer model-centric
- chat bindings may still be model-centric, because chat genuinely routes by model today

## 2. OOP Chat Contract

## Non-goal

Do not let `Cortex` use:

- provider-native thread ids
- raw turn cloning primitives
- direct thread storage mutation helpers

## Recommended object model

`Cortex` should see capability objects, but the write-path object should be `Thread`, not `Turn`.

Therefore the recommended direction is not to add a new `ChatDialogue` wrapper.

Example direction:

```rust
pub trait ChatCapability: Send + Sync {
    async fn open_thread(&self, spec: ThreadSpec) -> Result<ThreadHandle, ChatError>;
    async fn restore_thread(&self, snapshot: ThreadSnapshot) -> Result<ThreadHandle, ChatError>;
}

pub struct AppendRequest {
    pub messages: Vec<UserMessage>,
    pub options: AppendOptions,
}

pub struct AppendOptions {
    pub tool_overrides: Vec<ToolOverride>,
    pub output_mode: Option<OutputMode>,
    pub limits: Option<TurnLimits>,
    pub enable_thinking: Option<bool>,
    pub metadata: BTreeMap<String, String>,
}

#[async_trait::async_trait]
pub trait ThreadHandle: Send + Sync {
    async fn append(&mut self, request: AppendRequest) -> Result<AppendMessagesResult, ChatError>;
    async fn append_message(&mut self, message: UserMessage) -> Result<AppendMessagesResult, ChatError>;
    async fn append_messages(&mut self, messages: Vec<UserMessage>) -> Result<AppendMessagesResult, ChatError>;
    async fn rewrite_context(&mut self, request: ThreadRewriteRequest) -> Result<ThreadRewriteResult, ChatError>;
    async fn inspect_turns(&self, query: TurnQuery) -> Result<Vec<TurnSnapshot>, ChatError>;
    async fn snapshot(&self) -> Result<ThreadSnapshot, ChatError>;
}
```

The implementation may still use the concrete `Thread` type directly instead of a trait object if that stays readable.

This keeps:

- OOP style
- long-lived thread identity
- capability-local ownership
- existing thread/turn/message semantics
- write-path focus on `Thread`

while avoiding:

- extra naming-only wrapper layers
- direct exposure of thread storage surgery
- provider-native object leakage

## Thread-level public API direction

If `Thread` remains the main OOP object, its public surface should become narrower than today.

Public operations should be close to:

- `append(...)`
- `append_message(...)`
- `append_messages(...)`
- `rewrite_context(...)`
- `inspect_turns(...)`
- `snapshot()`

Operations that should not remain general external surface:

- raw append of already-built turns
- external turn reindexing expectations
- clone-by-selected-turns storage surgery

Those behaviors may still exist internally inside the chat capability.

## Thread Context Rewrite Contract

`rewrite_context(...)` should not be a generic arbitrary-edit API.

That would:

- leak storage surgery into callers
- weaken message and turn integrity constraints
- encourage provider- or prompt-specific hacks in caller code

Instead, it should expose a small closed set of semantic rewrite operations.

### Minimum required semantics

Based on current `Cortex` usage, the minimum required rewrite semantics are:

1. replace system prompt / system instructions
2. retain only selected committed turns
3. retain only the last N committed turns
4. drop unfinished continuation state
5. atomically produce one new canonical thread state

This is enough to cover the existing reset-style behavior that currently uses:

- select a subset of prior turns
- replace system prompt
- rebuild primary thread state

### Recommended request shape

```rust
pub struct ThreadRewriteRequest {
    pub new_system_prompt: Option<String>,
    pub retention: TurnRetentionPolicy,
    pub drop_unfinished_continuation: bool,
    pub reason: ThreadRewriteReason,
}

pub enum TurnRetentionPolicy {
    KeepAll,
    KeepLastTurns { count: usize },
    KeepSelectedTurnIds { turn_ids: Vec<u64> },
    DropAll,
}
```

### Recommended result shape

```rust
pub struct ThreadRewriteResult {
    pub thread_id: String,
    pub kept_turn_ids: Vec<u64>,
    pub dropped_turn_ids: Vec<u64>,
    pub continuation_dropped: bool,
}
```

### Explicit non-goals

`rewrite_context(...)` should not expose:

- arbitrary message splice/edit operations
- arbitrary turn reordering beyond explicit retention selection
- caller-managed turn reindexing rules
- raw provider-native rewrite primitives

If any of those become necessary, they should first be justified as stable chat semantics rather than convenience escape hatches.

## 3.5 Minimal Public Methods for `Thread / Turn / Message`

The user's latest clarification is accepted in this form:

- `Cortex` may directly operate `Thread` and create `Message`
- `Turn` remains an internal semantic unit for runtime bookkeeping and read-side inspection
- `Cortex` should not directly call `Turn` mutation methods

If unrestricted mutation is exposed broadly, readability and maintainability will degrade.
That would justify a separate trade-off discussion before implementation.

### `Thread`

`Thread` is the main long-lived OOP object.

Recommended public methods:

```rust
impl Thread {
    pub fn thread_id(&self) -> &ThreadId;
    pub async fn append(&mut self, request: AppendRequest) -> Result<AppendMessagesResult, ChatError>;
    pub async fn append_message(&mut self, message: UserMessage) -> Result<AppendMessagesResult, ChatError>;
    pub async fn append_messages(&mut self, messages: Vec<UserMessage>) -> Result<AppendMessagesResult, ChatError>;
    pub async fn rewrite_context(&self, request: ThreadRewriteRequest) -> Result<ThreadRewriteResult, ChatError>;
    pub async fn inspect_turns(&self, query: TurnQuery) -> Result<Vec<TurnSnapshot>, ChatError>;
    pub async fn snapshot(&self) -> Result<ThreadSnapshot, ChatError>;
}
```

Meaning:

- `append(...)` is the canonical thread-centric write operation when per-append execution options are needed
- `append_messages(...)` is the canonical ordinary write-path transaction
- one `append_messages(...)` call creates one internal turn input transaction and advances the thread
- `append_message(...)` and `append_messages(...)` are sugar over `append(...)`
- `rewrite_context(...)` is the canonical way to perform reset/trim/rebase style operations
- `inspect_turns(...)` is read-oriented inspection, not storage surgery
- `snapshot()` is for recovery, observability, and replay

Important constraint:

- there should be no long-lived public "open turn" state between calls
- if a caller wants to send multiple user messages as one logical input batch, it uses `append_messages(...)`
- turn creation and commitment stay internal to `Thread`
- if a caller needs dynamic tool/output/limit overrides for one append, it uses `append(...)`, not a public `TurnInput`

## Append Result Contract

Accepted direction:

- `append_messages(...)` does not return `Turn`
- it returns a very thin result envelope whose primary payload is the newly produced messages

Why this is the correct compromise:

- it keeps the write API thread-centric
- it avoids leaking internal turn structure back into ordinary callers
- it still preserves essential result semantics such as completion and usage

Accepted contract direction:

```rust
pub struct AppendMessagesResult {
    pub new_messages: Vec<Message>,
    pub finish_reason: Option<FinishReason>,
    pub usage: Option<UsageStats>,
}
```

This keeps the API compact while avoiding an under-specified "just messages" surface.

## Append Input Contract Correction

The earlier sketch that accepted `Message` on the ordinary write path is too permissive.

That shape would make the API self-contradictory:

- the contract says only `user` input is valid
- the type system would still invite callers to pass `assistant/tool/system/tool_result`
- the actual rule would move from API shape into runtime rejection

That is a readability and maintainability downgrade.

Therefore the ordinary write-path input should be narrowed to `UserMessage`, not `Message`.

Accepted direction:

- `append_message(...)` accepts `UserMessage`
- `append_messages(...)` accepts `Vec<UserMessage>`
- broader `Message` construction still exists for snapshot import, replay, testing, and read-side inspection

If per-append controls are needed, they should be expressed as thread-centric `AppendOptions`, not as a public `TurnInput`.

## ThreadSpec And AppendOptions Draft

The next contract should separate stable thread semantics from per-operation control knobs.

Recommended minimum shape:

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

pub struct AppendOptions {
    pub tool_overrides: Vec<ToolOverride>,
    pub output_mode: Option<OutputMode>,
    pub limits: Option<TurnLimits>,
    pub enable_thinking: Option<bool>,
    pub metadata: BTreeMap<String, String>,
}
```

Design rule:

- `ThreadSpec` owns stable thread semantics
- `AppendOptions` owns one-append execution adjustments
- observability metadata belongs to operations, not durable thread identity

Important refinement:

- `ThreadSpec.route_ref` is route-selection input, not persisted canonical identity
- the stored thread state and exported snapshot should always carry an explicit resolved `route_key`
- config-level default-route fallback may still exist at construction time
- but once a `Thread` exists, route ambiguity should be gone

Tool rule:

- `ThreadSpec.tools` is the baseline thread tool inventory
- `AppendOptions.tool_overrides` is a one-append delta, not long-lived thread mutation

This separation matches current `Cortex` needs without forcing `Cortex` back into turn-centric APIs.

## Append Transaction Semantics

One append call should equal one committed thread transaction and one committed turn.

Internally, that transaction may still perform multiple runtime steps:

1. validate the input batch
2. create one draft internal turn
3. call the selected backend
4. append runtime-owned assistant/tool messages
5. execute tool calls if required
6. continue backend execution if the tool cycle requires continuation
7. validate turn completeness
8. commit the completed turn atomically

This is the cleaner boundary:

- `Thread` owns runtime orchestration
- `Turn` owns turn-local invariants
- callers only see committed canonical results

Strong recommendation:

- there should be no public `pending_tool_call_continuation` protocol exposed to `Cortex`
- if a tool cycle cannot be completed, the append call should fail and commit nothing
- any partial provider output may still be logged for observability, but must not become committed canonical conversation state

Otherwise AI Gateway would still be leaking its internal continuation protocol into `Cortex`.

Methods that should not be public general-purpose APIs:

- raw `append_turn(...)`
- `clone_thread_with_turns(...)` as an externally orchestrated primitive
- `truncate_last_unit(...)` as a caller-facing history surgery primitive
- reindexing-sensitive mutation helpers

## Canonical Tool Message Representation

Committed chat history should have one canonical Beluna representation for tool activity.

The current dual model is weak:

- `AssistantMessage.tool_calls`
- standalone `ToolCallMessage` plus `ToolCallResultMessage`

Keeping both as first-class committed shapes would make `Turn` invariants ambiguous.

Recommended canonical rule:

- committed turn history uses explicit `ToolCallMessage` and `ToolCallResultMessage`
- `AssistantMessage` remains an assistant-content message, not the canonical owner of tool-call linkage
- if provider output arrives in a provider-native embedded format, the backend bridge normalizes it before commitment

This is the cleaner long-term model because:

- tool-call/result pairing becomes explicit
- truncation semantics stay simple
- snapshots and observability become easier to reason about
- provider-native wire format does not leak into Beluna canonical history

## Snapshot And Restore Contract

If conversation state remains internal to AI Gateway, `snapshot()` and `restore_thread(...)` must define the export boundary clearly.

Recommended minimum shape:

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

Important rules:

- snapshots export only committed canonical turns
- snapshots do not export a public open-turn or half-complete tool continuation state
- restore must succeed from canonical state alone, without requiring hidden provider authority
- provider-native caches or thread handles may exist only as optional execution hints, never as required restore inputs

## Stable Turn Identity Rule

`rewrite_context(...)` should preserve the original ids of kept turns.

That means:

- keeping turns must not imply reindexing them
- turn ids are stable observability handles, not disposable storage positions
- future internal storage compaction may change indexes, but not committed turn ids

This is a direct criticism of any clone/reset approach that reassigns turn ids during history surgery.
That behavior is convenient for storage but harmful for debugging, auditability, and log correlation.

Methods that should not be public general-purpose APIs:

- raw `append_turn(...)`
- `clone_thread_with_turns(...)` as an externally orchestrated primitive
- reindexing-sensitive mutation helpers

### `Turn`

`Turn` should be treated as an internal semantic unit with strong invariants.

It still owns important semantics:

- message ordering inside one turn
- tool-call/result linkage
- atomic removal and truncation behavior inside one turn
- completion metadata

But it is not part of the ordinary caller write-path.

Clarification:

- "caller should not be aware of `Turn`" is accepted only for the write path
- on the read / inspection path, `TurnSnapshot` or read-only `Turn` views remain useful and should not be artificially hidden

Otherwise observability and debugging would become worse for little gain.

Recommended public methods:

```rust
impl Turn {
    pub fn turn_id(&self) -> u64;
    pub fn messages(&self) -> &[Message];
    pub fn metadata(&self) -> &TurnMetadata;
    pub fn usage(&self) -> Option<&UsageStats>;
    pub fn finish_reason(&self) -> Option<&FinishReason>;
    pub fn completed(&self) -> bool;
    pub fn has_tool_calls(&self) -> bool;
}
```

Internal mutation rule:

- ordinary callers do not mutate `Turn` directly
- `Thread` mutates `Turn` internally
- if `Turn` detects an incomplete tool-call linkage, it should return an error to its caller rather than silently repairing itself
- `Thread` is then responsible for supplying the matching tool-call result message before the turn can be considered valid or committed

This keeps the invariant owner and the repair owner separate:

- `Turn` owns invariants
- `Thread` owns orchestration

Internal truncation rule:

- if the tail unit is a `tool_call_result`, truncation removes the matching `tool_call` together with it
- this behavior remains turn-owned, but it is not part of the ordinary external write API

What should not be public broad-surface behavior:

- direct internal vector mutation
- direct tool-linkage repair methods from outside runtime
- arbitrary field-by-field message surgery
- caller-managed completion bookkeeping that bypasses thread/turn invariants

Those are runtime integrity operations, not ordinary caller semantics.

### `Message`

`Message` should be fully inspectable and constructible, but not broadly mutable in-place.

Recommended public methods:

```rust
impl Message {
    pub fn id(&self) -> &MessageId;
    pub fn kind(&self) -> MessageKind;
    pub fn created_at_ms(&self) -> u64;
    pub fn as_system(&self) -> Option<&SystemMessage>;
    pub fn as_user(&self) -> Option<&UserMessage>;
    pub fn as_assistant(&self) -> Option<&AssistantMessage>;
    pub fn as_tool_call(&self) -> Option<&ToolCallMessage>;
    pub fn as_tool_result(&self) -> Option<&ToolCallResultMessage>;
}
```

Recommended construction helpers:

- `Message::system(...)`
- `Message::user(...)`
- `Message::assistant(...)`
- `Message::tool_call(...)`
- `Message::tool_result(...)`

This lets `Cortex` build inputs explicitly while preserving runtime control over turn integrity.

Recommended ordinary-write-path restriction:

- ordinary callers should only append `UserMessage` through `Thread::append_message(...)`
- ordinary callers should only append `Vec<UserMessage>` through `Thread::append_messages(...)`
- `system` changes should usually go through `ThreadSpec` or `rewrite_context(...)`
- `assistant/tool_call/tool_result` messages are normally runtime-owned even if the constructors exist for import, replay, or specialized paths

This is now the accepted contract direction.

## 5. Ordinary Write-Path Permission Matrix

For ordinary caller-facing thread advancement:

```text
Thread::append_message(...)
Thread::append_messages(...)
  - UserMessage: allowed
  - SystemMessage: not allowed
  - AssistantMessage: not allowed
  - ToolCallMessage: not allowed
  - ToolCallResultMessage: not allowed
```

Why this is the correct narrowing:

- ordinary caller intent enters through user messages
- system mutation belongs to thread creation or context rewrite
- assistant and tool lifecycle belong to runtime orchestration
- this keeps the write path simple and prevents caller-side semantic drift

## 3. Internal State Ownership Rule

Conversation state may remain entirely inside AI Gateway only if:

1. the internal state is Beluna canonical state
2. it is queryable or snapshot-able in canonical form
3. it is recoverable or reconstructable without hidden provider authority
4. observability can explain the dialogue using Beluna semantics

If any of these are false, then the internal state has become an opaque subsystem and the design should be rejected.

## 3.5 Error Boundary Direction

The public chat-capability surface should not expose raw transport-flavored `GatewayError` as its main error contract.

That would reintroduce backend-first thinking through the type system.

Current bias:

- transport and provider failures may still be carried through from `GatewayError`
- but the public capability-facing contract should prefer `ChatError`
- `ChatError` may wrap `GatewayError` for diagnostics, retry hints, and observability

Minimum error classes the chat contract must distinguish:

- invalid caller input
- unsupported capability feature
- tool execution failure
- thread invariant violation
- delegated backend / transport failure

The exact enum is still open, but the direction is now narrower:

- `append(...)` failure taxonomy should be capability-first
- backend error details remain attached, not authoritative

## 4. Unified Routing Reference / Key Syntax

The system needs one unified alias-reference grammar, while still separating alias from resolved key.

After review, the simpler choice is also the better one:

- always use a canonical `<capability>.<alias>` alias reference string

Reason:

- distinguishing "local alias allowed here" versus "global route required here" adds complexity
- the readability gain from that distinction is too small
- one global canonical grammar is easier to teach, validate, log, and search

## Routing design rule

The canonical alias reference always identifies:

1. capability
2. capability-local alias

It should not canonically identify:

- raw provider id
- raw model id
- provider-native object ids

## Recommended canonical syntax

Preferred syntax everywhere:

- `<capability>.<alias>`

Examples:

- `chat.default`
- `chat.cortex_primary`
- `chat.cortex_helper`
- `asr.default`
- `tts.default`

Why this is acceptable despite some redundancy:

- caller code stays capability-oriented
- provider layout can change without changing callers
- syntax remains readable and short
- the system avoids conditional route grammar rules
- validation and observability become simpler

## Internal representation

Inside Rust code, route references should not remain raw strings once they cross nontrivial boundaries.

Preferred internal shapes:

```rust
pub struct ChatRouteAlias {
    pub capability: String,
    pub alias: String,
}

pub struct ChatRouteKey {
    pub capability: String,
    pub binding_id: String,
}

pub enum ChatRouteRef {
    Alias(ChatRouteAlias),
    Key(ChatRouteKey),
}
```

Meaning:

- string grammar is for config, CLI, and human-authored files
- alias is caller-facing selection
- key is resolved stable runtime identity
- snapshots/persisted thread state store key, not alias

This avoids rebuilding the old ambiguity under a new string syntax.

## Non-canonical escape hatch

Direct provider targeting may still exist for debugging or development, but it should not be the canonical public route grammar.

If retained, it should be explicitly marked as debug/override syntax.

Example:

- `provider:openai/model:gpt-5`

or some similarly explicit override form.

It should not be mixed with the canonical route grammar because that would blur authority again.
