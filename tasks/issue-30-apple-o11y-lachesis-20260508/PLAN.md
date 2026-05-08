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

## Scope Boundaries

- Core Control owns wake, stop, force-kill, terminal reason, and process-state operation UI.
- Apple Clotho Management owns launch-target mutation, forge/install workflows, and profile editing.
- Host Event/Pulse API owns cross-host live update transport.
- Owner/Attach authority coordination belongs to a future local Moira authority packet.
- Sandbox and ledger platform adapters belong to separate packets.

## Open Decisions

1. Whether the first O11y panel starts from the existing `MoiraRuntime::loom_snapshot(selection)` or adds narrower Lachesis query FFI calls.
2. Which timeline grouping should appear first: raw chronological event list, Cortex owner-lane timeline, or tick-level summary lanes.
3. How much raw JSON should appear in the first viewport before requiring an inspector drilldown.
4. Which retired Loom ideas deserve product restatement before native implementation: narrative mode, Stem/Spine panels, AI transport, goal-forest comparison.
5. Whether event/pulse-driven refresh is a prerequisite for rich timeline interaction or a follow-on after the first native panel.
