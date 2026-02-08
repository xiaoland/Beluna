# L3-04 - Backend Adapter Implementation

- Task Name: `minimal-ai-gateway`
- Stage: `L3` detail: adapter implementation
- Date: 2026-02-08
- Status: `DRAFT_FOR_APPROVAL`

## 1) OpenAI-Compatible Adapter

### Implementation boundaries

1. Protocol target is `chat/completions`-like behavior, not strict parity.
2. Missing/divergent fields degrade gracefully when safe.
3. Unrecoverable protocol divergence maps to canonical `ProtocolViolation` or `InvalidRequest`.

### Step plan

1. Build request payload from canonical request:
- map messages,
- map tools/tool_choice,
- map JSON mode if requested.

2. Send HTTP request with:
- `Authorization` bearer token,
- `Content-Type: application/json`,
- request-id header.

3. If `stream=true`:
- parse SSE lines and events,
- handle `[DONE]`,
- convert deltas/tool deltas to backend raw events.

4. If non-stream:
- treat final body as one-shot raw event sequence.

5. Extract usage when available and emit terminal usage event.

6. Provide cancellation:
- abort HTTP in-flight request via cancellation token/drop-aware stream.

## 2) Ollama Adapter

### Step plan

1. Build `/api/chat` request payload.
2. Send request with JSON body and request-id header.
3. Stream mode:
- parse NDJSON by line,
- map `message.content` and tool-call structures,
- detect terminal chunk and usage fields.
4. Non-stream mode:
- convert body to synthetic incremental sequence.
5. Cancellation:
- abort in-flight request on stream drop.

### Compatibility handling

- if backend omits tool fields, continue text-only normalization.
- map unsupported requested features to deterministic `UnsupportedCapability` before request dispatch.

## 3) Copilot Adapter (SDK/LSP)

### Process and transport

1. start configured copilot language server process,
2. run JSON-RPC framing loop,
3. perform initialize handshake,
4. run auth readiness check (`checkStatus` + notifications).

### Inference path

1. build synthetic document/context from canonical request,
2. call `textDocument/copilotPanelCompletion`,
3. if unavailable, fallback to `textDocument/inlineCompletion`,
4. map completion result to canonical output deltas.

### Session/state model

- keep adapter-local process/session state keyed by backend profile,
- reconnect process on fatal transport failures,
- mark backend transient/auth errors deterministically.

### Cancellation

- cancel active RPC request when consumer drops stream,
- clear local in-flight entry,
- do not mark cancellation as backend failure.

## 4) Adapter Capability Defaults

1. OpenAI-compatible:
- `streaming=true`, `tool_calls=true`, `json_mode=true`, `vision=false` by default.

2. Ollama:
- `streaming=true`, `tool_calls=false` default (override allowed), `json_mode=false`, `vision=false`.

3. Copilot:
- `streaming=true`, `tool_calls=false`, `json_mode=false`, `vision=false`, `resumable_streaming=false`.

## 5) Adapter Error Mapping (Implementation Rules)

1. Never leak raw tokens/headers in errors.
2. Always attach backend id and retryability hint.
3. Normalize status/code families before retry logic:
- 4xx request/auth class -> non-retryable except 408/429,
- 5xx/transient transport -> retryable.

Status: `READY_FOR_L3_REVIEW`
