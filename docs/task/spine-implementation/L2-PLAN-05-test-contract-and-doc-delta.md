# L2-05 - Test Contract And Documentation Delta
- Task Name: `spine-implementation`
- Stage: `L2` detailed file
- Date: `2026-02-12`
- Status: `DRAFT_FOR_APPROVAL`

## 1) Contract-to-Test Matrix

### 1.1 Spine core routing contracts
1. Scenario: route exists and endpoint applies
- Expect: `ActionApplied`, ordered `seq_no`, linkage fields present.

2. Scenario: route missing
- Expect: `ActionRejected` with deterministic `reason_code="route_not_found"`.

3. Scenario: endpoint returns rejected
- Expect: `ActionRejected` with endpoint-provided reason and reference.

4. Scenario: endpoint invocation error
- Expect: `ActionRejected` with deterministic fallback reason.

5. Scenario: batch contains invalid action ids
- Expect: `InvalidBatch` error (whole call failure).

### 1.2 Catalog ownership and bridge contracts
6. Scenario: endpoints register two capabilities under same affordance
- Expect: catalog groups them into one affordance with two allowed handles for Cortex bridge.

7. Scenario: same affordance registered with schema mismatch
- Expect: registration rejected with invariant error.

8. Scenario: registration/unregistration updates version
- Expect: version monotonic increase and deterministic snapshot ordering.

### 1.3 Universal endpoint abstraction contracts
9. Scenario: native endpoint and adapter-backed endpoint both implement `EndpointPort`
- Expect: Spine dispatch path identical from core perspective (no branch by transport type).

10. Scenario: best-effort mode concurrent completion order differs from input
- Expect: final report events still ordered by `seq_no`.

### 1.4 Adapter shell contracts
11. UnixSocket adapter parses valid NDJSON line
- Expect: message forwarded to runtime channel.

12. UnixSocket adapter receives invalid JSON line
- Expect: line ignored; loop continues.

13. WebSocket adapter receives valid `sense` frame
- Expect: message forwarded into runtime ingress channel.

14. WebSocket adapter receives invalid frame
- Expect: deterministic protocol error message and frame/session handling per policy.

15. WebSocket adapter with closed/full channel
- Expect: deterministic backpressure error message and optional overload close.

## 2) Existing Tests Impact

1. `core/tests/spine/contracts.rs`
- migrate to async calls (`.await`) under `#[tokio::test]`.

2. `core/tests/admission/admission.rs`
- update calls to async `ContinuityEngine::process_attempts`.

3. `core/tests/continuity/debits.rs`
- same async migration.

4. `core/tests/cortex_continuity_flow.rs`
- update spine/continuity call signatures and catalog source assumptions.

## 3) Planned New Test Files

1. `core/tests/spine/routing.rs`
- route lookup and event mapping tests.

2. `core/tests/spine/catalog.rs`
- registration/catalog bridge invariants.

3. `core/tests/spine/adapters_websocket.rs`
- WebSocket ingress/egress behavior.

4. `core/tests/spine/adapters_unix_socket.rs`
- Unix socket line parsing/forwarding behavior.

## 4) Test Commands (L3 target)

1. `cargo test --test spine_bdt`
2. `cargo test --test continuity_bdt`
3. `cargo test --test admission_bdt`
4. `cargo test --test cortex_bdt`
5. `cargo test`

## 5) Documentation Delta Plan

### 5.1 Features
1. update `docs/features/spine/PRD.md`
- move from contract-only MVP to async routing + adapter shells.

2. update `docs/features/spine/HLD.md`
- include route table, registry, catalog ownership, adapter boundaries.

3. update `docs/features/spine/LLD.md`
- include route/registration invariants and dispatch algorithms.

### 5.2 Contracts
4. update `docs/contracts/spine/README.md`
- add route miss and registration invariants.

5. update `docs/contracts/cortex/README.md`
- state capability catalog source is Spine-owned snapshot.

### 5.3 Modules and overview
6. update `docs/modules/spine/README.md`
- include core/adapters split and WebSocket/UnixSocket shells.

7. update `docs/overview.md`
- clarify Spine as sole channel and catalog authority from endpoint registration.

8. update `docs/glossary.md`
- sharpen `affordance_key` and `capability_handle` definitions.

## 6) L2-05 Exit Criteria
This file is complete when:
1. every new invariant has planned tests,
2. async migration impact is explicit,
3. documentation updates are scoped and traceable.
