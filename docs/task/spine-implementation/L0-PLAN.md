# L0 Plan - Spine Implementation
- Task Name: `spine-implementation`
- Stage: `L0` (request + context analysis only)
- Date: `2026-02-12`
- Status: `DRAFT_FOR_APPROVAL`

## 1) Problem Deconstruction
Requested deliverable is not a small refactor. It is a semantic completion of Spine with strict invariants plus adapter shells:

1. Spine as the **only** channel between Mind (Cortex + Non-Cortex) and Body (Endpoints).
2. Spine keeps **total transport ignorance**:
- no network/protocol/process/physical-location logic.
3. Spine provides **universal endpoint abstraction**:
- local Rust function endpoint and remote gRPC endpoint must be treated uniformly.
4. Spine provides **mechanical routing only**:
- table lookup + pass-through dispatch, no policy/intelligence.
5. Build adapter shells for easier endpoint development:
- UnixSocket adapter (migrate `core/src/server.rs` concerns)
- HTTP adapter
- adapters are not Body Endpoints; they wrap transport/runtime concerns.

## 2) Context Collection (Sub-agent Style)
To reduce cognitive load, context was split into two parallel tracks:

1. Track A (local code/doc scan):
- `core/src/spine/*`, `core/src/continuity/*`, `core/src/server.rs`, `core/src/protocol.rs`
- `core/tests/spine/*`, `core/tests/cortex_continuity_flow.rs`
- `docs/features/spine/*`, `docs/contracts/spine/*`, `docs/modules/spine/*`, `docs/overview.md`

2. Track B (external architecture references via Firecrawl):
- Hexagonal architecture and ports/adapters boundary
- gRPC retry/deadline placement
- generic service abstraction and middleware layering
- Unix socket listener runtime behavior

## 3) Current Codebase Reality

### 3.1 Spine currently exists as contract-level MVP
Current module (`core/src/spine/*`) already has:
1. `SpineExecutorPort` trait with `execute_admitted(AdmittedActionBatch)`.
2. `SpineExecutionReport` + ordered `SpineEvent` types.
3. `DeterministicNoopSpine` implementation.

This matches earlier MVP docs, but it is still a single backend implementation (noop), not a universal endpoint abstraction.

### 3.2 Mechanical ordering and settlement linkage are already present
1. Events carry `reserve_entry_id` and `cost_attribution_id`.
2. Continuity reconciliation sorts by `seq_no` and applies settle/refund deterministically.
3. Existing tests validate ordering and linkage semantics.

### 3.3 Universal endpoint abstraction is missing
There is no first-class Spine-level endpoint abstraction such as:
1. endpoint interface/trait independent of transport,
2. endpoint registry/route table,
3. route lookup semantics from admitted action -> endpoint key.

### 3.4 Adapter shell layer is missing
`core/src/server.rs` currently mixes:
1. Unix socket transport handling,
2. NDJSON protocol parsing,
3. runtime wiring (cortex reactor, continuity bridge, message loops).

No dedicated `spine` adapter shell exists yet for Unix socket/HTTP.

### 3.5 Docs currently understate requested scope
`docs/features/spine/*` and module/contract docs still describe Spine as “contracts + deterministic noop MVP”, which is behind this task’s requested implementation scope.

## 4) Invariant Fit-Gap Matrix
1. Total Transport Ignorance
- Status: `PARTIAL`
- Reason: current Spine module itself is transport-agnostic, but there is no explicit structure preventing future transport leakage; runtime transport is still centralized in `server.rs`.

2. Universal Endpoint Abstraction
- Status: `GAP`
- Reason: no endpoint trait + registry + uniform dispatch model for native/remote endpoints.

3. Mechanical Routing (table lookup only)
- Status: `GAP`
- Reason: no route table in Spine; noop implementation does not route by affordance/capability/endpoint map.

4. Adapter shells (UnixSocket + HTTP) for endpoint development
- Status: `GAP`
- Reason: no spine adapter layer; Unix socket server is runtime monolith.

## 5) Architectural Trade-offs Identified
1. Async Spine API shape
- Keep sync trait: minimal change, but awkward for HTTP/gRPC endpoints.
- Move to async trait/future-based dispatch: needed for real transports, but propagates API/test changes.

