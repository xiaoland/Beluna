# AI Gateway — Bad Smells (Refactor Round 3)

> Explored 2026-02-27. All findings are based on code in `core/src/ai_gateway/`.
> Updated 2026-02-28 after PLAN-01 refactor (Session→Chat, Thread derives from Chat, CanonicalRequest eliminated, chat/ submodule consolidated).

---

## S1. ~~CanonicalRequest 破坏了 Session/Thread/Turn 的 OO API 设计~~ — RESOLVED

**Status: Resolved in PLAN-01**

- `CanonicalRequest` deleted; replaced by `TurnPayload` (pub(crate), internal to chat module).
- Messages are `Arc<Vec<ChatMessage>>` — zero-copy across retry loop.
- Session/Thread IDs are first-class fields in `Chat`/`Thread`, not metadata hacks.
- Retry loop clones only lightweight `TurnPayload` (messages share Arc).

---

## S2. ~~`adapters/http_common.rs` — Mixed Concerns~~ — RESOLVED

**Status: Resolved in PLAN-02**

- `http_common.rs` deleted. Split into focused modules:
  - `http_errors.rs` — pure HTTP status → `GatewayError` mapping (`map_http_error`).
  - `http_stream.rs` — generic HTTP streaming utilities (`HttpRequestConfig`, `send_post`, `post_json`, `extract_sse_frames`, `extract_ndjson_frames`). Zero chat-type imports.
  - `wire.rs` — protocol-neutral helpers shared across adapters (`role_to_wire`, `parse_finish_reason`).
  - `openai_compatible/wire.rs` — OpenAI-specific serialization (`messages_to_openai`, `tools_to_openai`, content part helpers).
  - `ollama/wire.rs` — Ollama-specific serialization (`messages_to_ollama`, `tools_to_ollama`, content part helpers).
- Per-adapter wire helpers kept deliberately separate (not unified) — necessary boilerplate improves maintainability when formats diverge.

---

## S3. ~~Adapter 内部 `invoke_stream` 超长函数~~ — RESOLVED

**Status: Resolved in PLAN-02**

- `http_stream.rs` extracts `send_post()` (auth headers, timeout, HTTP error mapping) and frame parsers (`extract_sse_frames`, `extract_ndjson_frames`).
- `stream()` spawn closures reduced from ~300 to ~60 lines per adapter — only protocol-specific parsing logic remains.
- `complete()` simplified: uses `http_stream::post_json()` + direct `parse_complete_response()` → `BackendCompleteResponse`.
  - Eliminates the intermediate `Vec<BackendRawEvent>` for the non-stream path (no more `aggregate_once_events`).
- Body construction extracted into per-adapter `build_body()` helpers.
- URL validation extracted into per-adapter `validated_url()` helpers.

---

## S4. ~~`gateway.rs` 中 `dispatch_once` 与 `run_stream_task` 的大量重复~~ — RESOLVED

**Status: Resolved in PLAN-01**

- `gateway.rs` deleted; replaced by `chat/dispatcher.rs` with unified `ChatDispatcher`.
- Pipeline stages (route → credential → capability → budget → reliability) are shared between `complete()` and `stream()`.
- Retry loop extracted into a single path.

---

## S5. ~~`CanonicalRequest` 与 `ChatRequest` 几乎同构的类型冗余~~ — RESOLVED

**Status: Resolved in PLAN-01**

- Both `CanonicalRequest` and `ChatRequest` deleted.
- Single type chain: `TurnInput` (caller-facing) → `TurnPayload` (internal, pub(crate)) → adapter.
- `RequestNormalizer` deleted — no more mechanical field-by-field copy.

---

## S6. ~~`ResponseNormalizer` 无实质逻辑~~ — RESOLVED

**Status: Resolved in PLAN-01**

- `ResponseNormalizer` deleted.
- `BackendRawEvent` → `ChatEvent` mapping is now inline in `chat/dispatcher.rs::map_raw_event()`.
- `is_output()`, `is_tool()`, `is_terminal()` are methods on `ChatEvent` itself.

---

