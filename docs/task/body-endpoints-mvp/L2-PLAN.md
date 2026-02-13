# L2 Plan - Body Endpoints MVP (Low-Level Design)
- Task Name: `body-endpoints-mvp`
- Stage: `L2` (low-level design)
- Date: `2026-02-12`
- Status: `DRAFT_FOR_APPROVAL`

This L2 is split into focused files so interfaces, data contracts, algorithms, and tests can be reviewed independently.

## L2 File Index
1. `/Users/lanzhijiang/Development/Beluna/docs/task/body-endpoints-mvp/L2-PLAN-01-module-and-file-map.md`
- exact source/test/doc change map
- crate boundaries and dependency direction (`core` unchanged)

2. `/Users/lanzhijiang/Development/Beluna/docs/task/body-endpoints-mvp/L2-PLAN-02-interfaces-and-data-contracts.md`
- `std-body` public interfaces
- runtime lifecycle and endpoint-client interfaces
- UnixSocket Apple endpoint wire contracts (Responses-aligned subset)

3. `/Users/lanzhijiang/Development/Beluna/docs/task/body-endpoints-mvp/L2-PLAN-03-algorithms-and-runtime-flow.md`
- lifecycle/registration/bootstrap algorithms
- shell and web endpoint execution algorithms
- Apple self-registration endpoint invoke/result lifecycle

4. `/Users/lanzhijiang/Development/Beluna/docs/task/body-endpoints-mvp/L2-PLAN-04-test-contract-and-risk-matrix.md`
- contract-to-test matrix
- safety and determinism checks
- doc delta plan

## L2 Objective
Define exact interfaces, structures, and deterministic algorithms for:
1. `std-body` hosted endpoints (`tool.shell.exec`, `tool.web.fetch`).
2. Apple Universal App as external Body Endpoint over UnixSocket for `chat.reply.emit`.
3. `runtime/src` lifecycle glue that starts/stops core and std-body while keeping `core` implementation untouched.

## L2 Completion Gate
L2 is complete when:
1. crate/module boundaries are unambiguous and cycle-free,
2. `core` implementation changes are avoided,
3. payload schemas and route contracts are explicit,
4. endpoint timeout/error mapping rules are deterministic,
5. UnixSocket request/response lifecycle for Apple endpoint is explicit,
6. L3 can execute without architecture reinterpretation.

Status: `READY_FOR_REVIEW`
