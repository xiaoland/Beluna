# L3-02 - File Change Map
- Task Name: `body-endpoints-mvp`
- Stage: `L3`
- Date: `2026-02-12`
- Status: `DRAFT_FOR_APPROVAL`

## 1) Additions - `runtime/`
1. `/Users/lanzhijiang/Development/Beluna/runtime/Cargo.toml`
2. `/Users/lanzhijiang/Development/Beluna/runtime/src/main.rs`
3. `/Users/lanzhijiang/Development/Beluna/runtime/src/config.rs`
4. `/Users/lanzhijiang/Development/Beluna/runtime/src/process_supervisor.rs`
5. `/Users/lanzhijiang/Development/Beluna/runtime/src/commands/mod.rs`
6. `/Users/lanzhijiang/Development/Beluna/runtime/src/commands/start.rs`
7. `/Users/lanzhijiang/Development/Beluna/runtime/src/commands/stop.rs`
8. `/Users/lanzhijiang/Development/Beluna/runtime/src/commands/status.rs`
9. `/Users/lanzhijiang/Development/Beluna/runtime/tests/lifecycle.rs`

## 2) Additions - `std-body/`
1. `/Users/lanzhijiang/Development/Beluna/std-body/Cargo.toml`
2. `/Users/lanzhijiang/Development/Beluna/std-body/src/lib.rs`
3. `/Users/lanzhijiang/Development/Beluna/std-body/src/main.rs`
4. `/Users/lanzhijiang/Development/Beluna/std-body/src/payloads.rs`
5. `/Users/lanzhijiang/Development/Beluna/std-body/src/shell.rs`
6. `/Users/lanzhijiang/Development/Beluna/std-body/src/web.rs`
7. `/Users/lanzhijiang/Development/Beluna/std-body/src/host.rs`
8. `/Users/lanzhijiang/Development/Beluna/std-body/src/wire.rs`
9. `/Users/lanzhijiang/Development/Beluna/std-body/tests/shell_endpoint.rs`
10. `/Users/lanzhijiang/Development/Beluna/std-body/tests/web_endpoint.rs`
11. `/Users/lanzhijiang/Development/Beluna/std-body/tests/host_protocol.rs`

## 3) Additions/Updates - docs
1. `/Users/lanzhijiang/Development/Beluna/docs/modules/body/README.md` (new)
2. `/Users/lanzhijiang/Development/Beluna/docs/modules/README.md` (update index)
3. `/Users/lanzhijiang/Development/Beluna/docs/overview.md` (update runtime topology narrative)
4. `/Users/lanzhijiang/Development/Beluna/docs/modules/spine/README.md` (update endpoint-client note)
5. `/Users/lanzhijiang/Development/Beluna/docs/task/body-endpoints-mvp/RESULT.md` (new)
6. `/Users/lanzhijiang/Development/Beluna/docs/task/RESULT.md` (update latest pointer)

## 4) Optional root update
1. `/Users/lanzhijiang/Development/Beluna/README.md` (component list update)

## 5) Explicitly Unchanged
1. `/Users/lanzhijiang/Development/Beluna/core/src/*`
2. `/Users/lanzhijiang/Development/Beluna/core/tests/*`
3. `/Users/lanzhijiang/Development/Beluna/apple-universal/*` code (no implementation edits in this task unless requested)

## 6) Conditional Scope
If WS0 compatibility gate fails:
1. only non-invasive scaffolding/docs may proceed.
2. endpoint-host runtime integration code pauses pending user decision.

Status: `READY_FOR_REVIEW`
