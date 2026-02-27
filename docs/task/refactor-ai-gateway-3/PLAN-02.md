# PLAN-02 — Adapter Layer Deduplication

> 2026-02-27. Addresses S2 (http_common mixed concerns), S3 (adapter stream long functions), and S10 (aggregate_once_events) from BAD-SMELL.md.
> **Implemented 2026-02-28.** Build passes with zero errors/warnings.

## Goal

Reduce copy-paste between OpenAI-compatible and Ollama adapters by extracting shared mechanical code into reusable building blocks, **without** introducing generics or abstraction that obscures the per-adapter protocol logic.

### Out of scope

- S7 (Copilot adapter correctness) — different problem class
- S10 (`aggregate_once_events` dedup) — folded in as a side-effect

---

## Problem Analysis

### What is duplicated today

| Code region | OpenAI loc | Ollama loc | Identical? |
|---|---|---|---|
| `tools_to_*()` | http_common L103–114 | http_common L116–128 | **100%** |
| `message_tool_call_to_*()` | http_common L232–240 | http_common L242–250 | **100%** |
| `aggregate_once_events()` | openai/chat.rs L508–552 | ollama/chat.rs L416–448 | **~95%** (only error message differs) |
| `stream()` scaffolding — cancel_flag, mpsc channel, spawn, http send, error mapping, terminal guard | openai/chat.rs L175–330 | ollama/chat.rs L150–290 | **~80%** structural |

### What differs

| Concern | OpenAI | Ollama |
|---|---|---|
| **Endpoint path** | `/chat/completions` | `/api/chat` |
| **Message wire format** | `messages_to_openai()` — multi-content parts, image_url | `messages_to_ollama()` — plain text join |
| **Stream framing** | SSE (`data: …\n`, `[DONE]`) | NDJSON (one JSON object per line) |
| **Stream payload parser** | `parse_stream_payload()` — reads `choices[].delta` | `parse_ollama_payload()` — reads `message.content`, `done` flag |
| **Non-stream response parser** | `parse_non_stream_payload()` — reads `choices[0].message` | `parse_ollama_payload()` same function for both |
| **`tool_choice`** | sends `"tool_choice": "auto"` | not sent |
| **Body extras** | `response_format`, `max_tokens`, `thinking.budget_tokens` | `options.num_predict`, `think: true` |
| **Usage field names** | `prompt_tokens` / `completion_tokens` | `prompt_eval_count` / `eval_count`, inside `done: true` payload |

---

## Design (as implemented)

### D1. Per-adapter wire helpers — KEPT SEPARATE (rejected unification)

Original plan proposed merging `tools_to_openai` / `tools_to_ollama` into `tools_to_wire`. **Rejected** — necessary boilerplate improves maintainability when formats diverge. Each adapter has its own `wire.rs`:

- `openai_compatible/wire.rs` — `messages_to_openai()`, `tools_to_openai()`, multi-content part helpers.
- `ollama/wire.rs` — `messages_to_ollama()`, `tools_to_ollama()`, plain-text content helpers.

### D2. Eliminate `aggregate_once_events` entirely (not just deduplicate)

Original plan proposed extracting `aggregate_once_events` into a shared module. **Eliminated instead** — the intermediate `Vec<BackendRawEvent>` representation was unnecessary for the non-stream path. Now:

- `complete()` calls `http_stream::post_json()` → adapter-specific `parse_complete_response()` → `BackendCompleteResponse` directly.
- Naming: `BackendOnceResponse` → `BackendCompleteResponse`, `once_result` → `complete_result`.

### D3. Extract HTTP stream scaffolding to `http_stream.rs`

Implemented as planned with key constraint: **zero chat-type imports** — module only knows `Value`, `GatewayError`, `ResolvedCredential`.

```rust
pub(crate) struct HttpRequestConfig { client, url, body, backend_id, request_id, credential, timeout }
pub(crate) async fn send_post(config) -> Result<Response, GatewayError>     // auth headers + error mapping
pub(crate) async fn post_json(config) -> Result<Value, GatewayError>        // convenience for non-stream
pub(crate) fn extract_sse_frames(buffer, backend_id) -> Result<(Vec<Value>, bool), GatewayError>
pub(crate) fn extract_ndjson_frames(buffer, backend_id) -> Result<Vec<Value>, GatewayError>
```

