# L3-04 - Adapter And Runtime Implementation
- Task Name: `spine-implementation`
- Stage: `L3` detail: adapter/runtime implementation
- Date: `2026-02-12`
- Status: `IMPLEMENTED`

## 1) Wire Model (`spine/adapters/wire.rs`)

Canonical ingress parse path implemented for:
1. `sense` (primary runtime trigger)
2. `exit`
3. `env_snapshot`
4. `admission_feedback`
5. `capability_catalog_update` (ignored by runtime; Spine owns catalog)
6. `cortex_limits_update`
7. `intent_context_update`

Rules:
1. unknown envelope type rejected.
2. unknown fields rejected (`deny_unknown_fields`).
3. parsed messages map to `ClientMessage` enum.

## 2) UnixSocket Adapter (`spine/adapters/unix_socket.rs`)

Implemented behavior:
1. stale socket cleanup + bind + accept loop.
2. one spawned task per client connection.
3. line-based NDJSON parse via shared wire parser.
4. valid messages forwarded into adapter channel.
5. invalid messages logged and ignored without dropping listener.

Shutdown:
1. adapter receives `CancellationToken`.
2. on cancellation, listener loop exits and socket path is cleaned up.

## 3) Runtime Orchestration (`server.rs`)

Implemented orchestration:
1. startup builds cortex pipeline and reactor.
2. startup builds continuity engine (`with_defaults`) which now boots routing spine + endpoint registry.
3. runtime keeps a `CortexIngressAssembler` for sense/context/snapshot windows.
4. capability catalog for Cortex is refreshed from Spine catalog snapshot.
5. reaction results are fed into async continuity processing.
6. on exit/signal, cancellation token stops adapter and runtime joins tasks cleanly.

## 4) MVP Config Reality

Current shipped shape remains:
1. `Config.socket_path` drives UnixSocket adapter.
2. no spine-specific config block in this MVP change set.

## 5) Completion Gates
1. adapter compiles and runs under tokio runtime.
2. runtime loop operates with adapter + cortex + continuity.
3. transport logic is not implemented in spine core routing modules.

Status: `DONE`
