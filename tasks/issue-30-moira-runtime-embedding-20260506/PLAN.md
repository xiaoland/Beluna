# Plan

## MVT Core

- Objective & Hypothesis: turn Moira into an embeddable backend/runtime unit and prove the direction with a minimum Moira Loom surface inside `apple-universal`.
- Guardrails Touched: Core remains runtime/config/observability-emission authority; Moira remains the local control-plane, supervision, artifact/profile preparation, and observability consumer authority inside the embedding host; Apple Universal remains a platform-native Human Interface host with explicit endpoint protocol compatibility.
- Verification: durable docs update the unit/container mapping and affected unit boundaries; Apple Universal source boundaries are cleaned enough to host the operations panel; Moira backend exposes a host-independent runtime API sufficient for the Apple host; `apple-universal` reaches the minimum Moira Loom workflows through its embedded Moira runtime and preserves body-endpoint socket use when Core is already available.

## Exploration Scaffold

- Perturbation: Issue #30 and product concern that installing a separate Moira app beside `apple-universal` or `win-native` creates a poor operator experience.
- Input Type: Intent.
- Active Mode or Transition Note: Solidify before Execute. The next step is architectural restatement and durable-doc alignment.
- Governing Anchors:
  - `docs/20-product-tdd/unit-topology.md`
  - `docs/20-product-tdd/unit-to-container-mapping.md`
  - `docs/20-product-tdd/cross-unit-contracts.md`
  - `docs/30-unit-tdd/moira/*`
  - `docs/30-unit-tdd/apple-universal/*`
  - `moira/AGENTS.md`
  - `apple-universal/AGENTS.md`
- Impact Hypothesis: the best long-term shape is Moira as a library-first runtime plus platform adapters, with the first proof landing inside Apple Universal.
- Temporary Assumptions:
  - `apple-universal`, `cli`, and future `win-native` should be described as Beluna Human Interface clients.
  - Each Human Interface client may implement its own Loom-equivalent operator UI.
  - The legacy Tauri/Vue Loom can retire after Apple Universal covers the minimum operator path.
  - This task implements the Apple Universal path only.
  - CLI and Windows embedding remain follow-on design targets.
  - This task uses process-local Moira backend inside Apple Universal.
  - Attach mode and cross-client authority coordination belong to a later task packet.
  - Multiple Human Interface processes may each include Moira backend code during this task.
  - Resource conflicts such as OTLP receiver bind conflicts are surfaced in UI as runtime status.
  - Body endpoint socket discovery remains useful when Core came from another process or prior session.
  - Apple Universal source cleanup is part of this task because the Settings-integrated panel will otherwise deepen existing mixed responsibilities.
  - Sandbox and ledger supervision belong to Moira backend design, with platform adapters selected by target OS and build features.
  - Sandbox and ledger implementation belong to later task packets.
- Negotiation Triggers:
  - A host integration starts to copy Core domain behavior into a Human Interface client.
  - Moira runtime API shape forces Core config schema authority into Moira.
  - Runtime resource conflicts become ambiguous in the Apple UI.
  - Platform adapter boundaries become unclear enough to block shared runtime testing.
  - A deletion slice would remove the working operator path before the Apple host is ready.
- Promotion Candidates:
  - Product TDD update: unit/container mapping separates Moira as a technical unit from its former Tauri container.
  - Product TDD update: endpoint clients become Beluna Human Interface clients where they host human-facing operation.
  - Moira Unit TDD update: backend/runtime API, host adapter boundary, and platform adapter boundary.
  - Apple Universal Unit TDD update: dual role as endpoint UX and Moira host surface, plus internal source ownership boundaries.
  - Task follow-on split: CLI host, Windows host, full native Loom, sandbox, and ledger work.

## Execution Notes

- key findings:
  - Current Moira backend already has explicit `clotho`, `lachesis`, and `atropos` service boundaries.
  - Legacy Tauri coupling was concentrated in app bootstrap, command facades, event emission, and a small amount of task spawning.
  - The existing crate already declares `staticlib`, `cdylib`, and `rlib` outputs, which supports a library-first direction.
  - Apple Universal currently concentrates endpoint lifecycle, reconnect policy, message paging, persistence, and settings draft state in `ChatViewModel`.
  - `SettingView` is the accepted first-slice container for Moira Loom, so it needs section-level extraction before adding operations UI.
- decisions made:
  - Apple Universal Moira Loom starts as a Settings-integrated operations panel.
  - Target architecture removes the Tauri/Vue Loom surface after Apple Universal provides the first minimum native Loom path.
  - This issue's implementation target is limited to Apple Universal.
  - Embedded Moira means each host can include the package; this task proves the Apple process-local path first.
  - Moira backend remains the owner of future sandbox and ledger supervision, mediated by platform adapters.
- final outcome:
  - Slice 0 promoted the architecture restatement into Product TDD, Moira Unit TDD, Apple Universal Unit TDD, and local AGENTS files.
  - Slice 1 prepared Apple Universal Settings boundaries and removed process singleton guarding.
  - Slice 2A recorded the Moira runtime API boundary, DTO sketch, and extraction map as task-packet artifacts.
  - Slice 2B introduced `moira/runtime`, moved Clotho/Lachesis/Atropos backend owners into it, and temporarily used the legacy desktop shell as an adapter over `MoiraRuntime`.
  - Slice 2C added public-boundary integration tests for runtime open/status, receiver conflict, Clotho wake preparation, Lachesis OTLP ingest, and Atropos process supervision.
  - Apple Universal host integration added a Swift Moira namespace, `MoiraRuntimeClient`, `MoiraOperationsViewModel`, Settings integration, and focused view-model tests.
  - Apple Universal Rust adapter proof added `moira/ffi`, dynamic Swift loading for `libmoira_ffi.dylib`, and DTO decoding tests for `MoiraRuntime.status()` JSON.
  - Slice 5 added the minimum Apple-native Loom read/query surface: runtime/receiver status, launch-target/profile context, wake list, tick list, and selected tick raw records through `MoiraRuntime::loom_snapshot(selection)` and `moira_runtime_loom_json`.
  - Slice 6 reframed Tauri/Vue retirement around the issue #30 Apple Universal minimum Loom contract, with legacy-only Clotho/Atropos/Lachesis ideas assigned to follow-on packets.
  - Slice 6 deletion removed the legacy Tauri/Vue code surface and left `moira/runtime` plus `moira/ffi` as Moira's active Rust packages.
