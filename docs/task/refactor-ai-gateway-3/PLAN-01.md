# PLAN-01 — AI Gateway Refactor Round 3

> 2026-02-27. Addresses S1, S2, S5, S6 from BAD-SMELL.md.

## Scope

Architectural reset of AI Gateway's chat surface: rename entities, delete CanonicalRequest, restructure adapters, introduce Dispatcher.

### Out of scope (this round)

- S3 (adapter invoke_stream long functions) — deferred
- S7 (Copilot panel completion abuse) — deferred
- S8 (logging functions in gateway.rs) — deferred
- S10 (aggregate_once_events duplication) — deferred
- Test migration — per AGENTS.md, just make build pass

---

## Entity Rename

| Old | New | Rationale |
|---|---|---|
| `ChatGateway` | deleted | Replaced by `Chat` as entry point |
| `ChatSessionHandle` | `Chat` | Tools/system_prompt/route aggregate — the "definition" |
| `ChatSessionOpenRequest` | `ChatOpenRequest` | |
| `ChatThreadHandle` | `Thread` | Runtime conversation, derived from Chat |
| `ChatThreadOpenRequest` | `ThreadOpenRequest` | |
| `ChatTurnRequest` | `TurnInput` | |
| `ChatTurnResponse` | `TurnOutput` | |
| `ChatThreadState` | `ThreadState` | |
| `BelunaMessage` | `ChatMessage` | Beluna has no top-level Message concept |
| `BelunaRole` | `ChatRole` | |
| `BelunaContentPart` | `ContentPart` | |
| `BelunaMessageToolCall` | `MessageToolCall` | |
| `BelunaToolDefinition` | `ChatToolDefinition` | |
| `AIGateway` | `ChatDispatcher` (internal) | No longer public Chat API |
| `InMemoryChatSessionStore` | `ThreadStore` | |
| `request_id` (in turn) | deleted | `thread_id` + `turn_id` sufficient |
| `CanonicalRequest` | deleted | |
| `CanonicalMessage`, etc. | deleted | Adapters consume `ChatMessage` directly |
| `RequestNormalizer` | deleted | Validation moves to Chat/Turn + Tool trait |
| `ResponseNormalizer` | deleted (inlined) | Trivial 1:1 mapping absorbed by Dispatcher |
| `ChatRequest` | deleted | Was the intermediate flat DTO |
| `ToolChoice` | deleted | Tool override via Thread/Turn tool list |

## New Module Structure

```
core/src/ai_gateway/
├── mod.rs                  # re-exports
├── chat.rs                 # Chat, Thread (public API)
├── thread_store.rs         # ThreadStore (internal)
├── tool.rs                 # ChatToolDefinition, Tool validation
├── types.rs                # Config types (unchanged), ChatMessage, ChatRole, etc.
├── dispatcher.rs           # ChatDispatcher (replaces gateway.rs)
├── error.rs                # unchanged
├── router.rs               # minor: accept route_hint &str instead of &CanonicalRequest
├── capabilities.rs         # minor: accept ChatMessage/tool slices
├── budget.rs               # minor: accept route + limits instead of CanonicalRequest
├── reliability.rs          # unchanged
├── telemetry.rs            # unchanged
├── credentials.rs          # unchanged
└── adapters/
    ├── mod.rs              # BackendAdapter trait (revised signature)
    ├── http_errors.rs      # extracted from http_common.rs
    ├── openai_compatible/
    │   ├── mod.rs
    │   ├── chat.rs         # OpenAI adapter impl
    │   └── wire.rs         # extracted from http_common: openai serialization
    ├── ollama/
    │   ├── mod.rs
    │   ├── chat.rs         # Ollama adapter impl
    │   └── wire.rs         # extracted from http_common: ollama serialization
    └── github_copilot/
        ├── mod.rs
        ├── chat.rs
        └── rpc.rs

Deleted files:
  - chat/api.rs, chat/mod.rs, chat/types.rs, chat/session_store.rs
  - types_chat.rs (merged into types.rs + tool.rs)
  - request_normalizer.rs
  - response_normalizer.rs
  - gateway.rs (replaced by dispatcher.rs)
  - adapters/http_common.rs (split into wire.rs + http_errors.rs)
```

## Key Design Decisions

### D1. Chat and Thread

```rust
pub struct Chat {
    dispatcher: Arc<ChatDispatcher>,
    store: Arc<ThreadStore>,
    chat_id: String,
    tools: Vec<ChatToolDefinition>,
    system_prompt: Option<String>,
    default_route: Option<String>,
    default_turn_timeout_ms: u64,
}

impl Chat {
    pub async fn open_thread(&self, request: ThreadOpenRequest) -> Result<Thread, GatewayError>;
}

pub struct Thread {
    chat: Arc<ChatInner>,          // shared Chat config
    store: Arc<ThreadStore>,
    thread_id: String,
}

impl Thread {
    pub async fn complete(&self, input: TurnInput) -> Result<TurnOutput, GatewayError>;
    pub async fn stream(&self, input: TurnInput) -> Result<TurnStream, GatewayError>;
    pub async fn state(&self) -> Result<ThreadState, GatewayError>;
}
```