2. Route table ownership
- Owned inside Spine executor: strong invariant locality.
- Injected by runtime: more flexible dynamic updates, but higher integration complexity.

3. Routing key choice
- Route by `capability_handle`: coarse and stable.
- Route by `affordance_key` + optional capability constraints: finer control, larger config surface.

4. Adapter boundary placement
- Keep adapters under `server` runtime namespace: easier migration.
- Move adapters under `spine/adapters`: cleaner “Spine shell” semantics, larger refactor.

5. Event semantics for missing route/endpoint failure
- Emit `ActionRejected` with deterministic code: preserves ledger terminality.
- Return hard engine error for whole batch: simpler internals, risks stalling admitted batch processing.

## 6) External Source Findings (Firecrawl)
1. Ports/Adapters boundary should isolate technology concerns from core logic.
- Source: Alistair Cockburn, “Hexagonal Architecture”
- Link: https://alistair.cockburn.us/hexagonal-architecture
- Relevance: validates Spine core as inside logic and adapters as technology-specific shells.

2. Retry/backoff logic belongs in client transport layer, not business core.
- Source: gRPC Retry Guide
- Link: https://grpc.io/docs/guides/retry/
- Relevance: supports placing retry/throttle/pushback behaviors inside endpoint/adapters, not Spine router.

3. Deadline/timeout handling is transport call behavior and should be explicit at client/server edges.
- Source: gRPC Deadlines Guide
- Link: https://grpc.io/docs/guides/deadlines/
- Relevance: supports keeping timeout/deadline logic out of Spine core and inside endpoint implementations.

4. A uniform request/response service trait enables middleware wrappers across different transports.
- Source: `tower-service` `Service` trait docs
- Link: https://docs.rs/tower-service/latest/tower_service/trait.Service.html
- Relevance: useful precedent for universal endpoint abstraction with wrappers like timeout/retry.

5. Unix socket listener has transport-level accept/error concerns that should remain outside core dispatch logic.
- Source: Tokio `UnixListener` docs
- Link: https://docs.rs/tokio/latest/tokio/net/struct.UnixListener.html
- Relevance: supports extracting current socket lifecycle/error handling into adapter shell.

## 7) L0 Recommendation (Scope Boundary for L1)
L1 should target a two-layer Spine implementation:

1. Spine Core (ignorant, mechanical):
- route table abstraction
- endpoint dispatch abstraction
- deterministic ordering + event conversion
- zero transport/protocol logic

2. Spine Adapter Shells (transport-aware wrappers):
- UnixSocket adapter (migrating transport/protocol concerns currently in `server.rs`)
- HTTP adapter (parallel shell pattern)
- retry/timeout/serialization/network details stay inside concrete endpoint or adapter shell

3. Keep Continuity contract stable where possible:
- preserve admitted-action-only entry
- preserve ordered settlement event semantics

4. Contract-first delivery:
- update/add tests for universal endpoint equivalence and routing-table determinism before full runtime refactor.

## 8) Open Questions Requiring User Decision
1. Should `SpineExecutorPort::execute_admitted` become async in this task?
2. What should be the canonical routing key in Spine table lookup?
- `affordance_key`
- `capability_handle`
- composite (`affordance_key`, `capability_handle`)
3. Should UnixSocket/HTTP adapters be implemented as:
- Spine-internal module (`core/src/spine/adapters/*`), or
- runtime/server module (`core/src/server/*` style)?
4. HTTP adapter scope for this task:
- skeleton shell + tests only, or
- runnable ingress endpoint integrated into runtime loop?

## 9) Working Assumptions (If Not Overridden)
1. Async Spine dispatch is acceptable for endpoint universality.
2. Routing key defaults to composite (`affordance_key`, `capability_handle`) for deterministic uniqueness.
3. UnixSocket and HTTP adapters will be created under Spine-oriented adapter modules, while preserving current runtime behavior.
4. Missing route/endpoint results in per-action `ActionRejected` event, not whole-batch abort.

## 10) L0 Exit Criteria
L0 is complete when:
1. invariant fit-gaps are explicit,
2. migration surfaces (`spine`, `continuity`, `server`) are mapped,
3. external references support boundary decisions,
4. user decisions needed for L1 are listed.

Status: `READY_FOR_L1_APPROVAL`
