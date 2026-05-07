# Runtime API Boundary

Slice 2A records the host-independent runtime boundary before moving Rust code.

## MVT Anchors

- Objective & Hypothesis: define a `MoiraRuntime` API that Apple Universal can embed for the minimum Loom surface, while preserving the current Clotho, Lachesis, and Atropos service ownership.
- Guardrails Touched: Core remains the authority for runtime behavior, config schema, endpoint protocol, and observability emission. Moira owns local preparation, supervision, ingestion, storage, query, and future sandbox/ledger adapters. Human Interface hosts own platform-native UI.
- Verification: this slice is a task-packet artifact. The next implementation slice should compile the backend through a host-independent Rust API, with Tauri types contained in adapter code.

## Current Seams

The current backend already has useful internal service seams:

- `ClothoService` owns launch target registration, local build forging, release installation, profile document IO, and wake-input preparation.
- `LachesisService` owns DuckDB store access, OTLP receiver status, run/tick queries, and selected tick detail.
- `AtroposService` owns Core wake, stop, force-kill, process monitoring, terminal reason, and receiver readiness gating.
- Tauri command functions are mostly thin service facades.

The main extraction pressure points are:

- `AppPaths` lives under `app::state`, although every service depends on those paths.
- `app::bootstrap` chooses app data paths, creates services, starts the receiver, registers commands, and handles shutdown.
- `lachesis::receiver` and `lachesis::pulse` depend on `tauri::AppHandle` for live update events.
- `AtroposService::spawn_monitor` depends on `tauri::async_runtime::spawn`.
- The crate currently emits `rlib`, `staticlib`, and `cdylib`; the backend crate still includes Tauri build/runtime dependencies.

## Target Runtime Handle

`MoiraRuntime` is the process-local embedded backend handle.

Responsibilities:

- own service instances for Clotho, Lachesis, and Atropos
- receive explicit host-selected paths and receiver configuration
- start or report local resource ownership for receiver/store/supervision resources
- expose typed command/query facades
- emit or buffer runtime events through framework-neutral types
- shut down supervised Core and receiver resources during host exit

Candidate Rust shape:

```rust
pub struct MoiraRuntime {
    clotho: Arc<ClothoService>,
    lachesis: Arc<LachesisService>,
    atropos: Arc<AtroposService>,
}

impl MoiraRuntime {
    pub async fn open(config: MoiraRuntimeConfig) -> Result<Self, MoiraRuntimeError>;
    pub async fn status(&self) -> MoiraRuntimeStatus;
    pub async fn shutdown(&self) -> Result<MoiraShutdownOutcome, MoiraRuntimeError>;

    pub fn clotho(&self) -> ClothoHostApi<'_>;
    pub fn lachesis(&self) -> LachesisHostApi<'_>;
    pub fn atropos(&self) -> AtroposHostApi<'_>;
}
```

## Runtime Configuration

Hosts pass configuration explicitly.

Candidate fields:

- `paths: MoiraPaths`
- `receiver_bind: SocketAddr`
- `event_sink: Arc<dyn MoiraEventSink>`
- `task_spawner: Arc<dyn MoiraTaskSpawner>`
- later `platform: Arc<dyn MoiraPlatformAdapter>`

`MoiraPaths` should move out of `app::state`. It can keep derived path helpers for artifacts, profiles, runtime cache, release cache, and telemetry DB.

Apple Universal should choose the root path at the host boundary. Moira should create and validate owned subdirectories from that root.

## Owner Facades

Organize the host API by Moira owner for the first extraction. This matches the durable unit language and current service boundaries.

Clotho host API:

- `list_launch_targets`
- `register_known_local_build`
- `forge_local_build`
- `list_published_releases`
- `install_published_release`
- `list_profile_documents`
- `load_profile_document`
- `save_profile_document`
- `prepare_wake_input`

Atropos host API:

- `core_status`
- `wake_core`
- `stop_core`
- `force_kill_core`
- `stop_core_if_running`

Lachesis host API:

- `receiver_status`
- `list_runs`
- `list_ticks`
- `tick_detail`

The Apple UI may label `run_id` rows as wakes, while the raw data contract keeps `run_id`.

## Event Delivery

Replace Tauri events with a runtime event sink.

Candidate event:

```rust
pub enum MoiraEvent {
    LachesisUpdated(IngestPulse),
    ResourceStatusChanged(MoiraResourceStatus),
    CoreSupervisionChanged(CoreSupervisionStatus),
}
```

First Apple UI can poll status and query surfaces. Live pulses should still be represented in the runtime API so the Rust side has one event contract before Swift binding work.

## Resource Status

Resource conflicts are represented as runtime status with structured details.

First resource statuses:

- root/app directories ensured or faulted
- telemetry DB opened or faulted
- OTLP receiver awakening/listening/awake/faulted, including bind conflict detail
- Atropos supervision local state

Future resource statuses:

- owner lock
- attach endpoint
- sandbox adapter availability
- ledger adapter availability

## Implementation Bias For Next Slice

Create a host-independent Rust runtime crate before Swift binding work.

Candidate layout:

```text
moira/
├── runtime/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── runtime/
│       ├── clotho/
│       ├── lachesis/
│       ├── atropos/
│       └── platform/
└── src-tauri/
    └── src/
        └── app/
```

`moira/src-tauri` then becomes a transitional app adapter over `moira/runtime`.

## Slice 2A Boundaries

This slice records the boundary and extraction path.

Implementation begins in Slice 2B after this packet is accepted.

First implementation exclusions:

- Swift binding technology selection
- Apple UI changes
- Owner/Attach coordination
- sandbox adapter implementation
- ledger adapter implementation
- Tauri/Vue deletion
