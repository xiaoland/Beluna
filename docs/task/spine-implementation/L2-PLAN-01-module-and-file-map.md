# L2-01 - Module And File Map
- Task Name: `spine-implementation`
- Stage: `L2` detailed file
- Date: `2026-02-12`
- Status: `DRAFT_FOR_APPROVAL`

## 1) Implementation Boundary
This task upgrades Spine from contract-only noop executor to async route-based execution kernel with adapter shells.

Canonical boundary after this task:
1. Input: `AdmittedActionBatch` only.
2. Routing key: (`affordance_key`, `capability_handle`).
3. Output: ordered `SpineExecutionReport`.
4. Catalog authority: Spine registration state.
5. Transport logic: only in `spine/adapters/*` and concrete endpoint adapter impls.

## 2) Source File Map (Planned)

### Spine core
1. `core/src/spine/mod.rs`
- export new core modules and adapter namespace.

2. `core/src/spine/types.rs`
- add route/capability/endpoint invocation and outcome structures.
- keep existing settlement event contracts.

3. `core/src/spine/ports.rs`
- convert `SpineExecutorPort` to async.
- add endpoint abstraction ports and registry/catalog port surface.

4. `core/src/spine/registry.rs` (new)
- in-memory route table + registration invariants + catalog snapshot.

5. `core/src/spine/router.rs` (new)
- async mechanical executor implementation using route table lookup.

6. `core/src/spine/noop.rs`
- keep deterministic noop implementation but convert to async trait implementation.

7. `core/src/spine/error.rs`
- extend deterministic error codes for registration and routing failures.

### Spine adapters
8. `core/src/spine/adapters/mod.rs` (new)
- adapter module exports.

9. `core/src/spine/adapters/unix_socket.rs` (new)
- migrate Unix socket transport loop from `core/src/server.rs` into adapter shell.

10. `core/src/spine/adapters/websocket.rs` (new)
- WebSocket shell for bi-directional Spine <-> Body Endpoint interface.

11. `core/src/spine/adapters/wire.rs` (new)
- shared wire message parsing structures reused by UnixSocket and WebSocket shells.

12. `core/src/spine/adapters/catalog_bridge.rs` (new)
- map Spine-owned catalog snapshot to Cortex `CapabilityCatalog` input contract.

### Runtime/continuity integration
13. `core/src/server.rs`
- reduce to runtime orchestrator/bootstrap and adapter wiring.
- remove direct Unix socket accept/parsing logic.

14. `core/src/continuity/ports.rs`
- make `SpinePort::execute_admitted` async.

15. `core/src/continuity/noop.rs`
- async adapter from `SpineExecutorPort` to `SpinePort`.

16. `core/src/continuity/engine.rs`
- make `effectuate_attempts` / `process_attempts` async and await spine execution.

### Config/schema
17. `core/src/config.rs`
- add optional WebSocket adapter runtime config.

18. `core/beluna.schema.json`
- add schema for WebSocket adapter config block.

### Protocol cutover
19. `core/src/protocol.rs`
- removed from canonical runtime path.
- canonical wire parser/type definitions move to `spine/adapters/wire.rs`.

## 3) Tests File Map (Planned)

1. `core/tests/spine/contracts.rs`
- update for async trait calls.

2. `core/tests/spine/routing.rs` (new)
- routing by composite key and missing-route deterministic rejection.

3. `core/tests/spine/catalog.rs` (new)
- registration and catalog derivation invariants.

4. `core/tests/spine/adapters_websocket.rs` (new)
- WebSocket adapter ingress/egress and wire parsing path.

5. `core/tests/continuity/debits.rs`
- migrate sync calls to async `process_attempts`.

6. `core/tests/admission/admission.rs`
- migrate sync calls to async continuity path.

7. `core/tests/cortex_continuity_flow.rs`
- route Spine catalog into Cortex input via catalog bridge.

## 4) Dependency Direction Rules
1. `spine` core modules may depend on:
- local spine modules,
- async/runtime primitives,
- serde/json utilities.

2. `spine` core modules must not depend on:
- Unix socket listener APIs,
- WebSocket server/client APIs,
- transport wire formats.

3. `spine/adapters/*` may depend on:
- transport/runtime libraries,
- wire formats,
- Spine core ports/types.

4. `cortex` receives catalog as data only.
- no dependency on adapter internals.

5. `continuity` depends only on async Spine execution boundary.
- no transport/protocol dependencies.

## 5) Cutover Notes
1. Keep `core/src/server.rs::run(config)` as binary entrypoint, but internal flow can change fully.
2. Old wire and old config shapes are not preserved unless directly reused by the new adapter design.
3. Keep `DeterministicNoopSpine` available for deterministic tests and fallback wiring.

## 6) L2-01 Exit Criteria
This file is complete when:
1. file-level implementation scope is explicit,
2. ownership boundaries are clear,
3. migration surfaces (`spine`, `continuity`, `server`) are concrete.
