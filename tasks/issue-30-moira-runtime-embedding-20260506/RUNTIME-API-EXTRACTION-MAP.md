# Runtime API Extraction Map

This historical map records how the former Tauri-backed Moira code moved into the host-independent runtime API.

## Former Commands To Runtime Facades

| Former Tauri command | Runtime facade | Owner |
| --- | --- | --- |
| `register_known_local_build` | `runtime.clotho().register_known_local_build` | Clotho |
| `prepare_wake_input` | `runtime.clotho().prepare_wake_input` | Clotho |
| `forge_local_build` | `runtime.clotho().forge_local_build` | Clotho |
| `list_launch_targets` | `runtime.clotho().list_launch_targets` | Clotho |
| `list_published_releases` | `runtime.clotho().list_published_releases` | Clotho |
| `install_published_release` | `runtime.clotho().install_published_release` | Clotho |
| `list_profile_documents` | `runtime.clotho().list_profile_documents` | Clotho |
| `load_profile_document` | `runtime.clotho().load_profile_document` | Clotho |
| `save_profile_document` | `runtime.clotho().save_profile_document` | Clotho |
| `runtime_status` | `runtime.atropos().core_status` | Atropos |
| `wake` | `runtime.atropos().wake_core` | Atropos |
| `stop` | `runtime.atropos().stop_core` | Atropos |
| `force_kill` | `runtime.atropos().force_kill_core` | Atropos |
| `receiver_status` | `runtime.lachesis().receiver_status` | Lachesis |
| `list_runs` | `runtime.lachesis().list_runs` | Lachesis |
| `list_ticks` | `runtime.lachesis().list_ticks` | Lachesis |
| `tick_detail` | `runtime.lachesis().tick_detail` | Lachesis |

## Code Movement Candidates

| Current path | Target owner | Notes |
| --- | --- | --- |
| `moira/src-tauri/src/app/state.rs` | `moira/runtime/src/runtime/paths.rs` and `runtime/state.rs` | Move `AppPaths` first. Rename to `MoiraPaths`. Runtime state should own service arcs. |
| `moira/src-tauri/src/clotho/*` | `moira/runtime/src/clotho/*` | Mostly Tauri-free already. Keep release provider injection for tests. |
| `moira/src-tauri/src/lachesis/model.rs` | `moira/runtime/src/lachesis/model.rs` | DTOs are already host-friendly. |
| `moira/src-tauri/src/lachesis/store/*` | `moira/runtime/src/lachesis/store/*` | Store is Tauri-free. |
| `moira/src-tauri/src/lachesis/normalize.rs` | `moira/runtime/src/lachesis/normalize.rs` | Tauri-free. |
| `moira/src-tauri/src/lachesis/receiver.rs` | `moira/runtime/src/lachesis/receiver.rs` | Replace `AppHandle` with event sink before or during move. |
| `moira/src-tauri/src/lachesis/pulse.rs` | `moira/runtime/src/runtime/events.rs` | Make the pulse event framework-neutral. |
| `moira/src-tauri/src/atropos/*` | `moira/runtime/src/atropos/*` | Replace Tauri task spawning with runtime spawner. |
| `moira/src-tauri/src/app/bootstrap.rs` | `moira/src-tauri/src/app/bootstrap.rs` adapter only | Keep Tauri setup, command registration, path resolution, and adapter event sink here. |
| `moira/src-tauri/src/app/commands/*` | transitional adapter facades | Commands call `MoiraRuntime` facades from managed Tauri state. |

## Adapter Responsibilities

Tauri adapter should own:

- resolving Tauri app local data directory into `MoiraPaths`
- building a Tauri event sink for `MoiraEvent`
- starting the runtime during setup
- exposing Tauri commands as thin facades
- requesting runtime shutdown on app exit

Runtime should own:

- service construction
- path directory creation and validation
- receiver lifecycle
- store open and query
- Core process supervision
- resource status shaping
- typed command/query facades

## Extraction Order

1. Create `moira/runtime` crate and add it to the workspace.
2. Move `AppPaths` as `MoiraPaths`.
3. Move Clotho modules and compile with direct unit tests.
4. Move Lachesis store/model/normalize modules.
5. Introduce `MoiraEventSink`; move Lachesis receiver and pulse emission.
6. Introduce `MoiraTaskSpawner`; move Atropos modules.
7. Add `MoiraRuntime::open`, facades, `status`, and `shutdown`.
8. Convert Tauri app state to hold `MoiraRuntime`.
9. Keep existing Tauri commands as compatibility facades through the transition.
10. Run Moira backend checks and existing frontend tests.

## Verification Gate For Slice 2B

Required:

- `cargo check --manifest-path moira/runtime/Cargo.toml --locked`
- `cargo test --manifest-path moira/runtime/Cargo.toml --locked`
- Transitional desktop adapter check during extraction.

Optional when DuckDB linking requires source build:

- `cargo check --manifest-path moira/runtime/Cargo.toml --locked --no-default-features --features duckdb-bundled`
- Transitional desktop adapter bundled-DuckDB check during extraction.

Acceptance:

- backend services are reachable through `MoiraRuntime`
- desktop-shell types stay out of `moira/runtime`
- tests instantiate runtime services through direct harnesses independent of Tauri app bootstrap
- existing command names remain callable during the transition

## Risks To Watch

- DuckDB native dependency behavior may differ between the runtime crate and Tauri crate.
- Moving modules can expose private visibility choices that previously worked inside one crate.
- `tauri::async_runtime` currently hides task-spawner ownership.
- Swift binding needs async and large JSON payload handling; that selection belongs after Rust runtime shape compiles.
