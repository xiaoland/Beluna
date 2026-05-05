# Moira Verification

## Behavioral Checks

1. Moira can register a known local build and use that selected launch target for the next wake through Loom.
2. Moira can create, edit, persist, and reselect multiple local JSONC profile documents through Loom.
3. Moira can wake the selected launch target with the selected profile, or wake without profile and omit `--config`.
4. Atropos exposes runtime status, graceful stop, explicit force-kill with second confirmation, and app-exit stop behavior.
5. Loom exposes separate `Lachesis`, `Atropos`, and `Clotho` stations without collapsing feature ownership back into one permanently stacked control page.

## Current Clotho Follow-On Checks

1. Moira can forge a local launch target from a Beluna repo root or `core/` crate root before wake.
2. Moira can discover a published Core release for the current supported target and verify it against `SHA256SUMS` before activation.
3. Moira can install the verified release into a version-isolated local directory and expose it as a launch target.
4. Wrapper profile documents can package `core_config`, `env_files`, and inline environment variables through Loom.
5. Moira still defers selected JSONC profile validation against the Core schema authority to a later slice.

## Rust Build Checks

1. Local Moira backend verification passes with `cargo check --manifest-path moira/src-tauri/Cargo.toml --locked`.
2. DuckDB bundled verification is explicit through `cargo check --manifest-path moira/src-tauri/Cargo.toml --locked --features duckdb-bundled`.
3. Cargo metadata for the default Moira feature set resolves DuckDB without the `bundled` feature.
4. Cargo metadata with `--features duckdb-bundled` resolves `duckdb/bundled` and `libduckdb-sys/bundled`.

## Observability Checks

1. OTLP logs are ingested and persisted locally without requiring an external first-party log UI.
2. Loom can show wake-scoped inspection from local Moira state plus locally stored Core OTLP logs.
3. Loom can show a tick timeline from locally stored native OTLP log data without relying on free-form raw payload parsing as the primary contract.
4. Loom can render one Cortex-handled tick as a raw-first native event timeline using native `traceId` as the machine grouping key.
5. The selected-tick workspace exposes native chronology and raw source-grounded inspection before deeper owner-specific reconstruction.
6. Cortex View recognizes ticks with native Cortex owner evidence without erasing broader tick inspection from Lachesis.
7. Raw inspection surfaces `record_kind` so operators can distinguish native owner records, legacy contract records, and ordinary logs during migration.
8. Legacy contract payloads remain readable through compatibility normalization while native records keep scope/event/trace/span identity.
9. Loom can inspect AI transport, AI Chat, Stem, Spine, and goal-forest data through raw records first and targeted projections as those projections mature.
10. Goal-forest comparison is derived from selected ticks rather than loaded from a precomputed diff artifact.
11. Metrics/traces surfaces show exporter status and handoff links without claiming local signal ownership.

## Evidence Homes

1. Moira Clotho preparation and Atropos supervision logic in the `moira/` app container.
2. Core OTLP event-shape tests, [Core Observability](../core/observability.md), contract fixtures, and config validation guardrails.
3. Live end-to-end operator walkthroughs remain valid evidence for wake/stop and browse-surface checks while the current local read models and control-plane slices continue to stabilize.

## Current Architecture Checks

1. Current wake list, broader tick list, Cortex timeline mode, and raw-event inspection remain operator-equivalent after the Cortex View reshape.
2. Tauri command handlers delegate through `app` into explicit backend owners rather than accumulating ownership directly.
3. Loom root views no longer own live refresh wiring and selection-state orchestration directly; bridge transport does not own normalization or sorting.
4. Bridge contracts, normalized Loom-facing models, and query-owned UI state remain distinct layers rather than collapsing back into one shared frontend type bucket.
5. Lachesis persistence and Lachesis projections remain the owner of Lachesis state only; Clotho and Atropos state do not reuse Lachesis tables as a shortcut.
6. Clotho durable manifests and profile documents remain app-local preparation truth, while current selected launch-target/profile refs remain query-owned session state until an explicit persistence slice lands.
7. Shared shell chrome such as feature tabs and dialog scaffolding remains reusable without becoming the owner of feature-specific preparation, supervision, or observability semantics.
