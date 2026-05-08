# Runtime API Implementation

Slice 2B introduced the first host-independent Rust runtime crate.

## Implemented Shape

New crate:

- `moira/runtime`

Runtime API:

- `MoiraRuntime`
- `MoiraRuntimeConfig`
- `MoiraPaths`
- `MoiraRuntimeStatus`
- `MoiraResourceStatus`
- `MoiraEvent`
- `MoiraEventSink`
- `MoiraTaskSpawner`

Owner modules moved into the runtime crate:

- `moira/runtime/src/clotho`
- `moira/runtime/src/lachesis`
- `moira/runtime/src/atropos`

Historical transition state: the desktop crate depended on `moira-runtime` and kept:

- Tauri bootstrap
- existing Tauri command names
- Tauri app path resolution
- Tauri event sink for `lachesis-updated`
- Tauri task spawner adapter
- app-exit runtime shutdown call

## Boundary Result

Slice 2B moved backend behavior into `moira/runtime`; Slice 6 later retired the desktop adapter.

The runtime crate owns backend behavior and can be compiled/tested independently from Tauri.

`Lachesis` receiver event emission now flows through `MoiraEventSink`.

`Atropos` monitor spawning now flows through `MoiraTaskSpawner`.

`MoiraRuntime::status` returns runtime lifecycle, resource status, receiver status, and Core supervision status.

## Verification

Passed on 2026-05-07:

- `cargo check --manifest-path moira/runtime/Cargo.toml --locked`
- `cargo test --manifest-path moira/runtime/Cargo.toml --locked`
- Transitional desktop adapter check passed during this slice before retirement.

Runtime test coverage includes receiver bind conflict reporting as `MoiraResourceState::Conflict`.

Slice 2C moved that receiver-conflict coverage into public integration tests and added runtime open, Clotho prepare, Lachesis ingest, and Atropos supervision integration coverage. See `RUNTIME-INTEGRATION-TESTS.md`.

## Next Slice Pressure

Apple Universal uses the first C ABI binding proof for the minimum Loom surface.

Follow-on binding design can decide how to scale Rust/Swift calls:

- UniFFI-style generated Swift bindings
- C ABI plus Swift wrapper
- Swift Package wrapping a local Rust artifact

The current Apple binding proves runtime status and minimum Loom snapshots. Broader Clotho/Lachesis/Atropos operations remain follow-on design.
