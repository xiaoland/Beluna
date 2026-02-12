# L1 Plan - Spine Implementation (High-Level Strategy)
- Task Name: `spine-implementation`
- Stage: `L1` (high-level strategy)
- Date: `2026-02-12`
- Status: `DRAFT_FOR_APPROVAL`

## 0) Inputs Locked From L0 Approval
User-confirmed decisions:

1. `SpineExecutorPort` becomes async; Beluna runtime is stream-first and asynchronous.
2. Routing key is composite: (`affordance_key`, `capability_handle`).
3. Adapter modules live under `core/src/spine/adapters/*`.
4. Adapter direction updated: implement WebSocket adapter (bi-directional), not HTTP.
5. Capability alignment direction:
- Body Endpoints register capabilities to Spine.
- Spine maintains Capability Catalog.
- Cortex senses current capabilities via this catalog.
6. Body Endpoint -> Spine ingress data is named `Sense`.

## 1) Strategy Summary
Implement Spine as an async, transport-ignorant routing kernel with pluggable endpoint abstraction and adapter shells.

Core strategy:
1. Separate Spine into two layers:
- **Spine Core**: route table lookup + async endpoint dispatch + event ordering.
- **Spine Adapters**: UnixSocket and WebSocket shells that host protocol/transport/runtime mechanics.
2. Introduce a Spine-owned `CapabilityCatalog` lifecycle:
- endpoint registration updates catalog,
- Cortex consumes catalog snapshots/updates as sensing input.
3. Keep all network/process/serialization/retry/timeout logic outside Spine Core:
- encapsulated in Body Endpoint impls or adapter shells.
4. Preserve continuity/ledger settlement invariants:
- admitted-action-only dispatch,
- ordered settlement events,
- replay cursor semantics.

## 2) Affordance/Capability Alignment (Canonical Model)

To resolve current ambiguity, define the terms as follows:

1. `affordance_key` = **what** action shape is possible.
- semantic operation key expected by Cortex/Admission policy, e.g. `observe.state`, `execute.tool`, `notify.webhook`.

2. `capability_handle` = **which concrete executable channel** realizes that affordance.
- endpoint-instance or implementation handle, e.g. `cap.websocket.remote`, `cap.grpc.mars-rover`, `cap.native.fs`.

3. Composite route key (`affordance_key`, `capability_handle`) = unique dispatch target identity inside Spine.

4. Capability Catalog entry describes:
- route key,
- payload schema/limits,
- endpoint metadata needed by Cortex for planning/clamp (but not transport internals).

5. Registration ownership:
- Body Endpoint (or endpoint factory in adapter shell) registers/unregisters capabilities in Spine.
- Spine is authoritative source for currently executable capabilities.

## 3) Sense Alignment

1. Body Endpoint -> Spine ingress datum is `Sense`.
2. `Sense` is transport-agnostic semantic data, independent of UnixSocket/WebSocket framing.
3. Adapter shells only encode/decode `Sense`; Spine core and Cortex consume normalized internal form.

## 4) Target Architecture

```text
Body Endpoints
  -> Sense stream
Spine Adapters (UnixSocket/WebSocket shells)
  -> normalized ingress messages
Spine Core (async, ignorant)
  - route table lookup by (affordance_key, capability_handle)
  - dispatch to EndpointPort
  - normalize endpoint outcomes -> ordered SpineEvent[]
  <- EndpointPort (native/grpc/websocket/etc. implementations)
Admission/Continuity
  -> AdmittedActionBatch -> Spine Core
Cortex
  <- capability catalog snapshot/updates (from Spine)
  -> IntentAttempt[]
```

## 5) Responsibility Split

1. Spine Core owns:
- route registration/query,
- mechanical routing only,
- execution orchestration of admitted actions,
- deterministic sequencing/event report assembly,
- capability catalog materialization.

2. Body Endpoint implementations own:
- transport/protocol details,
- retries/timeouts/deadline handling,
- serialization/deserialization,
- remote/local process interaction.

3. Spine Adapters own:
- ingress protocol handling (UnixSocket NDJSON / WebSocket frames),
- bi-directional interface exposure for Spine <-> Body Endpoint,
- stream lifecycle and backpressure handling,
- wiring Cortex + Continuity + Spine Core event loops,
- endpoint registration bootstrap.