### D2. Tool override model

Tools are defined on Chat. Thread/Turn can:

- **Add** tools not in Chat's set
- **Override** tools by name (same name replaces definition)
- **Remove** tools by providing a sentinel (e.g., tool with `remove: true` marker, or a dedicated `ToolOverride` enum)

No `ToolChoice` enum. The effective tool set is: `chat.tools ← thread.tool_overrides ← turn.tool_overrides` (last writer wins per name).

### D3. TurnPayload (adapter contract)

```rust
pub(crate) struct TurnPayload {
    pub messages: Arc<Vec<ChatMessage>>,
    pub tools: Vec<ChatToolDefinition>,
    pub output_mode: OutputMode,
    pub limits: TurnLimits,
    pub enable_thinking: bool,
    pub metadata: BTreeMap<String, String>,
}
```

### D4. BackendAdapter revised

```rust
#[async_trait]
pub trait BackendAdapter: Send + Sync {
    fn dialect(&self) -> BackendDialect;
    fn static_capabilities(&self) -> BackendCapabilities;
    fn supports_tool_retry(&self) -> bool { false }

    async fn complete(
        &self, ctx: AdapterContext, payload: &TurnPayload,
    ) -> Result<BackendOnceResponse, GatewayError>;

    async fn stream(
        &self, ctx: AdapterContext, payload: &TurnPayload,
    ) -> Result<AdapterInvocation, GatewayError>;
}
```

### D5. ChatDispatcher (internal)

```rust
pub(crate) struct ChatDispatcher {
    router: BackendRouter,
    credential_provider: Arc<dyn CredentialProvider>,
    adapters: HashMap<BackendDialect, Arc<dyn BackendAdapter>>,
    capability_guard: CapabilityGuard,
    budget_enforcer: BudgetEnforcer,
    reliability: ReliabilityLayer,
}

impl ChatDispatcher {
    pub(crate) async fn complete(&self, payload: &TurnPayload, route: Option<&str>)
        -> Result<TurnDispatchResult, GatewayError>;
    pub(crate) async fn stream(&self, payload: &TurnPayload, route: Option<&str>)
        -> Result<ChatEventStream, GatewayError>;
}
```

### D6. Messages as Arc

`ThreadStore` internally holds `Arc<Vec<ChatMessage>>`. When preparing a turn, it returns the Arc (for history) plus the new input messages. The dispatcher receives `TurnPayload` with `messages: Arc<Vec<ChatMessage>>` — retry loop clones the Arc pointer, not the messages.

---

## Implementation Steps

1. **New types** — Create `types.rs` additions (ChatMessage, ChatRole, ContentPart, etc.), `tool.rs`, delete old Beluna/Canonical parallel types
2. **Dispatcher** — Port `gateway.rs` dispatch logic into `dispatcher.rs`, consuming `TurnPayload`
3. **Chat/Thread/Turn API** — New `chat.rs` with Chat + Thread, `thread_store.rs`
4. **Adapter interface** — Update trait to `complete`/`stream` with `&TurnPayload`
5. **Wire migration** — Move serialization from `http_common.rs` into per-adapter `wire.rs`, create `http_errors.rs`
6. **Adapter implementations** — Update OpenAI, Ollama, Copilot to use ChatMessage + wire.rs
7. **Router/CapabilityGuard/Budget** — Update to not depend on CanonicalRequest
8. **Delete old code** — Remove CanonicalRequest, RequestNormalizer, ResponseNormalizer, ChatRequest, old chat/ dir, types_chat.rs, http_common.rs
9. **External callers** — Update cortex/runtime.rs, main.rs, config.rs imports
10. **Build** — Iterate until `cargo build` passes

---

## Callers to Update

| Caller | Change |
|---|---|
| `main.rs` | `AIGateway::new(...)` → `ChatDispatcher::new(...)`, wrap in `Chat::new(...)` |
| `config.rs` | `AIGatewayConfig` unchanged |
| `cortex/runtime.rs` | Session/Thread/Turn API rename; `BelunaMessage` → `ChatMessage`; `ChatSessionOpenRequest` → `ChatOpenRequest`; drop `request_id` from turns |
| `cortex/helpers/*.rs` | `OutputMode` import path change; `BelunaMessage` → `ChatMessage` |
| `cortex/helpers/mod.rs` | `ChatResponse` import path change |
