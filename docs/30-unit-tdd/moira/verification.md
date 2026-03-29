# Moira Verification

## Behavioral Checks

1. Moira can validate a selected JSONC profile against the Core schema authority before wake.
2. Moira can compile a local Core source folder for development wake flow.
3. Moira can verify a published Core artifact against `SHA256SUMS` before activation.
4. Quitting Moira stops the supervised Core.
5. Force-kill requires a second confirmation step and is not the default stop path.

## Observability Checks

1. OTLP logs are ingested and persisted locally without requiring an external first-party log UI.
2. Loom can show run-scoped inspection from local Moira state plus locally stored Core OTLP logs.
3. Loom can show a tick timeline from locally stored log data without relying on free-form raw payload parsing as the primary contract.
4. Loom can inspect one selected tick through Cortex, Stem, Spine, and raw-event drilldown surfaces.
5. Goal-forest comparison is derived from selected ticks rather than loaded from a precomputed diff artifact.
6. Dedicated Stem signal timeline and Spine topology pages, when added, are powered by the same local raw store and query boundary.
7. Metrics/traces surfaces show exporter status and handoff links without claiming local signal ownership.

## Evidence Homes

1. Moira artifact/build and supervision logic in the future `moira/` app container.
2. Core OTLP event-shape tests, [Core Observability](../core/observability.md), contract fixtures, and config validation guardrails.
3. Early MVP evidence should prefer live end-to-end operator walkthroughs for ingest and inspection flows; broader automation is added only after read models and surfaces stabilize.