Note: did not implement `spawn_http_stream()` with `parse_frame` callback. The composable `send_post` + `extract_*_frames` approach is simpler and avoids closure type complexity.

### D4. Extract `map_http_error` to `http_errors.rs` — done as planned

### D5. Split `http_common.rs` into focused modules — done

```
adapters/
├── mod.rs
├── http_errors.rs          # map_http_error()
├── http_stream.rs          # HttpRequestConfig, send_post, post_json, extract_sse_frames, extract_ndjson_frames
├── wire.rs                 # role_to_wire, parse_finish_reason (shared, protocol-neutral)
├── openai_compatible/
│   ├── mod.rs
│   ├── wire.rs             # messages_to_openai, tools_to_openai, content part helpers
│   └── chat.rs             # OpenAI adapter: build_body, parse_complete_response, parse_stream_delta
├── ollama/
│   ├── mod.rs
│   ├── wire.rs             # messages_to_ollama, tools_to_ollama, content part helpers
│   └── chat.rs             # Ollama adapter: build_body, parse_complete_response, parse_stream_delta
└── github_copilot/
    ├── mod.rs
    ├── chat.rs             # Only type rename (BackendOnceResponse → BackendCompleteResponse)
    └── rpc.rs
```

---

## Actual changes by file

### New files

| File | Content | Lines |
|---|---|---|
| `adapters/http_errors.rs` | `map_http_error()` | ~42 |
| `adapters/http_stream.rs` | `HttpRequestConfig`, `send_post`, `post_json`, `extract_sse_frames`, `extract_ndjson_frames` | ~172 |
| `adapters/wire.rs` | `role_to_wire`, `parse_finish_reason` | ~28 |
| `openai_compatible/wire.rs` | `messages_to_openai`, `tools_to_openai`, content part helpers | ~115 |
| `ollama/wire.rs` | `messages_to_ollama`, `tools_to_ollama`, content part helpers | ~85 |

### Modified files

| File | Change |
|---|---|
| `chat/types.rs` | `BackendOnceResponse` → `BackendCompleteResponse` |
| `chat/dispatcher.rs` | `once_result` → `complete_result`, `once_response` → `complete_response` |
| `adapters/mod.rs` | New module declarations; remove `http_common`; update type import |
| `openai_compatible/mod.rs` | Add `pub(crate) mod wire;` |
| `openai_compatible/chat.rs` | **Rewritten** — uses `http_stream`, `openai_wire`, direct `parse_complete_response` |
| `ollama/mod.rs` | Add `pub(crate) mod wire;` |
| `ollama/chat.rs` | **Rewritten** — uses `http_stream`, `ollama_wire`, direct `parse_complete_response` |
| `github_copilot/chat.rs` | `BackendOnceResponse` → `BackendCompleteResponse` (3 occurrences) |

### Deleted files

| File | Reason |
|---|---|
| `adapters/http_common.rs` | Split into http_errors, http_stream, wire, per-adapter wire |

---

## Key design decisions vs. original plan

| Decision | Original plan | Actual | Rationale |
|---|---|---|---|
| Per-adapter wire helpers | Merge into shared `tools_to_wire` | Keep separate per-adapter `wire.rs` | Necessary boilerplate improves maintainability — formats may diverge |
| `aggregate_once_events` | Move to shared module | **Eliminated** | `complete()` directly parses JSON → `BackendCompleteResponse`; intermediate `Vec<BackendRawEvent>` was unnecessary |
| `spawn_http_stream` | Single function with `parse_frame` callback | **Not implemented** | Composable `send_post` + `extract_*_frames` is simpler; avoids closure type complexity |
| `http_stream.rs` scope | Included chat types | **Zero chat-type imports** | Module only knows `Value`, `GatewayError`, `ResolvedCredential` |
