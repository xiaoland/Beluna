# Adapter Matrix

## OpenAI-Compatible Adapter

- Transport: HTTP
- Protocol target: `chat/completions`-like (not strict API parity)
- Streaming parse: SSE
- Notes: missing/divergent fields degrade gracefully when possible

## Ollama Adapter

- Transport: HTTP
- Endpoint target: `/api/chat`
- Streaming parse: NDJSON

## GitHub Copilot Adapter

- Transport: stdio JSON-RPC (language server process)
- Lifecycle: initialize + auth status check
- Completion path: `textDocument/copilotPanelCompletion` with inline fallback

## Shared Adapter Rules

- Adapter owns both transport and dialect mapping.
- Adapter exposes cancellation handle for stream-drop propagation.
- Adapter maps backend errors to canonical error kinds.
