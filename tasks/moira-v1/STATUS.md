# Moira V1 Status

This note is the session handoff anchor for `tasks/moira-v1`.
It is procedural, not authoritative.

## Snapshot

- Date: `2026-04-02`
- Parent issue: `#21`
- Main active sub-issues:
  - `#22` Atropos v1: `In progress`
  - `#23` Clotho v1: `In progress`
  - `#24` Lachesis v1: `In progress`
  - `#25` live/replay ingest verification: `Backlog`

## What Is Landed

### Lachesis / `#24`

- `tasks/moira-v1/L2.md`, `L3.md`, `LACHESIS.md`, and `OPEN-QUESTIONS.md` now align to the current canonical Core observability family names.
- Historical draft vocabulary and historical fixture-id naming are explicitly marked as non-canonical.
- Current Lachesis browse surface remains the active Moira operator surface:
  - receiver status
  - wake list
  - tick timeline
  - selected tick detail

### Clotho / `#23`

- The first Clotho slice is now real:
  - `KnownLocalBuildRef`
  - optional `ProfileRef`
  - `PreparedWakeInput`
- Clotho now also owns the first operator-facing profile-document surface:
  - list saved profile documents
  - load one profile document by `profile_id`
  - save one JSONC profile document under `profiles/<profile-id>.jsonc`
- Backend contract is implemented in:
  - `moira/src-tauri/src/clotho/model.rs`
  - `moira/src-tauri/src/clotho/service.rs`
  - `moira/src-tauri/src/clotho/profiles.rs`
- Known local builds persist under:
  - `artifacts/known-local-builds/<build-id>.json`
- Profile documents persist under:
  - `profiles/<profile-id>.jsonc`
- The first-slice boundary is now end-to-end, not just backend-internal:
  - frontend can register a known local build
  - frontend can create and edit multiple profile documents
  - frontend stores the selected build ref for wake
  - Atropos consumes the Clotho-prepared wake input

### Atropos / `#22`

- Backend minimal supervision is implemented:
  - `runtime_status`
  - `wake`
  - `stop`
- Current runtime phases:
  - `idle`
  - `waking`
  - `running`
  - `stopping`
  - `terminated`
- `wake` now:
  - checks Lachesis receiver readiness first
  - resolves `KnownLocalBuildRef` through Clotho
  - launches Core without directly owning build-path resolution
- `stop` currently uses unix `SIGTERM` for the graceful path.
- `force_kill` is now implemented as a distinct supervision path and exposed through a second confirmation dialog in Loom.
- `terminal_reason` is exposed in runtime status.
- App exit now requests graceful stop for the supervised Core in the Tauri exit hook.

### Frontend Control Plane

- Loom now exposes the first tabbed operator shell instead of piling control plus browse into one long page:
  - `Lachesis` tab for wake list, tick timeline, and selected tick drilldown
  - `Atropos` tab for wake / stop / force-kill plus runtime inspection
  - `Clotho` tab for dual-column build registration and profile library management
- Dialog-backed operators are now present for the high-density editing paths:
  - register or replace known local build
  - create and edit multiple local profile documents
  - second confirmation before `force_kill`
- Current control-plane capability set:
  - select one saved profile id or wake without profile
  - wake the selected build from Atropos using current Clotho selection
  - graceful stop
  - refresh / polling runtime status
- Data flow follows the intended split:
  - `bridge -> query -> projection -> presentation`
- New frontend anchors:
  - `moira/src/bridge/{clotho,atropos}.ts`
  - `moira/src/query/loom/navigation.ts`
  - `moira/src/query/atropos/runtime.ts`
  - `moira/src/query/clotho/{builds,profiles}.ts`
  - `moira/src/presentation/loom/chrome/{StatusHeader,LoomFeatureTabs,LoomDialogShell}.vue`
  - `moira/src/presentation/atropos/runtime/*`
  - `moira/src/presentation/clotho/{dialogs,workshop}/*`
  - `moira/src/presentation/lachesis/workspace/LachesisWorkspacePanel.vue`

## Verified In This Turn

- `cargo check --quiet` in `moira/src-tauri`
- `cargo check --tests --quiet` in `moira/src-tauri`
- `cargo test clotho:: --quiet` in `moira/src-tauri`
- `pnpm -C moira build`
- Operator-reported live desktop-shell walkthrough on `2026-04-02`:
  1. register known local build
  2. wake Core from Loom
  3. confirm Lachesis received a new wake plus related OTLP logs
  4. stop the supervised Core successfully

## Not Yet Verified

- A fully written evidence note for the operator-reported live walkthrough.
- An explicit operator walkthrough of the full Lachesis browse surface from Moira itself:
  1. inspect wake list
  2. inspect tick timeline
  3. inspect one selected tick through chronology / cortex / stem / spine / raw
- App-exit stop behavior against a real supervised Core process.

## Important Current Constraints

- `Atropos` currently uses polling for frontend runtime refresh; it does not emit a dedicated supervision event yet.
- `stop()` graceful behavior is unix-only in the current slice.
- Frontend selected build ref is query-state only for now; the durable build manifest lives in app-local storage, but the UI selection itself is not yet persisted.
- frontend selected profile ref is query-state only for now; the durable profile document lives in app-local storage, but the wake selection itself is not yet persisted.
- `profile_id` still maps to `profiles/<profile-id>.jsonc`; if omitted, Atropos wakes Core without `--config`.

## Suggested Next Step

Use the clearer operator shell to close the remaining `#21` evidence gaps:

1. validate the new Clotho profile-document workflow against a real Core config
2. perform one explicit browse-surface walkthrough for selected tick detail (`chronology / cortex / stem / spine / raw`)
3. verify app-exit stop behavior against a real supervised Core process
4. only after those acceptance notes land, negotiate whether the next quality slice should be supervision events or persistence for selected build/profile refs

## Handoff File Anchors

- control-plane backend:
  - `moira/src-tauri/src/app/bootstrap.rs`
  - `moira/src-tauri/src/app/commands/{clotho,atropos}.rs`
  - `moira/src-tauri/src/atropos/{model,service}.rs`
  - `moira/src-tauri/src/clotho/{model,service,profiles}.rs`
- control-plane frontend:
  - `moira/src/app/LoomApp.vue`
  - `moira/src/bridge/contracts/{clotho,atropos}.ts`
  - `moira/src/bridge/{clotho,atropos}.ts`
  - `moira/src/query/loom/navigation.ts`
  - `moira/src/query/atropos/runtime.ts`
  - `moira/src/query/clotho/{builds,profiles}.ts`
  - `moira/src/projection/clotho/*`
  - `moira/src/presentation/loom/chrome/{StatusHeader,LoomFeatureTabs,LoomDialogShell}.vue`
  - `moira/src/presentation/atropos/runtime/*`
  - `moira/src/presentation/clotho/{dialogs,workshop}/*`
  - `moira/src/presentation/lachesis/workspace/LachesisWorkspacePanel.vue`
- task-buffer context:
  - `tasks/moira-v1/L2.md`
  - `tasks/moira-v1/L3.md`
  - `tasks/moira-v1/LACHESIS-WALKTHROUGH-STATUS.md`
