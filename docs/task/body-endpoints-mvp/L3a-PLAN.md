# L3a Plan - Comprehensive Implementation Checklist
- Task Name: `body-endpoints-mvp`
- Stage: `L3a` (strict execution checklist)
- Date: `2026-02-12`
- Status: `DRAFT_FOR_APPROVAL`

Follow this checklist in order without skipping gates.

## 1) Pre-Change Gate
1. confirm L3 package approval.
2. snapshot workspace status.
3. keep `core/src/*` and `core/tests/*` untouched.

## 2) WS0 Compatibility Gate (Blocking)
1. inspect current core wire/parser runtime paths for endpoint lifecycle messages.
2. run socket probe if static inspection is inconclusive.

Gate:
1. if endpoint registration/invoke/result is unsupported, stop and ask user for decision before implementation continues.

## 3) Runtime Lifecycle Scaffold (`runtime/src`)
1. create runtime crate and command modules (`start`, `stop`, `status`).
2. implement process supervisor + state persistence.
3. implement stop sequence via socket `exit`.
4. add lifecycle tests.

Gate:
1. runtime tests pass.

## 4) Std-Body Endpoint Handlers
1. create std-body crate scaffold.
2. implement shell handler with hard caps.
3. implement web handler with hard caps.
4. add handler unit tests.
5. ensure no duplicated core-generic endpoint type definitions.

Gate:
1. std-body handler tests pass.

## 5) Std-Body Endpoint Host Loop
1. implement UnixSocket connection/reconnect.
2. implement shell/web route registration.
3. implement invoke dispatch and result envelopes.
4. add host protocol tests.

Gate:
1. host loop tests pass.

## 6) Apple Compatibility Fixtures
1. add protocol fixture tests for Apple self-registration envelopes.
2. validate Responses-aligned payload subset fields.

Gate:
1. apple fixture tests pass (or are marked blocked if WS0 failed).

## 7) Documentation Closure
1. update overview/modules docs.
2. add body module doc.
3. write task result docs.

## 8) Final Validation
1. run all runnable tests in `runtime` and `std-body`.
2. verify no core file changes.
3. prepare concise summary with gate outcomes and file references.

Status: `READY_FOR_EXECUTION_AFTER_APPROVAL`
