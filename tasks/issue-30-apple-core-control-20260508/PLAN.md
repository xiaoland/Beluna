# Apple Core Control Plan

## MVT Core

- Objective & Hypothesis: add Apple-native Core lifecycle controls through the embedded Moira runtime so Apple Universal can operate the local Core process from a standalone Core Control panel parallel to Settings.
- Guardrails Touched: Core retains runtime behavior, config schema, endpoint protocol, and observability emission authority; Moira owns local preparation and supervision semantics; Apple Universal owns native UI and keeps Moira calls off the main thread.
- Verification: Moira runtime and FFI tests cover lifecycle operations; Apple tests cover DTO decoding and view-model state transitions; macOS build/test proves bundled FFI loading and UI integration.

## First Scope

1. Runtime and FFI surface
- Expose the minimum Atropos lifecycle operations needed by Apple Universal.
- Reuse existing Clotho launch-target and profile preparation semantics.
- Return JSON DTOs that preserve runtime status, resource status, terminal reason, and operation errors.

2. Apple binding
- Extend `DynamicMoiraRuntimeClient` with lifecycle methods.
- Decode operation results into typed Swift DTOs.
- Keep Rust calls on background execution paths.

3. Apple UI
- Add a first-class Core Control panel beside Settings in Apple Universal navigation.
- Keep Settings focused on Moira configuration such as runtime paths, receiver bind address, socket candidates, refresh policy, and diagnostics.
- Add launch-target/profile selection context sized for wake.
- Add wake controls using the currently selected launch target and optional profile.
- Add graceful stop control for supervised Core.
- Add force-kill behind a second confirmation path.
- Surface receiver/resource conflicts and terminal supervision state in the Core Control panel.

4. Tests
- Add Moira FFI tests for wake/stop/force-kill JSON operations where process fixtures are practical.
- Add Apple view-model tests for wake, stop, force-kill confirmation, error preservation, and refresh after operation.
- Keep existing body endpoint socket discovery behavior intact.

## Scope Boundaries

- Clotho mutation UI such as register, forge, install, and profile editing belongs to Apple Clotho Management.
- Rich Lachesis/Cortex inspection belongs to Native Lachesis/Cortex Inspection.
- Apple O11y / Lachesis owns wake/tick investigation depth, raw event inspection, Cortex timeline, and owner-specific drilldown.
- Owner/Attach authority coordination belongs to a future local Moira authority packet.
- Sandbox and ledger platform adapters belong to separate packets.
- CLI and Windows host coverage belongs to separate Human Interface host packets.

## Open Decisions

1. Whether the first wake UI uses only existing launch targets and profiles, or also accepts a raw executable path.
2. How Apple Universal should persist selected launch-target and profile refs.
3. Exact second-confirmation shape for force-kill on macOS.
4. How much terminal reason detail belongs in the Core Control panel's first viewport.
5. Whether lifecycle operation results should refresh through explicit polling first or introduce host event delivery in the same packet.
6. Which Apple Universal navigation surface should host the Core Control panel on macOS, iPadOS, and iOS.
