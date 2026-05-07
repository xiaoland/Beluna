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

The transitional Tauri crate now depends on `moira-runtime` and keeps:

- Tauri bootstrap
- existing Tauri command names
- Tauri app path resolution
- Tauri event sink for `lachesis-updated`
- Tauri task spawner adapter
- app-exit runtime shutdown call

## Boundary Result

`moira/src-tauri` is now an adapter over `moira/runtime`.

The runtime crate owns backend behavior and can be compiled/tested independently from Tauri.

`Lachesis` receiver event emission now flows through `MoiraEventSink`.

`Atropos` monitor spawning now flows through `MoiraTaskSpawner`.

`MoiraRuntime::status` returns runtime lifecycle, resource status, receiver status, and Core supervision status.

## Verification

Passed on 2026-05-07:

- `cargo check --manifest-path moira/runtime/Cargo.toml --locked`
- `cargo test --manifest-path moira/runtime/Cargo.toml --locked`
- `cargo check --manifest-path moira/src-tauri/Cargo.toml --locked`

Runtime test coverage includes receiver bind conflict reporting as `MoiraResourceState::Conflict`.

Slice 2C moved that receiver-conflict coverage into public integration tests and added runtime open, Clotho prepare, Lachesis ingest, and Atropos supervision integration coverage. See `RUNTIME-INTEGRATION-TESTS.md`.

## Next Slice Pressure

Apple Universal still needs a binding strategy before it can call `moira/runtime`.

The next technical design decision is how to cross Rust/Swift:

- UniFFI-style generated Swift bindings
- C ABI plus Swift wrapper
- Swift Package wrapping a local Rust artifact

The first Apple binding should prove runtime status and receiver status before exposing broader Clotho/Lachesis/Atropos operations.
