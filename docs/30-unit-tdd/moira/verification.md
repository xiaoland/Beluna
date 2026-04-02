# Moira Verification

## Behavioral Checks

1. Moira can create, edit, persist, and reselect multiple local JSONC profile documents through Loom.
2. Moira can validate a selected JSONC profile against the Core schema authority before wake.
3. Moira can compile a local Core source folder for development wake flow.
4. Moira can verify a published Core artifact against `SHA256SUMS` before activation.
5. Quitting Moira stops the supervised Core.
6. Force-kill requires a second confirmation step and is not the default stop path.

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
3. Early evidence should prefer live end-to-end operator walkthroughs for ingest and inspection flows; broader automation is added only after the new read models and browsing surfaces stabilize.

## Cleanup Stage Exit Intent

1. Current wake list, tick list, selected-tick chronology, and raw-event inspection remain operator-equivalent after the internal split.
2. The backend cleanup slice is concrete in code: Tauri command handlers delegate through `app` into explicit backend owners rather than continuing to accumulate ownership directly.
3. The first frontend cleanup slice is concrete in code: root views no longer own live refresh wiring and selection-state orchestration directly, and bridge transport no longer owns normalization or sorting.
4. Lachesis persistence and Lachesis projections remain the owner of Lachesis state only; Clotho and Atropos state have prepared homes that do not require reusing Lachesis tables as a shortcut.
5. The second frontend cleanup slice is concrete in code: `normalize.ts` is replaced by explicit `projection/lachesis/*` owners, and presentation-facing helpers no longer mix source-grounded event interpretation with display-only formatting.
6. Current wake list, tick timeline, chronology popup inspection, sectional tick detail views, and raw-event drilldown remain operator-equivalent after the projection/presentation split.
7. The cleanup integration pass is concrete in code: bridge calls no longer expose only `unknown`, backend-shaped transport contracts are separated from normalized Loom-facing models, and the former shared frontend type bucket no longer acts as a cross-layer dumping ground.
