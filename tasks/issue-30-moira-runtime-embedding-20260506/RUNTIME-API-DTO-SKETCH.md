# Runtime API DTO Sketch

This file sketches the typed data crossing the host-independent Moira runtime boundary.

The names here are design candidates. Existing Rust DTOs should be reused where they already match the host contract.

## Runtime

```rust
pub struct MoiraRuntimeConfig {
    pub paths: MoiraPaths,
    pub receiver_bind: SocketAddr,
    pub event_sink: Arc<dyn MoiraEventSink>,
    pub task_spawner: Arc<dyn MoiraTaskSpawner>,
}

pub struct MoiraPaths {
    pub root: PathBuf,
}

pub struct MoiraRuntimeStatus {
    pub lifecycle: MoiraRuntimeLifecycle,
    pub resources: Vec<MoiraResourceStatus>,
    pub receiver: ReceiverStatus,
    pub core: CoreSupervisionStatus,
}

pub enum MoiraRuntimeLifecycle {
    Opening,
    Ready,
    Degraded,
    Closing,
    Closed,
}
```

`MoiraPaths` should keep helper methods for derived directories. The host only needs to pass the root unless a later binding needs advanced overrides.

## Resource Status

```rust
pub struct MoiraResourceStatus {
    pub kind: MoiraResourceKind,
    pub state: MoiraResourceState,
    pub label: String,
    pub detail: Option<String>,
}

pub enum MoiraResourceKind {
    Directory,
    TelemetryStore,
    OtlpReceiver,
    CoreSupervisor,
    PlatformAdapter,
}

pub enum MoiraResourceState {
    Available,
    Claiming,
    Claimed,
    Degraded,
    Conflict,
    Faulted,
}
```

Resource status should describe recoverable conflict and degraded states for Apple UI, so hosts can avoid parsing string errors.

## Events

```rust
pub trait MoiraEventSink: Send + Sync {
    fn emit(&self, event: MoiraEvent);
}

pub enum MoiraEvent {
    LachesisUpdated(IngestPulse),
    ResourceStatusChanged(MoiraResourceStatus),
    CoreSupervisionChanged(CoreSupervisionStatus),
}
```

Apple Universal can start with polling. The event DTOs keep the backend contract ready for live updates and future host adapters.

## Clotho

Existing DTOs that can cross the host boundary:

- `LaunchTargetRef`
- `LaunchTargetProvenance`
- `LaunchTargetReadiness`
- `LaunchTargetSummary`
- `KnownLocalBuildRegistration`
- `ForgeLocalBuildRequest`
- `PublishedReleaseSummary`
- `InstallPublishedReleaseRequest`
- `ProfileRef`
- `ProfileDocumentSummary`
- `ProfileDocument`
- `SaveProfileDocumentRequest`
- `WakeInputRequest`
- `PreparedWakeInput`

Host-facing invariants:

- path values returned to hosts are canonical where the current service already canonicalizes them
- ref ids keep ASCII segment validation
- profile wrapper parsing and Core config materialization stay inside Clotho
- Core config schema authority stays in Core

## Atropos

Current `RuntimeStatus` is better named `CoreSupervisionStatus` at the host boundary.

Candidate shape:

```rust
pub struct CoreSupervisionStatus {
    pub phase: SupervisionPhase,
    pub target_label: Option<String>,
    pub executable_path: Option<PathBuf>,
    pub working_dir: Option<PathBuf>,
    pub profile_path: Option<PathBuf>,
    pub pid: Option<u32>,
    pub terminal_reason: Option<String>,
}
```

Existing `SupervisionPhase` values remain useful:

- `Idle`
- `Waking`
- `Running`
- `Stopping`
- `Terminated`

Atropos should remain the owner of receiver readiness gating before Core wake.

## Lachesis

Existing DTOs that can cross the host boundary:

- `ReceiverStatus`
- `RunSummary`
- `TickSummary`
- `EventRecord`
- `TickDetail`
- `IngestPulse`

Contract notes:

- `RunSummary.run_id` remains the data identifier.
- Apple UI can present run rows as wake history.
- `TickDetail.raw` is the primary inspection source.
- `TickDetail.cortex`, `stem`, and `spine` are convenience groupings derived from raw records.
- JSON columns stay as JSON values at the Rust boundary and can become Swift Codable wrappers later.

## Errors

Replace host-boundary `String` errors with typed runtime errors during extraction.

Candidate shape:

```rust
pub enum MoiraRuntimeError {
    InvalidConfig { message: String },
    ResourceConflict { resource: MoiraResourceStatus },
    Io { message: String },
    Store { message: String },
    Receiver { message: String },
    Process { message: String },
    Profile { message: String },
    Artifact { message: String },
}
```

The first implementation can keep internal `String` errors and convert them at the runtime boundary.

## Shutdown

```rust
pub struct MoiraShutdownOutcome {
    pub core: Option<CoreSupervisionStatus>,
    pub resources: Vec<MoiraResourceStatus>,
}
```

Shutdown should request graceful Core stop through Atropos and release receiver resources owned by the process-local runtime.
