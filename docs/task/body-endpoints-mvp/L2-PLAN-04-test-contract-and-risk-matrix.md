# L2-04 - Test Contract And Risk Matrix
- Task Name: `body-endpoints-mvp`
- Stage: `L2`
- Date: `2026-02-12`
- Status: `DRAFT_FOR_APPROVAL`

## 1) Contract-To-Test Matrix

### 1.1 `std-body` endpoint tests
1. shell success/timeout/non-zero/invalid-payload outcomes.
2. web success/timeout/network-error/unsupported-scheme outcomes.
3. deterministic truncation behaviors for output/response caps.

### 1.2 `std-body` host protocol tests
1. startup sends two `endpoint_register` envelopes (shell/web).
2. incoming `endpoint_invoke` is routed to correct handler.
3. handler outcome maps to correct `endpoint_result` envelope.
4. reconnect path re-registers routes.

### 1.3 `runtime` lifecycle tests
1. `start` launches core and std-body host.
2. `stop` sends core exit and cleans child processes.
3. `status` reflects liveness and socket readiness.

### 1.4 Apple self-registration compatibility tests
1. apple-style `endpoint_register` envelope accepted by socket server.
2. apple-style `endpoint_result` correlates with invoke request id.
3. Apple disconnect behavior is deterministic (endpoint unavailable until reconnect).

## 2) Boundary Enforcement Checks
1. no file edits under `/Users/lanzhijiang/Development/Beluna/core/src/*`.
2. no file edits under `/Users/lanzhijiang/Development/Beluna/core/tests/*`.
3. no duplicated core-generic endpoint model types in `std-body`.
4. runtime logic exists under `/Users/lanzhijiang/Development/Beluna/runtime/src/*`.

## 3) Risks And Mitigations
1. Risk: current core socket protocol may not support endpoint registration flow.
- Mitigation: explicit compatibility gate before implementation; stop and discuss exception if missing.

2. Risk: runtime and std-body lifecycle drift.
- Mitigation: PID/state-based supervisor with bounded waits and explicit teardown.

3. Risk: shell/web endpoint abuse.
- Mitigation: strict timeout/output caps with deterministic rejection reason codes.

4. Risk: Apple/client endpoint disconnect churn.
- Mitigation: idempotent re-registration and deterministic invoke timeout handling.

## 4) Documentation Delta Plan
1. `/Users/lanzhijiang/Development/Beluna/docs/overview.md`
- add component split: `core` process + `runtime` lifecycle + external endpoint clients.

2. `/Users/lanzhijiang/Development/Beluna/docs/modules/spine/README.md`
- clarify Spine socket adapter endpoint-client mode.

3. new module doc:
- `/Users/lanzhijiang/Development/Beluna/docs/modules/body/README.md`
- describe `std-body` endpoint host role and Apple peer model.

4. `/Users/lanzhijiang/Development/Beluna/docs/task/RESULT.md`
- update latest task index once implementation completes.

Status: `READY_FOR_REVIEW`
