# Moira Verification

## Behavioral Checks

1. Moira can register a known local build and use that selected build for the next wake through Loom.
2. Moira can create, edit, persist, and reselect multiple local JSONC profile documents through Loom.
3. Moira can wake the selected build with the selected profile, or wake without profile and omit `--config`.
4. Atropos exposes runtime status, graceful stop, explicit force-kill with second confirmation, and app-exit stop behavior.
5. Loom exposes separate `Lachesis`, `Atropos`, and `Clotho` stations without collapsing feature ownership back into one permanently stacked control page.

## Later Behavioral Slices

1. Moira can validate a selected JSONC profile against the Core schema authority before wake.
2. Moira can compile a local Core source folder for development wake flow.
3. Moira can verify a published Core artifact against `SHA256SUMS` before activation.

## Observability Checks

1. OTLP logs are ingested and persisted locally without requiring an external first-party log UI.
2. Loom can show wake-scoped inspection from local Moira state plus locally stored Core OTLP logs.
3. Loom can show a tick timeline from locally stored log data without relying on free-form raw payload parsing as the primary contract.
4. Loom can render one selected tick as a chronology using `tick` as the trace anchor and reconstructed interval work where Core boundary records allow pairing.
5. The selected-tick workspace defaults to the chronology view while preserving Cortex, Stem, Spine, and raw source-grounded inspection tabs as secondary inspections.
6. Loom can inspect nested AI transport activity from tick chronology or expanded Cortex intervals whenever the corresponding Core records exist, including provider, model, attempt or retry detail, token consumption, provider request or response payloads, and terminal errors when present.
7. Loom can inspect nested chat-capability activity from the same tick chronology or expanded Cortex intervals whenever the corresponding Core records exist, including thread snapshots, turn state, tool activity, thinking payload when present, and full chat payloads.
8. Loom can inspect one selected tick through non-empty Cortex, Stem, Spine, and raw-event surfaces whenever the corresponding Core events exist, including per-organ Cortex intervals, goal-forest snapshot or mutation history, Stem afferent/efferent activity, and Spine endpoint/sense/act records.
9. Goal-forest comparison is derived from selected ticks rather than loaded from a precomputed diff artifact.
10. Metrics/traces surfaces show exporter status and handoff links without claiming local signal ownership.

## Evidence Homes

1. Moira Clotho preparation and Atropos supervision logic in the `moira/` app container.
2. Core OTLP event-shape tests, [Core Observability](../core/observability.md), contract fixtures, and config validation guardrails.
3. Live end-to-end operator walkthroughs remain valid evidence for wake/stop and browse-surface checks while the current local read models and control-plane slices continue to stabilize.

## Current Architecture Checks

1. Current wake list, tick list, selected-tick chronology, and raw-event inspection remain operator-equivalent after the internal split and tabbed shell refactor.
2. Tauri command handlers delegate through `app` into explicit backend owners rather than accumulating ownership directly.
3. Loom root views no longer own live refresh wiring and selection-state orchestration directly; bridge transport does not own normalization or sorting.
4. Bridge contracts, normalized Loom-facing models, and query-owned UI state remain distinct layers rather than collapsing back into one shared frontend type bucket.
5. Lachesis persistence and Lachesis projections remain the owner of Lachesis state only; Clotho and Atropos state do not reuse Lachesis tables as a shortcut.
6. Clotho durable manifests and profile documents remain app-local preparation truth, while current selected build/profile refs remain query-owned session state until an explicit persistence slice lands.
7. Shared shell chrome such as feature tabs and dialog scaffolding remains reusable without becoming the owner of feature-specific preparation, supervision, or observability semantics.
