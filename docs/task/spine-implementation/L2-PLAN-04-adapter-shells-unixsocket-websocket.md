# L2-04 - Adapter Shells: UnixSocket And WebSocket
- Task Name: `spine-implementation`
- Stage: `L2` detailed file
- Date: `2026-02-12`
- Status: `DRAFT_FOR_APPROVAL`

## 1) Adapter Role Definition

Adapters are shells, not Body Endpoints.

Responsibilities:
1. expose Spine <-> Body Endpoint interface in transport-specific form,
2. transport/protocol lifecycle,
3. parsing/serialization,
4. retries/timeouts/backpressure (if needed),
5. forwarding normalized messages into runtime channels.

Non-responsibilities:
1. semantic routing decisions,
2. admission logic,
3. ledger logic,
4. non-mechanical policy interpretation.

## 2) Canonical Body->Spine Ingress Data: Sense

Lock terminology:
1. Body Endpoint -> Spine ingress data is named `Sense`.
2. `Sense` payload shape (wire-level canonical fields):
- `sense_id: String`
- `source: String`
- `payload: serde_json::Value`

Runtime mapping:
- `Sense` maps 1:1 into Cortex ingress `SenseDelta` fields.

## 3) UnixSocket Adapter (Server.rs migration)

### 3.1 Migration target from `core/src/server.rs`
Move these concerns into `core/src/spine/adapters/unix_socket.rs`:
1. socket path preparation and cleanup,
2. accept loop and per-client task spawning,
3. line-oriented NDJSON parse loop,
4. mapping wire line -> internal message.

### 3.2 API sketch
```rust
pub struct UnixSocketAdapter {
    pub socket_path: PathBuf,
}

impl UnixSocketAdapter {
    pub async fn run(
        &self,
        tx: mpsc::Sender<ClientMessage>,
        shutdown: CancellationToken,
    ) -> anyhow::Result<()>;
}
```

### 3.3 Behavior rules
1. ignore empty lines.
2. invalid line -> log + continue (no process crash).
3. each valid message forwarded through shared channel.
4. preserve current graceful cleanup behavior for socket file.

## 4) WebSocket Adapter

### 4.1 Scope
Implement runnable WebSocket shell under `core/src/spine/adapters/websocket.rs`.

### 4.2 Dependency
Use `axum` websocket support (or equivalent async websocket runtime).

### 4.3 Bi-directional contract (v1)

Connection endpoint:
1. `GET /v1/stream` (websocket upgrade)

Inbound (Body Endpoint -> Spine):
1. `sense` messages (required canonical ingress)
2. optional control messages (`exit`, etc.) for runtime operations

Outbound (Spine/Runtime -> Body Endpoint):
1. adapter-level acknowledgements/errors for ingress processing
2. optional pushed runtime events via egress channel (when producers exist)

This keeps WebSocket bi-directional from day one while allowing minimal producer set initially.

### 4.4 API sketch
```rust
pub struct WebSocketAdapterConfig {
    pub bind_addr: String,
    pub path: String,
    pub max_message_bytes: usize,
}

pub async fn run_websocket_adapter(
    cfg: WebSocketAdapterConfig,
    ingress_tx: mpsc::Sender<ClientMessage>,
    egress_rx: tokio::sync::broadcast::Receiver<ServerOutboundMessage>,
    shutdown: CancellationToken,
) -> anyhow::Result<()>;
```

## 5) Shared Wire Layer

Add `core/src/spine/adapters/wire.rs`:
1. canonical wire message enum with `serde(tag="type")`.
2. `sense` envelope as first-class message.
3. parse helpers reused by:
- UnixSocket line handler,
- WebSocket text-frame handler.

`core/src/protocol.rs` is removed from canonical runtime path; `spine/adapters/wire.rs` is authoritative.

## 6) Runtime Orchestration Changes

`core/src/server.rs` becomes runtime coordinator:

Pseudo-flow:
```text
1. construct cortex pipeline/reactor.
2. construct continuity engine with async spine port.
3. construct spine executor + registry.
4. bootstrap endpoint registrations (initial capabilities).
5. create shared ingress message channel.
6. create shared egress broadcast channel for websocket push.
7. spawn UnixSocket adapter task.
8. if configured, spawn WebSocket adapter task.
9. run select loop:
   - signals
   - incoming messages (including Sense)
   - reactor outputs
10. shutdown adapters and reactor gracefully.
```

## 7) Backpressure And Reliability

1. adapter -> runtime ingress channel remains bounded at core ingress boundary.
2. when full:
- UnixSocket adapter drops with deterministic warning,
- WebSocket adapter emits deterministic backpressure error message and may close client session for overload control.
3. adapter-level retries/timeouts are adapter-local only (no leakage into spine core).

## 8) Capability Catalog Source In Runtime

`CortexIngressAssembler` update:
1. remove direct external catalog ownership.
2. obtain catalog snapshot from Spine executor/registry bridge.
3. include latest snapshot when building each `ReactionInput`.

Result:
- Cortex senses capabilities from Spine-owned state, not external pushed catalog blobs.

## 9) Config Changes

Add optional config block:
```json
"spine": {
  "websocket": {
    "bind_addr": "127.0.0.1:9020",
    "path": "/v1/stream",
    "max_message_bytes": 65536
  }
}
```

Cutover:
1. config shape is spine-centric; old top-level runtime config shape is not required to remain compatible.
2. if `spine.websocket` missing, WebSocket adapter is disabled.

## 10) L2-04 Exit Criteria
This file is complete when:
1. adapter/runtime boundary is explicit,
2. Sense ingress naming is locked,
3. `server.rs` migration destination is concrete,
4. WebSocket adapter API and behavior are implementation-ready.
