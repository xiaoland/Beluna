# L3-01 - Workstreams And Sequence
- Task Name: `body-endpoints-mvp`
- Stage: `L3`
- Date: `2026-02-12`
- Status: `DRAFT_FOR_APPROVAL`

## Workstream 0 - Compatibility Gate (Blocking)
Goal:
1. verify current core UnixSocket protocol supports endpoint-client lifecycle envelopes:
- `endpoint_register`
- `endpoint_invoke`
- `endpoint_result`

Actions:
1. inspect `core/src/spine/adapters/wire.rs` and runtime message handling in `core/src/server.rs`.
2. run minimal socket probe against running core (if needed) with synthetic envelopes.

Gate:
1. if supported -> continue to WS1.
2. if unsupported -> stop implementation and ask user for scoped exception/alternative before code changes.

## Workstream 1 - `runtime/src` Lifecycle Component
Goal:
1. add runtime lifecycle binary with `start`, `stop`, `status`.

Actions:
1. create `runtime/` crate scaffold.
2. implement process supervisor:
- spawn core process,
- wait for socket readiness,
- spawn std-body host process,
- persist runtime state.
3. implement stop flow:
- send core `exit` message,
- bounded wait,
- terminate std-body child if needed.

Gate:
1. lifecycle tests pass (`start`/`stop`/`status` behavior).

## Workstream 2 - `std-body` Endpoint Handlers
Goal:
1. implement shell and web endpoint execution logic with caps.

Actions:
1. create `std-body/` crate scaffold.
2. implement shell handler:
- argv-based execution, timeout, capped outputs.
3. implement web handler:
- HTTP(S) only, timeout, capped response body, deterministic status handling.
4. keep only endpoint-specific payload DTOs; avoid duplicated core-generic endpoint model types.

Gate:
1. unit tests for shell/web handler outcomes pass.

## Workstream 3 - `std-body` Endpoint Host Loop
Goal:
1. implement UnixSocket endpoint-client loop for self-registration and invoke/result handling.

Actions:
1. connect/reconnect loop to core socket.
2. send shell/web `endpoint_register` envelopes.
3. dispatch `endpoint_invoke` to handler by route.
4. send `endpoint_result` envelopes.

Gate:
1. host protocol tests pass, including reconnect/re-register behavior.

## Workstream 4 - Apple Compatibility Surface
Goal:
1. lock payload schema compatibility for Apple self-registration flow.

Actions:
1. provide runtime/std-body protocol fixtures for:
- `chat.reply.emit` registration envelope,
- invoke/result roundtrip,
- Responses-aligned payload subset fields.
2. add tests validating schema acceptance and request-id correlation.

Gate:
1. apple compatibility tests pass.

## Workstream 5 - Docs And Result Closure
Goal:
1. update docs and produce final task result.

Actions:
1. update `docs/overview.md`.
2. update `docs/modules/spine/README.md`.
3. add `docs/modules/body/README.md`.
4. write `/Users/lanzhijiang/Development/Beluna/docs/task/body-endpoints-mvp/RESULT.md`.
5. update `/Users/lanzhijiang/Development/Beluna/docs/task/RESULT.md` index.

Gate:
1. docs reflect implemented behavior and boundary constraints.

## Execution Order
1. WS0
2. WS1
3. WS2
4. WS3
5. WS4
6. WS5

No parallel execution before WS0 passes.

Status: `READY_FOR_REVIEW`
