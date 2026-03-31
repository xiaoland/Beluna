# L3-01 - Workstreams And Sequence
- Task Name: `spine-implementation`
- Stage: `L3` detail: execution sequence
- Date: `2026-02-12`
- Status: `IMPLEMENTED`

## 1) Execution Principles
1. Spine core remains transport-ignorant; adapter absorbs transport details.
2. Routing is mechanical table lookup by (`affordance_key`, `capability_handle`).
3. Body Endpoint -> Spine ingress datum is canonically `Sense`.
4. Async boundaries are end-to-end across Spine and Continuity.
5. No compatibility-preservation requirement blocked refactor decisions.

## 2) Ordered Workstreams

### Workstream A - Spine Core Routing Kernel
Scope:
1. Introduce async endpoint and executor ports.
2. Add in-memory endpoint registry with route conflict and affordance descriptor consistency checks.
3. Add routing executor for serialized and best-effort modes.
4. Add Spine capability catalog snapshot support.

Exit criteria:
1. async contracts compile.
2. route miss/error mapping deterministic.
3. catalog snapshots deterministic and ordered.

### Workstream B - Continuity Async Migration
Scope:
1. Convert continuity spine port to async.
2. Convert effectuation/process calls to async.
3. Keep settlement and debit invariants intact.

Exit criteria:
1. continuity/admission flow compiles.
2. ledger invariants remain green.

### Workstream C - UnixSocket Adapter Extraction
Scope:
1. Move socket bind/accept/client parsing lifecycle into `spine/adapters/unix_socket.rs`.
2. Create `spine/adapters/wire.rs` for canonical message parsing (`sense` first-class).
3. Add Spine catalog bridge for Cortex (`spine/adapters/catalog_bridge.rs`).

Exit criteria:
1. runtime uses adapter module instead of embedding transport logic in `server.rs`.
2. wire parsing tests pass.

### Workstream D - Runtime Orchestration Refactor
Scope:
1. Rewire `server.rs` as orchestrator for adapter + reactor + continuity.
2. Feed Cortex capability catalog from Spine capability snapshot each cycle.
3. Add shutdown cancellation flow for adapter task.

Exit criteria:
1. runtime loop operates with adapter and async continuity.
2. signal and socket-exit shutdown behavior preserved.

### Workstream E - Test And Docs Closure
Scope:
1. Migrate impacted tests to async continuity/spine APIs.
2. Add focused Spine registry/router unit tests.
3. Update feature/module/contract/task docs and publish task result.

Exit criteria:
1. `cargo test` is green.
2. docs align with shipped UnixSocket-only MVP scope.

## 3) Dependency Graph
1. A -> B
2. A + B -> C
3. C -> D
4. D -> E

## 4) Out Of Scope
1. WebSocket adapter.
2. HTTP adapter.
3. distributed endpoint self-registration protocol.
4. transport auth and multi-tenant policy framework.

Status: `DONE`
