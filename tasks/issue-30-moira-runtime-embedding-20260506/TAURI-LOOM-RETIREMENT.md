# Tauri/Vue Loom Retirement

This note captures the Slice 6 retirement gate and deletion outcome for the legacy Tauri/Vue Loom.

## Gate Standard

Retirement readiness is contract-based.

The gate is satisfied when Apple Universal covers the issue #30 minimum operator workflow and durable docs identify follow-on owners for any useful ideas left in the Tauri/Vue implementation.

The Tauri/Vue implementation is historical evidence. It is a source for inventory and cautionary examples. The Apple Universal product contract comes from issue #30 scope and durable docs.

## Retirement Baseline

Apple Universal now covers the selected issue #30 minimum surface:

- embedded Moira runtime status
- Lachesis receiver status
- launch-target/profile read context
- wake list
- tick list
- selected tick raw-first inspection
- bundled FFI proof through `moira_runtime_loom_json`

Before deletion, the legacy Tauri/Vue Loom contained broader operation surfaces:

- Clotho launch-target mutation: register local build, forge local build, install published release
- Clotho profile mutation: create/edit wrapper profile documents, env files, inline environment
- Atropos supervision actions: wake, graceful stop, force kill with second confirmation
- Lachesis richer browsing: Cortex timeline/narrative mode, Stem/Spine sectional panels, raw event inspector
- Vue shell composition: feature tabs, status header, dialogs, layout, and visual styling

## Retirement Matrix

| Legacy surface | Current Apple status | Disposition | Follow-on owner |
| --- | --- | --- | --- |
| Runtime/receiver status | Covered by Settings-integrated Moira panel | Retire Tauri surface | Apple Universal minimum Loom |
| Wake list | Covered by `MoiraLoomSnapshot.runs` | Retire Tauri surface | Apple Universal minimum Loom |
| Tick list | Covered by `MoiraLoomSnapshot.ticks` | Retire Tauri surface | Apple Universal minimum Loom |
| Selected tick raw records | Covered by `MoiraLoomSnapshot.tickDetail.raw` and `MoiraEventRecordRow` | Retire Tauri surface | Apple Universal minimum Loom |
| Launch-target/profile read context | Covered as read-only Core context | Retire Tauri read surface | Apple Universal minimum Loom |
| Clotho target registration/forge/install UI | Backend exists in `moira/runtime`; Apple UI intentionally absent from issue #30 | Redesign later | Full Apple-native Loom or a Clotho-specific follow-on |
| Clotho profile editor UI | Backend exists in `moira/runtime`; Apple UI intentionally absent from issue #30 | Redesign later | Full Apple-native Loom or a profile-management follow-on |
| Atropos wake/stop/force-kill UI | Backend exists in `moira/runtime`; Apple UI intentionally absent from Slice 5 | Redesign later | Apple Core Control follow-on |
| Cortex chronology timeline | Useful reconstruction idea; current Vue presentation is legacy-specific | Redesign later | Native Lachesis/Cortex inspection follow-on |
| Cortex narrative mode | Useful investigation idea; current shape requires product review | Redesign later | Native Lachesis/Cortex inspection follow-on |
| Stem/Spine sectional panels | Useful owner-specific inspection idea; current shape exceeds issue #30 | Redesign later | Native owner-inspection follow-on |
| Goal-forest and AI transport inspection | Valuable raw-first investigation direction; current projections need contract review | Redesign later | Observability/Loom projection follow-on |
| Live update wiring through Tauri events | Replaced for this slice by explicit snapshot refresh | Redesign later | Moira host event/pulse API follow-on |
| Vue feature tabs, dialogs, and layout | Apple Universal has its own Settings-integrated shape | Delete | Tauri/Vue retirement slice |
| Tauri command facades | Runtime behavior has moved to `moira/runtime` | Delete | Tauri/Vue retirement slice |

## Slice 6 Decision And Outcome

The issue #30 minimum retirement gate is satisfied, and deletion is implemented.

The remaining Tauri/Vue surfaces fell into two groups:

1. Deleted with the Tauri/Vue container:
- Vue shell, presentation components, query state, projection helpers, bridge contracts, and Tauri invoke wiring that served the legacy Loom.
- Tauri app bootstrap, command registration, event sink, capabilities, and packaging files.

2. Promote into follow-on packets:
- Apple Core Control: wake, graceful stop, force kill, terminal reason, and process state.
- Apple Clotho Management: register/forge/install launch targets and profile editing.
- Native Lachesis Inspection: timeline, narrative, owner-specific panels, and richer raw drilldown.
- Host Event/Pulse API: live updates for Apple and future Human Interface hosts.

## Deletion Slice Guardrails

The deletion slice should keep these invariants:

- `moira/runtime` remains the backend owner for Clotho, Lachesis, and Atropos.
- `moira/ffi` remains available for Apple Universal until a stronger binding replaces it.
- Apple Universal build/test/packaging remains green.
- Rust runtime integration tests remain green.
- Follow-on gaps are listed by owner before deleting the only legacy UI entrypoint.

## Implemented Deletion Scope

Deleted files and directories:

- `moira/src`
- `moira/src-tauri`
- frontend package, Vite, TypeScript, and Tauri build metadata that exists only for the legacy desktop shell
- task/docs references that described Tauri/Vue as an active migration container

Workspace metadata, maintenance scripts, and active durable docs now point at `moira/runtime`, `moira/ffi`, and Apple Universal host integration.

## Verification For Deletion Slice

Minimum verification:

- `git diff --check`
- `cargo metadata --locked --format-version 1`
- `cargo test --manifest-path moira/runtime/Cargo.toml --locked`
- `cargo check -p moira-ffi --locked`
- `cargo test -p moira-ffi --locked`
- `xcodebuild test -project apple-universal/BelunaApp.xcodeproj -scheme BelunaApp -destination 'platform=macOS' -only-testing:BelunaAppTests`
- `xcodebuild build -project apple-universal/BelunaApp.xcodeproj -scheme BelunaApp -destination 'platform=macOS'`
- repository search confirms no stale Tauri/Vue build entrypoint remains in active docs or scripts

Latest result: passed on 2026-05-08. Runtime and FFI tests need sandbox-external execution when they bind local TCP ports.