## S7. `GitHubCopilotAdapter` — Panel Completion 滥用

**Severity: Medium (correctness)**

### 问题

`GitHubCopilotAdapter` 把 **Chat messages** 强行拼接成纯文本，然后送给 Copilot LS 的 `textDocument/copilotPanelCompletion` / `textDocument/inlineCompletion` API。这不是 Chat API — 这是代码补全 API。

```rust
fn build_panel_completion_params(req: &CanonicalRequest) -> Value {
    let text = req.messages.iter()
        .flat_map(|message| message.parts.iter())
        .filter_map(|part| match part {
            CanonicalContentPart::Text { text } => Some(text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n");
    // ...送给 panelCompletion
}
```

后果：

- 丢失了所有消息结构（role, tool_calls, tool_call_id）。
- Copilot 返回的是代码补全，不是 Chat 回复。
- `invoke_once` 通过 `invoke_stream` + collect 实现，每次 spawn 一个子进程 → kill。

### 建议

如果 Copilot adapter 真的需要保留，应该使用 Copilot Chat API（`@github/copilot-language-server` 的 chat 通道），或者清晰标记这个 adapter 只支持 "completion" 能力而非 "chat"。

---

## S8. ~~Logging 函数散布在 `gateway.rs` 底部~~ — RESOLVED

**Status: Resolved in PLAN-01**

- `gateway.rs` deleted. Logging is now in `chat/api.rs` (`emit_turn_summary`, `record_turn_metrics`).

---

## S9. ~~`types_chat.rs` — 两套平行类型体系共存~~ — RESOLVED

**Status: Resolved in PLAN-01**

- `types_chat.rs` deleted.
- Single type system in `chat/types.rs` (domain primitives) + `chat/tool.rs` (tool definitions).
- No more Beluna*/Canonical* parallel hierarchies.
- Adapter-level types (`BackendRawEvent`, `AdapterInvocation`, `BackendOnceResponse`) colocated in `chat/types.rs` but clearly separated by section.

---

## S10. ~~缺少 `aggregate_once_events` 共享~~ — RESOLVED

**Status: Resolved in PLAN-02**

- `aggregate_once_events` eliminated entirely from both adapters.
- `complete()` now directly parses the JSON response into `BackendCompleteResponse` — the intermediate `Vec<BackendRawEvent>` representation was unnecessary for the non-stream path.
- `BackendOnceResponse` renamed to `BackendCompleteResponse` for naming consistency (`once` → `complete`).

---

## Summary of Impact on Chat Performance

| Smell | Status | Performance Impact | Description |
|---|---|---|---|
| **S1** | **RESOLVED** | ~~High~~ | ~~CanonicalRequest.clone() in retry loop~~ → `TurnPayload` with `Arc<Vec<ChatMessage>>` |
| **S2** | **RESOLVED** | ~~Low~~ | ~~http_common.rs mixed concerns~~ → split into http_errors, http_stream, wire, per-adapter wire |
| **S3** | **RESOLVED** | ~~Low~~ | ~~Adapter stream functions 300+ lines~~ → extracted send_post, frame parsers; spawn ~60 lines |
| **S4** | **RESOLVED** | ~~Medium~~ | ~~dispatch_once/run_stream_task duplication~~ → unified ChatDispatcher |
| **S5** | **RESOLVED** | ~~Medium~~ | ~~ChatRequest/CanonicalRequest parallel types~~ → single TurnPayload |
| **S6** | **RESOLVED** | ~~Low~~ | ~~ResponseNormalizer no-op~~ → inline map_raw_event + ChatEvent methods |
| **S7** | Open | Medium | Copilot adapter still sends flattened text to completion API |
| **S8** | **RESOLVED** | ~~Low~~ | ~~Logging scattered in gateway.rs~~ → chat/api.rs helpers |
| **S9** | **RESOLVED** | ~~Medium~~ | ~~two parallel type systems~~ → single chat/types.rs |
| **S10** | **RESOLVED** | ~~Low~~ | ~~aggregate_once_events duplicated~~ → eliminated; complete() parses directly |
