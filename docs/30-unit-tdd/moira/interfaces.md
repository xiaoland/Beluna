# Moira Interfaces

## External Interface

1. Desktop application entrypoint:
- Tauri desktop app exposing Loom as the local operator UI.

2. Artifact preparation interface:
- GitHub Releases discovery for published Core artifacts.
- Trusted checksum file: `SHA256SUMS`.
- Trusted Core archive pattern: `beluna-core-<rust-target-triple>.tar.gz`.
- Current macOS-first expected asset: `beluna-core-aarch64-apple-darwin.tar.gz`.
- Local source-folder input for development builds compiled before launch.

3. Lifecycle supervision interface:
- Wake local Core with a selected artifact/build and JSONC profile.
- Graceful stop for supervised Core.
- Explicit force-kill behind second confirmation.

4. Observability interface:
- Local OTLP gRPC logs receiver.
- Raw-event query and live-subscription interfaces for Loom.
- Minimum guaranteed log-backed Loom surfaces:
  - wake list
  - tick list
  - selected tick workspace
  - human-friendly per-tick chronology anchored by `tick`, including interval work when reconstructable
  - expanded interval inspection for nested AI-capability activity when present
  - Cortex per-organ and goal-forest inspection
  - Stem state and pathway inspection
  - Spine adapter, endpoint, sense-ingress, and act-routing inspection
  - raw event inspector as the source-grounded inspection surface
- Metrics/traces exporter-status surfaces and handoff links only.

## Consumed Contract

1. Core typed config boundary remains the schema authority.
2. Core OTLP logs satisfy the cross-unit reconstruction rules defined in `docs/20-product-tdd/observability-contract.md` and the current exported family catalog described in `docs/30-unit-tdd/core/observability.md`.
3. Core startup/shutdown semantics remain Core-owned even when Moira supervises the process locally.
