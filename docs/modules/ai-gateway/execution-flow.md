# Execution Flow

## `chat_stream`

1. Normalize `ChatRequest` to `CanonicalRequest`.
2. Resolve route deterministically from:
   - alias (`default`, `low-cost`, ...)
   - or direct `<backend-id>/<model-id>`
3. Resolve credential for the selected backend.
4. Validate requested capabilities.
5. Apply budget pre-dispatch checks and acquire resources.
6. Invoke adapter under reliability control.
7. Normalize backend raw events into canonical gateway events.
8. Map canonical events to chat capability events.
9. On stream drop, cancel in-flight backend work and release resources.

## `chat_once`

1. Calls internal chat dispatch pipeline in non-stream mode.
2. Aggregates stream events to a final response.
3. Returns on terminal success.
4. Returns error on terminal failure.

## Determinism Guarantees

- No multi-backend fallback.
- Unknown alias/backend/model fails fast.
- Exactly one selected `(backend_id, model_id)` per request.
