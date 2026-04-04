# System State And Authority

## Authoritative Ownership

| State / Decision Area | Authority Owner | Participating Units | Notes |
|---|---|---|---|
| Cognition state persistence and restore | `core` (`continuity`) | `core` | Continuity owns persisted cognition-state guardrails. |
| Cognition transformation logic | `core` (`cortex`) | `core` | Transformation executes within core cognition boundary. |
| Physical state snapshot (ledger/descriptor/proprioception) | `core` (`stem` + related subsystems) | `core` | Physical-state mutation authority remains inside core runtime. |
| Endpoint dispatch routing and terminal outcome generation | `core` (`spine`) | `core`, `cli`, `apple-universal` | External units consume outcomes, core owns routing authority. |
| Runtime observability export policy | `core` | `core`, `moira`, endpoint units (consumers) | Core controls runtime observability export surfaces. |
| Local Core artifact/install state | `moira` | `moira`, `core` | `#8` publishes release artifacts as the producer workflow; Moira consumes them and owns local selection, checksum verification, install isolation, and activation state. |
| Local Core supervision state | `moira` | `moira`, `core` | Moira owns local wake/stop/supervision state; core owns runtime behavior after launch. |
| Local observability ingestion/storage/query state | `moira` | `moira`, `core` | Moira owns local OTLP log ingestion, storage, and operator-facing query state. |
| External endpoint UX/local history | `cli` / `apple-universal` (local app state) | endpoint unit + `core` protocol contract | Local UI/app state is endpoint-owned; core remains runtime authority. |

## Authority Rule

When ownership is unclear, treat the area as unresolved design debt and document explicit authority before further behavior expansion.
