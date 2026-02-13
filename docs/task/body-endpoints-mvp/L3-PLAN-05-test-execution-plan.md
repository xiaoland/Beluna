# L3-05 - Test Execution Plan
- Task Name: `body-endpoints-mvp`
- Stage: `L3`
- Date: `2026-02-12`
- Status: `DRAFT_FOR_APPROVAL`

## 1) Preflight Validation
1. run WS0 compatibility check (required before implementation continuation).

## 2) Unit Test Groups

### 2.1 std-body handlers
1. shell handler cases:
- success
- timeout
- invalid payload
- non-zero exit
2. web handler cases:
- success
- timeout
- invalid payload
- unsupported scheme
- network error

Command:
```bash
cd /Users/lanzhijiang/Development/Beluna/std-body && cargo test
```

### 2.2 runtime lifecycle
1. start/stop/status flows.
2. state file and PID handling behavior.

Command:
```bash
cd /Users/lanzhijiang/Development/Beluna/runtime && cargo test
```

## 3) Integration Smoke Tests
1. start runtime.
2. verify core socket ready.
3. verify std-body registrations observed (or probe invoke flow if protocol supports).
4. stop runtime and verify cleanup.

Example command sequence:
```bash
cd /Users/lanzhijiang/Development/Beluna/runtime && cargo run -- start
cd /Users/lanzhijiang/Development/Beluna/runtime && cargo run -- status
cd /Users/lanzhijiang/Development/Beluna/runtime && cargo run -- stop
```

## 4) Compatibility-Gated Tests
Only if WS0 passes:
1. host protocol tests with `endpoint_register/invoke/result`.
2. Apple self-registration fixture tests.

If WS0 fails:
1. mark endpoint-protocol integration tests as blocked.
2. report block in `RESULT.md`.

## 5) Success Criteria
1. all runnable tests pass.
2. no core source changes.
3. runtime lifecycle works deterministically.

Status: `READY_FOR_REVIEW`
