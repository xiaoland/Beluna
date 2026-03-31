# Backend Refactor Target

This note defines the cleanup-stage backend landing target for `moira/src-tauri`.
It is a procedural buffer anchored on the current Moira Unit TDD, not an authoritative contract by itself.

## Purpose

- turn the Unit TDD backend split into a concrete Rust module target
- keep current Lachesis and Loom behavior stable during cleanup
- prevent new Clotho or Atropos work from extending the current Lachesis-heavy modules into catch-all owners
- make the first post-cleanup feature slice land on explicit module boundaries instead of on `lib.rs`, `commands.rs`, and ad hoc shared state

## Cleanup-Stage Boundary

The cleanup stage is behavior-preserving.
It should not change the current operator-facing observability surface:

- keep the current Tauri observability commands:
  - `receiver_status`
  - `list_runs`
  - `list_ticks`
  - `tick_detail`
- keep the current live ingest event name: `lachesis-updated`
- keep the current Lachesis data authority:
  - OTLP logs receiver
  - raw-event persistence
  - `runs` and `ticks`
  - selected-tick inspection
- do not use cleanup work as a vehicle for new GitHub Releases UX, broad supervision UI, or first-party endpoint-app launch flow

## Backend Module Target

The backend target remains the Unit TDD split:

1. `app`
- owns process bootstrap, shared app state, Tauri command registration, and app-wide event wiring
- does not own Clotho, Lachesis, or Atropos behavior

2. `clotho`
- owns wake-input preparation before Core starts
- owns published artifact discovery, checksum trust, install isolation, local source-build orchestration, JSONC profile documents, active profile selection, and schema-validation interactions with Core authority
- should keep literal internal seams such as `artifacts` and `profiles` rather than mythologizing every submodule

3. `lachesis`
- owns OTLP receiver lifecycle, log normalization, raw-event persistence, projections, query APIs, and ingest pulses
- remains the owner of current Lachesis behavior
- does not become a convenience home for process control or Clotho persistence

4. `atropos`
- owns wake, graceful stop, force-kill, supervised process state, readiness gating, and terminal reason tracking
- does not own Clotho preparation data or Lachesis storage

## Target Module Shape

The target is a module-oriented tree with one owner per durable concern.
The exact filenames can still move, but the ownership split below is the intended landing shape.

```text
moira/src-tauri/src/
  lib.rs
  main.rs
  app/
    mod.rs
    bootstrap.rs
    state.rs
    commands/
      mod.rs
      lachesis.rs
      clotho.rs
      atropos.rs
  clotho/
    mod.rs
    service.rs
    artifacts.rs
    profiles.rs
    model.rs
  lachesis/
    mod.rs
    service.rs
    receiver.rs
    normalize.rs
    pulse.rs
    query.rs
    model.rs
    store/
      mod.rs
      schema.rs
      read.rs
      write.rs
  atropos/
    mod.rs
    service.rs
    process.rs
    state.rs
    model.rs
```

Notes:

- `lib.rs` should collapse into a thin entrypoint that delegates to `app::bootstrap`.
- `app` should compose top-level modules and expose transport only.
- `clotho`, `lachesis`, and `atropos` are the top-level backend owners because they match Moira's product-facing roles.
- inside those top-level modules, functional names should stay literal where that improves readability and grep-ability.
- `lachesis` can stay deeper than the other modules because it already has transport, persistence, and query concerns in production code.
- `clotho` and `atropos` may begin as thin skeletons during cleanup, but they need real homes before their first feature slice lands.

## Current Code To Target Mapping

### Entrypoint and App Wiring

- `moira/src-tauri/src/lib.rs`
  - target owner: `app`
  - move bootstrap, app-local data-root setup, service construction, command registration, and background task spawning into `app::bootstrap`

- `moira/src-tauri/src/state.rs`
  - split into:
    - `app::state`
    - `lachesis::receiver`
    - future `atropos::state`
    - future `clotho::artifacts`
    - future `clotho::profiles`
  - current issue: app composition state and Lachesis runtime state are fused
  - current issue: the current receiver-facing `wake_state` wording will collide with future Core wake supervision semantics and should not survive as a shared state concept

### Command Surface

- `moira/src-tauri/src/commands.rs`
  - target owner: `app::commands::lachesis`
  - current commands should remain transport façades only
  - current issue: commands reach directly into `store` and `receiver` instead of an owner boundary

### Lachesis Runtime

- `moira/src-tauri/src/ingest/mod.rs`
  - target owner: `lachesis::receiver` and `lachesis::pulse`
  - current issue: OTLP transport lifecycle and app event emission are coupled in a top-level module

- `moira/src-tauri/src/ingest/otlp.rs`
  - split into:
    - `lachesis::receiver`
    - `lachesis::normalize`
    - `lachesis::service`
  - current issue: one file handles OTLP transport, normalization, persistence invocation, receiver-state mutation, and Tauri event emission

