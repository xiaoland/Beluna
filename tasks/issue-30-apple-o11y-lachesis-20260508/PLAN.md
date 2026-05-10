# Apple O11y / Lachesis Plan

## MVT Core

- Objective & Hypothesis: add an Apple-native O11y / Lachesis panel so operators can browse, inspect, and investigate Core observability data from the embedded Moira runtime.
- Guardrails Touched: Core retains observability emission semantics; Moira owns ingestion, storage, query, and projection; Apple Universal owns native investigation UI and keeps Moira calls off the main thread.
- Verification: Moira runtime/FFI tests cover the needed query DTOs; Apple tests cover DTO decoding, selection state, raw-event rendering state, and refresh behavior; macOS build/test proves bundled FFI loading and panel integration.

## First Scope

1. Runtime and FFI surface
- Extend the minimum Loom snapshot or add focused query functions for wake/tick browsing and selected tick detail.
- Preserve raw OTLP records as the source-grounded inspection fallback.
- Expose enough metadata for native timeline grouping, record-kind display, and owner-lane navigation.

2. Apple binding
- Extend `DynamicMoiraRuntimeClient` with Lachesis query methods.
- Decode raw event and timeline DTOs into typed Swift models.
- Keep large payload parsing on background execution paths.

3. Apple UI
- Add an O11y / Lachesis panel beside Core Control and Settings.
- Show wake list, tick list, and selected tick context as durable navigation state.
- Add raw-first selected tick inspection and a native raw event inspector.
- Add first native Cortex chronology or timeline view when DTO evidence is sufficient.
- Keep owner-specific drilldown behind follow-on UI sections until each projection has a clear contract.

4. Tests
- Add Moira FFI tests for selected query DTOs.
- Add Apple view-model tests for wake/tick selection, raw event inspection state, empty states, and refresh errors.
- Add focused UI state tests where SwiftUI rendering boundaries allow stable assertions.

## Slice 1 Record

Decision:

- Start the Apple O11y panel from the existing `MoiraRuntime::loom_snapshot(selection)` / `MoiraLoomSnapshot` binding.
- Add narrower Lachesis query FFI calls when panel interactions or payload size require a separate contract.

Implemented:

- `MoiraO11yViewModel` owns wake/tick/raw event selection, refresh state, selected raw-event state, and refresh error text.
- `MoiraO11yPanel` adds a standalone Apple-native window parallel to Core Control and Settings.
- The first detail surface shows selected tick summary, raw event list, and a source-grounded raw JSON inspector.
- The main Beluna window exposes an O11y / Lachesis entry point.

Verified:

- `xcodebuild test -project apple-universal/BelunaApp.xcodeproj -scheme BelunaApp -destination 'platform=macOS' -derivedDataPath apple-universal/.derived-data -only-testing:BelunaAppTests/MoiraRuntimeBindingTests CODE_SIGNING_ALLOWED=NO CODE_SIGNING_REQUIRED=NO`
- Real-app Computer Use smoke opened `O11y / Lachesis` from the Beluna main window and confirmed wake list, tick list, raw event list, and raw inspector render from the embedded Moira runtime.

## Slice 2 Record

Decision:

- Add the first Tick Gantt view as an Apple-side raw-derived visualization parallel to Raw view.
- Keep interval pairing and richer owner projections behind later Moira query contracts.

Implemented:

- `MoiraTickGanttSnapshot` derives owner lanes and event positions from selected tick raw records.
- `MoiraTickGanttSnapshot` pairs recognized lifecycle records such as `started`/`finished`, `*.started`/`*.finished`, and `dispatched`/`committed` into interval items.
- `MoiraTickGanttView` renders selected tick source records as lane-based event markers and interval blocks on a relative timeline.
- Gantt selection shows a bottom detail pane with the selected point or interval source records.
- `MoiraO11yPanel` switches selected tick detail between `Gantt` and `Raw`.
- `MoiraO11yViewModel` owns the selected detail mode while preserving raw-event selection.

Verification target:

- `MoiraRuntimeBindingTests` covers Gantt lane derivation, lifecycle interval pairing, timeline positions, and Gantt/Raw detail mode switching.

## Scope Boundaries

- Core Control owns wake, stop, force-kill, terminal reason, and process-state operation UI.
- Apple Clotho Management owns launch-target mutation, forge/install workflows, and profile editing.
- Host Event/Pulse API owns cross-host live update transport.
- Owner/Attach authority coordination belongs to a future local Moira authority packet.
- Sandbox and ledger platform adapters belong to separate packets.

## Open Decisions

1. How much owner-specific interval reconstruction belongs in Apple raw-derived UI before Moira exposes richer projections.
2. How much raw JSON should appear in the first viewport before requiring deeper inspector modes.
3. Which retired Loom ideas deserve product restatement before native implementation: narrative mode, Stem/Spine panels, AI transport, goal-forest comparison.
4. Whether event/pulse-driven refresh is a prerequisite for rich timeline interaction or a follow-on after the first native panel.