4. Continuity/Admission own:
- admission gate and economic constraints,
- ledger reservation/settlement reconciliation,
- no transport logic.

## 6) Core Technical Decisions

1. Async port boundary
- `SpineExecutorPort` and endpoint-facing dispatch interfaces become async.

2. Route table semantics
- deterministic lookup by composite key,
- missing key yields per-action `ActionRejected` (deterministic code),
- no batch-wide panic for route miss.

3. Capability catalog as Spine state
- catalog derived from registration state, versioned for snapshot/update flow,
- exposed to runtime/adapter for Cortex ingestion.

4. Endpoint abstraction uniformity
- native function endpoint and remote endpoint implement same endpoint port shape,
- Spine Core does not branch on transport kind.

5. Adapter placement
- create `core/src/spine/adapters/unix_socket.rs` and `core/src/spine/adapters/websocket.rs` (plus support modules if needed).

6. `server.rs` migration direction
- migrate existing Unix socket runtime concerns into Spine UnixSocket adapter shell,
- add WebSocket shell for bi-directional interface,
- no backward-compatibility constraints; protocol/config can be cut over directly.

7. Streaming-first runtime
- keep bounded async channels and explicit backpressure policy,
- avoid unbounded queues inside adapter shells.

## 7) Dependency Direction Requirements

1. `core/src/spine/*` core modules may depend on:
- Spine-local types/errors/ports,
- shared domain types (`AdmittedActionBatch`, etc.),
- async primitives.

2. Spine core must not depend on:
- `tokio::net::UnixListener`,
- websocket server/framework specifics,
- protocol wire formats.

3. `core/src/spine/adapters/*` may depend on:
- runtime transport libs (`tokio` sockets, websocket server stack),
- protocol parsing/serialization modules,
- Spine core ports.

4. Cortex consumes capability catalog snapshots as data input only.
- Cortex must not read adapter internals.

## 8) Migration Strategy (High-Level)

1. Extend Spine contracts and types for async endpoint abstraction + route table + catalog.
2. Introduce Spine core executor implementation using route-table mechanical routing.
3. Add endpoint registration API and catalog snapshot/update API.
4. Port current UnixSocket runtime flow from `server.rs` into Spine UnixSocket adapter.
5. Implement WebSocket adapter with bi-directional shell behavior.
6. Rewire runtime entrypoint to use Spine adapters while preserving current flow behavior.
7. Update tests and docs to reflect non-MVP Spine semantics.

## 9) Risks and Mitigations

1. Risk: transport logic leaks into Spine core.
- Mitigation: strict module boundaries and tests asserting no adapter imports in core modules.

2. Risk: async conversion causes broad breakage.
- Mitigation: staged signature migration with compile checkpoints and focused integration tests.

3. Risk: capability catalog drift between Spine and Cortex expectations.
- Mitigation: define single Spine-owned catalog schema and versioned snapshot/update contract.

4. Risk: WebSocket adapter complexity expands scope.
- Mitigation: implement minimal but runnable bi-directional shell with strict scope caps and explicit deferred items.

5. Risk: routing misses break settlement determinism.
- Mitigation: deterministic `ActionRejected` outcome codes and ordered event emission.

## 10) Deliverables Expected from L2

L2 should define:
1. exact module/file map for Spine core and adapters,
2. async interface definitions (`SpineExecutorPort`, endpoint port, registry/catalog APIs),
3. route-table and catalog data structures,
4. action dispatch and event assembly algorithms,
5. UnixSocket adapter runtime flow and WebSocket adapter runtime flow,
6. migration map from `core/src/server.rs` to adapter shell modules,
7. contract + test matrix (unit/integration) for routing, catalog propagation, and transport-ignorance invariants.

## 11) L1 Exit Criteria

L1 is complete when accepted:
1. async, stream-first Spine direction is locked,
2. composite routing key and affordance/capability semantics are locked,
3. Spine-owned capability catalog flow to Cortex is locked,
4. adapter-shell placement and WebSocket scope are locked,
5. migration path from `server.rs` to Spine adapters is accepted,
6. Body Endpoint ingress naming as `Sense` is locked.

Status: `READY_FOR_L2_APPROVAL`
