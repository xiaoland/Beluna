# L2-04 - Backend Adapter Specs

- Task Name: `minimal-ai-gateway`
- Stage: `L2` detail: per-backend mapping
- Date: 2026-02-08
- Status: `DRAFT_FOR_APPROVAL`

## 1) Shared Adapter Contract

Each backend adapter must implement:

1. transport setup and auth injection
2. canonical request -> backend request mapping
3. backend stream parsing -> backend raw events
4. backend raw event -> canonical event normalization hooks
5. usage extraction and finish-reason extraction
6. error mapping seed data (status codes/provider codes)
7. in-flight cancellation hook used when consumer drops canonical stream

## 2) OpenAI-Compatible BackendAdapter

Compatibility note:
- OpenAI-compatible means `chat/completions`-like protocol compatibility, not exact OpenAI API parity.
- Missing or divergent provider fields must degrade gracefully when possible.
- Non-reconcilable divergence maps to deterministic canonical errors.

### Transport

- HTTP via `reqwest`.
- Base URL from backend profile (default pattern: `.../v1`).
- Endpoint MVP:
  - `POST /chat/completions` (streaming and non-streaming)

### Request mapping

Canonical -> OpenAI-compatible:

- `model` -> `model`
- `messages` -> `messages`
- `tools` -> `tools`
- `tool_choice` -> `tool_choice`
- `output_mode=json` -> `response_format: {"type":"json_object"}` when supported
- `stream` -> `stream`
- `limits.max_output_tokens` -> `max_tokens`

Headers:

- `Authorization: Bearer <token>`
- `Content-Type: application/json`
- `X-Request-Id: <request_id>`

### Stream parsing

- Parse SSE frames.
- Handle `[DONE]` sentinel.
- Map delta content and delta tool_calls into canonical deltas.
- Capture usage from final chunk when available.
- Support cancellation by aborting underlying HTTP request on stream drop.

## 3) Ollama BackendAdapter

### Transport

- HTTP via `reqwest`.
- Endpoint MVP:
  - `POST /api/chat` with `stream=true|false`.

### Request mapping

Canonical -> Ollama:

- `model` -> `model`
- `messages` -> `messages`
- `tools` -> `tools` (if backend version supports)
- `stream` -> `stream`
- `limits.max_output_tokens` -> adapter option map if supported by runtime

Headers:

- `Content-Type: application/json`
- `X-Request-Id: <request_id>`

### Stream parsing

- Parse NDJSON chunks.
- Map `message.content` to `OutputTextDelta`.
- Map tool-call payloads to canonical tool-call events.
- Usage extraction from terminal fields (`prompt_eval_count`, `eval_count`, etc.) when present.
- Support cancellation by aborting underlying HTTP request on stream drop.

## 4) GitHub Copilot BackendAdapter (SDK/LSP)

Source anchors used for this design:

- GitHub Copilot Language Server SDK public preview announcement (2025-02-10).
- `copilot-language-server-release` npm package docs and API sample.

### Transport

- Spawn Copilot language server process over stdio.
- JSON-RPC/LSP framing using `Content-Length` headers.
- Dedicated `copilot_rpc` transport module for process lifecycle.

### Session lifecycle (MVP)

1. Start process from configured command/path.
2. Execute `initialize` handshake and `initialized` notification.
3. Perform auth status check via SDK-supported methods (`checkStatus` + status notifications).
4. Reject request with `Authentication` error if session is not ready.

### Inference mapping (MVP)

- Use Copilot SDK/LSP completion-oriented requests as backend primitive.
- Primary request path: `textDocument/copilotPanelCompletion` (fallback: `textDocument/inlineCompletion`), with adapter-owned synthetic document context.
- Adapter converts canonical prompt/messages into completion request context.
- Return generated text as canonical output stream (single-chunk stream when backend response is non-incremental).
- Handle stream drop by canceling/closing the active RPC request and cleaning session-local inflight state.

MVP capability declaration (conservative):

- `streaming=true`
- `tool_calls=false`
- `json_mode=false`
- `vision=false`
- `resumable_streaming=false`

### Auth and secrets

- Copilot-specific auth/session data stays inside adapter-local state.
- Any token/env inputs still flow through CredentialProvider.

## 5) Error Mapping Seeds

### OpenAI-compatible

- 400/422 -> `InvalidRequest` (`retryable=false`)
- 401/403 -> `Authentication` or `Authorization`
- 408/429/5xx -> `RateLimited` or `BackendTransient` (`retryable=true`)

### Ollama

- Connection refused / timeout -> `BackendTransient`
- 400-series invalid payload -> `InvalidRequest`
- 500-series -> `BackendTransient`

### Copilot SDK/LSP

- process spawn/init failure -> `BackendTransient` or `ProtocolViolation`
- explicit auth status failure -> `Authentication`
- SDK rate-limit signal -> `RateLimited`

## 6) Capability Probe Strategy

MVP probing approach:

1. Use static capability defaults per adapter.
2. Allow per-backend config overrides.
3. Optional active probes:
   - OpenAI-compatible: lightweight model metadata probe (if configured)
   - Ollama: tags/model metadata endpoint
   - Copilot: initialize/auth/session capability check

Status: `READY_FOR_L2_REVIEW`
