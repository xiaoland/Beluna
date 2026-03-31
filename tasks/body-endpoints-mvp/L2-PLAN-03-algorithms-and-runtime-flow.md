# L2-03 - Algorithms And Runtime Flow
- Task Name: `body-endpoints-mvp`
- Stage: `L2`
- Date: `2026-02-12`
- Status: `DRAFT_FOR_APPROVAL`

## 1) Runtime Lifecycle Algorithms (`runtime/src`)

### 1.1 `start` algorithm
1. Resolve runtime config (core command/path, std-body command/path, socket path).
2. Spawn core process.
3. Poll until UnixSocket is ready (bounded wait).
4. Spawn std-body endpoint host process.
5. Persist runtime state (PIDs, socket path, start timestamp).
6. Return success once both processes are healthy.

### 1.2 `stop` algorithm
1. Load runtime state.
2. Send `{"type":"exit"}` to core socket.
3. Wait bounded grace period for core exit.
4. Terminate std-body host process if still running.
5. Clear runtime state.

### 1.3 `status` algorithm
1. Load runtime state.
2. Check process liveness for tracked PIDs.
3. Check UnixSocket availability.
4. Print structured status.

## 2) Std-Body Host Registration Algorithm
`std-body` acts as endpoint client to Spine socket.

1. Connect to UnixSocket.
2. Send `endpoint_register` for shell route:
- `tool.shell.exec/cap.std.shell`
3. Send `endpoint_register` for web route:
- `tool.web.fetch/cap.std.web.fetch`
4. Enter read loop for incoming `endpoint_invoke`.
5. Dispatch by `affordance_key` + `capability_handle`.
6. Return `endpoint_result`.
7. On socket reconnect, re-register routes idempotently.

## 3) Shell Endpoint Handler Algorithm
Input: `endpoint_invoke.action.normalized_payload`.

1. Parse into `ShellExecRequest`; fail -> rejected outcome.
2. Validate `argv` minimum requirements.
3. Clamp timeout/stdout/stderr limits.
4. Run command by argv directly (no shell interpolation).
5. Timeout -> kill best effort -> rejected (`timeout`).
6. I/O/runtime failure -> rejected (`exec_failure`).
7. Non-zero exit -> rejected (`non_zero_exit`).
8. Success -> applied with reference id and optional observation payload.

## 4) Web Endpoint Handler Algorithm
Input: `endpoint_invoke.action.normalized_payload`.

1. Parse into `WebFetchRequest`; fail -> rejected (`invalid_payload`).
2. Validate scheme (`http|https`); else rejected (`unsupported_scheme`).
3. Clamp timeout/body limits.
4. Execute HTTP request.
5. Timeout -> rejected (`timeout`).
6. Transport error -> rejected (`network_error`).
7. Read capped response bytes and produce text body.
8. Return applied with response observation payload.

HTTP non-2xx is treated as applied with status included.

Reference:
1. https://developer.mozilla.org/en-US/docs/Web/API/Fetch_API

## 5) Apple Endpoint Lifecycle (Self-Registration)
Apple app lifecycle is independent from runtime:

1. Apple connects to UnixSocket when app starts.
2. Apple sends `endpoint_register` for `chat.reply.emit`.
3. Apple handles `endpoint_invoke`.
4. Apple sends `endpoint_result`.
5. On app disconnect, endpoint becomes unavailable until reconnect.

Runtime does not inject or proxy Apple registration.

## 6) Execution Compatibility Gate
Given no core changes allowed, L3 execution depends on one condition:
1. core’s active UnixSocket Spine adapter must already support endpoint registration/invocation/result message flow.

If condition fails in current code, L3 implementation must pause and request a scoped exception or alternative integration path from user.

Status: `READY_FOR_REVIEW`
