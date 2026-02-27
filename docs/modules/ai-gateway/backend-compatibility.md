# AI Gateway — Backend Compatibility Map

## Static Capabilities by Adapter

| Capability | `OpenAiCompatible` | `Ollama` | `GitHubCopilotSdk` |
|---|:---:|:---:|:---:|
| **Streaming** | Yes | Yes | Yes |
| **Tool Calls** | Yes | No | No |
| **JSON Mode** (`json_object`) | Yes | No | No |
| **JSON Schema Mode** (`json_schema`) | Yes | No | No |
| **Vision** (image_url parts) | No | No | No |
| **Resumable Streaming** | No | No | No |
| **Tool Retry Safe** | No | No | No |

> These are the *static* defaults from `static_capabilities()`. Per-backend config can override via `BackendProfile.capabilities`.

## Adapter Protocol Details

| Property | `OpenAiCompatible` | `Ollama` | `GitHubCopilotSdk` |
|---|---|---|---|
| **Transport** | HTTP | HTTP | Stdio (child process) |
| **Endpoint** | `{endpoint}/chat/completions` | `{endpoint}/api/chat` | N/A (LSP RPC) |
| **Stream format** | SSE (`data:` lines, `[DONE]`) | NDJSON (newline-delimited JSON) | N/A (single RPC response) |
| **Auth** | `Authorization: Bearer {token}` | `Authorization: Bearer {token}` | N/A (Copilot LS handles auth) |
| **`invoke_once` impl** | Native HTTP `stream: false` | Native HTTP `stream: false` | Delegates to `invoke_stream` + collects |
| **`invoke_stream` impl** | Native HTTP `stream: true` + SSE parse | Native HTTP `stream: true` + NDJSON parse | LSP panelCompletion → single delta + Completed |
| **Thinking support** | `thinking` + `enable_thinking` param | `think: true` param | Not supported |
| **Output mode** | `response_format` (json_object / json_schema) | Not supported | Not supported |
| **Max tokens** | `max_tokens` | `options.num_predict` | Not supported |
| **Message format** | OpenAI multi-part (`content` array or string) | Ollama flat text (`content` string) | Flat text (concatenated from all parts) |
| **Wire helpers** | `http_common::canonical_messages_to_openai` | `http_common::canonical_messages_to_ollama` | Custom `build_panel_completion_params` |

## Feature × Backend Matrix (CapabilityGuard enforced)

| Feature Request | Required Capability | `OpenAiCompatible` | `Ollama` | `GitHubCopilotSdk` |
|---|---|:---:|:---:|:---:|
| `stream: true` | `streaming` | Pass | Pass | Pass |
| `tools: [...]` | `tool_calls` | Pass | **Reject** | **Reject** |
| `tool_choice: required` | `tool_calls` | Pass | **Reject** | **Reject** |
| `output_mode: json_object` | `json_mode` | Pass | **Reject** | **Reject** |
| `output_mode: json_schema` | `json_schema_mode` | Pass | **Reject** | **Reject** |
| Message with `ImageUrl` part | `vision` | **Reject** | **Reject** | **Reject** |
| Retry after partial output | `resumable_streaming` | **Reject** | **Reject** | **Reject** |

## Error Mapping (HTTP adapters)

Both `OpenAiCompatible` and `Ollama` use `http_common::map_http_error`:

| HTTP Status | `GatewayErrorKind` | Retryable |
|---|---|---|
| 401 | `Authentication` | No |
| 403 | `Authorization` | No |
| 408, 429 | `RateLimited` | Yes |
| 400–499 (other) | `InvalidRequest` | No |
| 500+ | `BackendTransient` | Yes |

`GitHubCopilotSdk` maps errors through the RPC layer:

- Spawn failure → `BackendTransient` (retryable)
- RPC error → `BackendPermanent` (not retryable)
- Auth not ready → `Authentication` (not retryable)
- Missing text → `ProtocolViolation` (not retryable)
