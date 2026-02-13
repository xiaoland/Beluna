# L2-01 - Module And File Map
- Task Name: `body-endpoints-mvp`
- Stage: `L2`
- Date: `2026-02-12`
- Status: `DRAFT_FOR_APPROVAL`

## 1) Hard Boundary (User-Constrained)
1. `core` implementation must not be modified in this task.
2. `std_body_bridge` inside `core` is forbidden.
3. Integration glue must live in `runtime/src`.

## 2) Dependency Direction
1. `core` stays unchanged and is treated as a standalone process.
2. `runtime` orchestrates core process lifecycle (`start`/`stop`), not in-process composition.
3. `std-body` is an external endpoint host process connecting to Spine UnixSocket.
4. `apple-universal` is also an external endpoint host process connecting to Spine UnixSocket.

## 3) New/Changed Files (Planned)

### 3.1 New crate: `std-body/`
1. `std-body/Cargo.toml`
2. `std-body/src/lib.rs`
3. `std-body/src/host.rs`
- endpoint host event loop over Spine UnixSocket (`endpoint_register`, `endpoint_invoke`, `endpoint_result`).
4. `std-body/src/shell.rs`
5. `std-body/src/web.rs`
6. `std-body/src/payloads.rs`
- endpoint-specific payload structs only (shell/web), no duplicated core route/invocation/outcome types.
7. `std-body/tests/shell_endpoint.rs`
8. `std-body/tests/web_endpoint.rs`
9. `std-body/tests/host_protocol.rs`

### 3.2 New component: `runtime/`
1. `runtime/Cargo.toml`
- runtime lifecycle binary crate.

2. `runtime/src/main.rs`
- CLI entry (`start`, `stop`, `status`).

3. `runtime/src/commands/start.rs`
- start core process and std-body host process.

4. `runtime/src/commands/stop.rs`
- stop core via socket `exit` message and terminate managed child processes.

5. `runtime/src/process_supervisor.rs`
- process lifecycle and PID tracking.

6. `runtime/src/config.rs`
- runtime process-level config (paths, commands, env).

7. `runtime/tests/lifecycle.rs`
- start/stop integration tests.

### 3.3 Optional root-level docs/index updates
1. `/Users/lanzhijiang/Development/Beluna/README.md`
- add `runtime` and `std-body` listing.
2. docs updates listed in L2-04.

## 4) Ownership Boundaries
1. `core` owns domain components and contracts.
2. `runtime` owns process lifecycle orchestration (`start`/`stop`).
3. `std-body` owns shell/web endpoint logic and endpoint host protocol loop.
4. `apple-universal` owns chat UI and endpoint host protocol loop for Apple endpoint.

## 5) Non-Goals In This L2
1. No edits under `/Users/lanzhijiang/Development/Beluna/core/src/*`.
2. No edits under `/Users/lanzhijiang/Development/Beluna/core/tests/*` unless later approved.
3. No websocket adapter addition.
4. No in-process runtime composition with core library APIs.

Status: `READY_FOR_REVIEW`