- `moira/src-tauri/src/store/mod.rs`
  - target owner: `lachesis::store`
  - keep DuckDB open/init concern here, but move schema bootstrap into `store::schema`

- `moira/src-tauri/src/store/read.rs`
  - target owner: `lachesis::store::read`
  - current issue: query code contains projection-level subsystem grouping that belongs to observability service/query logic, not to generic storage access

- `moira/src-tauri/src/store/write.rs`
  - target owner: `lachesis::store::write`
  - keep ingest transaction and projection refresh here, but keep it strictly Lachesis-owned

- `moira/src-tauri/src/model.rs`
  - split primarily into `lachesis::model`
  - later add module-local model files for `clotho` and `atropos`
  - current issue: app-wide shared model namespace will not scale once non-Lachesis modules land

## Ownership Rules

### `app`

- may know every top-level module handle
- may not encode domain logic
- should be the only layer that knows Tauri builder wiring details

### `clotho`

- may own artifact preparation and profile preparation together because both are pre-wake concerns
- must keep internal functional seams explicit enough that release/build logic and profile logic do not collapse into one blob
- may not own live process state or OTLP persistence

### `lachesis`

- may own DuckDB and the OTLP receiver
- may expose query DTOs tailored for Loom, but those DTOs remain Lachesis-owned, not app-owned
- may not become the home of wake state, selected profile, or artifact metadata

### `atropos`

- may depend on `clotho` and `lachesis` readiness interfaces
- may not call DuckDB tables directly as a state shortcut
- should own process handles and stop intent explicitly

## Command-Surface Principles

1. Commands are transport façades, not owners.
2. Each command delegates to one owning module or one module-local service.
3. Cross-module orchestration belongs in an owning module or a thin `app` composition helper, never in the command handler.
4. Cleanup keeps current observability command names and payload shapes stable unless a break is strictly required by correctness.
5. Future wake/build/profile commands should follow the same rule: command names reflect operator actions, while module boundaries reflect ownership.

## Persistence Principles

1. Lachesis remains the only module that writes DuckDB during cleanup.
2. Clotho and Atropos persistence must not be folded into `raw_events`, `runs`, `ticks`, or future Lachesis tables.
3. Module-owned local state should have explicit homes under the app data root, for example:
   - `telemetry/`
   - `artifacts/`
   - `profiles/`
   - `runtime/`
   - `cache/`
4. If multiple modules eventually share one durable database, ownership must still remain partitioned by explicit schema and migration boundaries.
5. Ephemeral runtime handles stay in module state, not in storage rows designed for operator browsing.

## Landing Sequence

1. Introduce `app`, `clotho`, `lachesis`, and `atropos` modules without changing operator behavior.
2. Replace `MoiraState` with an app-owned state container that holds top-level module handles instead of raw store and receiver handles.
3. Move the current OTLP receiver, normalization, DuckDB store, ingest pulse emission, and query APIs fully under `lachesis`.
4. Shrink `lib.rs` into bootstrap-only wiring and move command handlers under `app::commands`.
5. Add thin module skeletons for `clotho` and `atropos`, including their local-state roots and module-local service interfaces, without yet expanding user-facing features.
6. Validate that the current wake list, tick list, chronology, and raw inspection remain behavior-equivalent after the split.
7. Only then land the first cross-service feature slice.

## First Validating Feature Slice After Cleanup

The first validating slice should be:

- local source-folder build
- wake
- graceful stop

Reason:

- it forces Clotho, Atropos, and Lachesis to collaborate
- it tests the new split with one real operator workflow rather than with placeholder APIs
- it proves that the backend is no longer structurally blocked on Lachesis-only ownership

Expected backend path:

1. `clotho::artifacts` accepts a local Core source folder and produces a build artifact reference.
2. `clotho::profiles` resolves the selected JSONC profile reference.
3. `atropos` waits for `lachesis` receiver readiness, then wakes Core with the selected build and profile.
4. `lachesis` continues to ingest OTLP logs and expose the existing Loom surfaces.
5. `atropos` performs graceful stop and records terminal reason.

## Explicit Non-Goals For This Refactor

- no AI-family ownership redesign inside this cleanup doc
- no new observability contract redesign
- no large query-model expansion
- no frontend presentation rewrite hidden inside the backend cleanup
- no generic `manager` or `service locator` abstraction that recreates the current catch-all problem under a new name

## Immediate Follow-Through

After this target is accepted, the next useful task is to write the frontend cleanup target in the same style, anchored on the Unit TDD split:

- `bridge`
- `query state`
- `projection`
- `presentation`

That discussion should stay tied to the current backend target so the cleanup stage lands as one coherent architectural move instead of two unrelated refactors.
