# Execution Flow

## `infer_stream`

1. Normalize `BelunaInferenceRequest` to `CanonicalRequest`.
2. Route deterministically to one backend profile.
3. Resolve credential for the selected backend.
4. Validate requested capabilities.
5. Apply budget pre-dispatch checks and acquire resources.
6. Invoke adapter under reliability control.
7. Normalize backend raw events into canonical gateway events.
8. Emit canonical stream to caller.
9. On stream drop, cancel in-flight backend work and release resources.

## `infer_once`

1. Calls `infer_stream`.
2. Aggregates stream events to a final response.
3. Returns on terminal success.
4. Returns error on terminal failure.

## Determinism Guarantees

- No multi-backend fallback.
- Exactly one selected backend per request.
