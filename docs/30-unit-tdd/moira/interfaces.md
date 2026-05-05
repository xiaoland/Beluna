# Moira Interfaces

## External Interface

1. Desktop application entrypoint:
- Tauri desktop app exposing Loom as the local operator UI.

2. Artifact preparation interface:
- GitHub Releases discovery for published Core artifacts.
- Trusted checksum file: `SHA256SUMS`.
- Trusted Core archive pattern: `beluna-core-<rust-target-triple>.tar.gz`.
- Current macOS-first expected asset: `beluna-core-aarch64-apple-darwin.tar.gz`.
- The published archive may contain executable `beluna`; archive basename and embedded executable basename are not required to match exactly.
- Local source-folder input accepts a Beluna repo root or `core/` crate root for explicit development forge before launch.
- App-local JSONC profile documents managed under Clotho-owned profile ids.

3. Lifecycle supervision interface:
- Wake local Core with a selected Clotho launch target and JSONC profile.
- Graceful stop for supervised Core.
- Explicit force-kill behind second confirmation.

4. Observability interface:
- Local OTLP gRPC logs receiver.
- Raw-event query and live-subscription interfaces for Loom.
- Minimum guaranteed log-backed Loom surfaces:
  - wake list
  - tick list
  - selected tick workspace
  - raw-first native event timeline anchored by selected tick and native `traceId`
  - raw event inspector as the source-grounded inspection surface, including native/legacy/ordinary `record_kind`
  - Cortex interval inspection when matching boundary records are reconstructable
  - AI transport, AI Chat, Stem, Spine, and goal-forest projections when their native owner events are available
- Metrics/traces exporter-status surfaces and handoff links only.

## Consumed Contract

1. Core typed config boundary remains the schema authority.
2. Core OTLP logs satisfy the cross-unit reconstruction rules defined in `docs/20-product-tdd/observability-contract.md` and the current owner scope / `eventName` surface described in `docs/30-unit-tdd/core/observability.md`.
3. Core startup/shutdown semantics remain Core-owned even when Moira supervises the process locally.
4. Legacy Core contract logs remain readable through Lachesis compatibility normalization during the migration period.

## Cleanup-Stage Constraint

1. The cleanup stage may restructure internal backend modules, state containers, and frontend layers without changing the current log-backed Loom surfaces or current Tauri observability command names.
2. New Clotho and Atropos command surfaces should land only after the internal backend split `app / clotho / lachesis / atropos` is established well enough that those features do not extend the current Lachesis-heavy modules as catch-all owners.
