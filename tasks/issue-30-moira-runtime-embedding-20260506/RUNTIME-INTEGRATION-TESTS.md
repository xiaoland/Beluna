# Runtime Integration Tests

This note captures the first integration coverage added for `moira/runtime`.

## Test Homes

- `moira/runtime/tests/runtime_open.rs`
- `moira/runtime/tests/clotho_prepare.rs`
- `moira/runtime/tests/lachesis_ingest.rs`
- `moira/runtime/tests/atropos_supervision.rs`
- `moira/runtime/tests/common/mod.rs`

## Covered Runtime Paths

1. Runtime open and resource status
- opens `MoiraRuntime` from host-provided paths
- waits for Lachesis receiver readiness
- verifies Moira runtime directories are created
- verifies OTLP receiver resource reaches `Claimed`

2. Receiver bind conflict
- holds a local TCP port before opening runtime
- verifies receiver fault maps to `MoiraResourceState::Conflict`

3. Clotho wake preparation
- registers a known local build through `runtime.clotho()`
- writes a JSONC profile wrapper under the runtime root
- prepares wake input through the public runtime boundary
- verifies profile-backed Core config materialization

4. Lachesis OTLP ingest
- starts the runtime receiver
- sends native OTLP logs through a tonic client
- verifies run, tick, and selected tick raw-first detail projections

5. Atropos supervision
- registers a Unix process fixture
- wakes it through `runtime.atropos()`
- requests graceful stop
- verifies terminal supervision state

## Verification

Passed on 2026-05-07:

- `cargo test --manifest-path moira/runtime/Cargo.toml --locked`
- `cargo check --manifest-path moira/src-tauri/Cargo.toml --locked`

Runtime test count after this slice:

- 17 unit tests
- 5 integration tests
