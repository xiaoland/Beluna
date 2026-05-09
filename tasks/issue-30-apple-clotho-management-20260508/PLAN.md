# Apple Clotho Management Plan

## MVT Core

- Objective & Hypothesis: extend Apple Core Control so operators can prepare and manage Core launch targets and profiles in the same view that wakes and supervises Core.
- Guardrails Touched: Core retains config schema authority and runtime behavior; Moira owns Clotho preparation and Atropos supervision semantics; Apple Universal owns native presentation and host-local UI state.
- Verification: Moira runtime/FFI tests cover target/profile mutation surfaces; Apple tests cover DTO decoding, view-model mutation state, and Core Control integration; macOS build/test proves bundled binding behavior.

## Product Shape

The Core Control view is organized around target plus profile.

1. Target:
- current selected launch target
- registered local builds
- forged local builds from a Beluna repo root or `core/` crate root
- installed release artifacts
- readiness, provenance, executable path, working directory, checksum state, and issue text

2. Profile:
- current selected profile
- app-local JSONC profile documents
- profile path
- create, edit, save, and delete affordances when the schema boundary is sufficiently explicit
- validation status that defers to Core schema authority where needed

3. Lifecycle:
- wake selected target with optional selected profile
- graceful stop and force-kill for the supervised process
- Atropos phase, PID, terminal reason, and receiver readiness

Clotho and Atropos belong in one Apple operator surface because the target/profile preparation context is the immediate input to wake/stop supervision.

## First Scope

1. Runtime and FFI surface
- Expose Clotho launch-target mutation APIs needed by Apple Universal: known local build registration, local forge, installed release discovery/install where practical.
- Expose profile document list/load/save APIs through the host binding.
- Keep operation payloads typed and JSON-backed for the current C ABI proof.

2. Apple binding
- Extend `MoiraRuntimeClient` with target/profile management methods.
- Decode target/profile operation results into existing Swift DTOs or narrow dedicated DTOs.
- Keep file access, JSON encoding, Rust calls, and profile document reads/writes off the main thread.

3. Apple UI
- Keep target/profile selection in `MoiraCoreControlPanel` Launch Context.
- Open focused target/profile editor sheets from Create/Edit actions.
- Use sheet-local drafts with `Cancel` and `Save` as the modal operation surface.
- Keep lifecycle buttons adjacent to the selected target/profile context.

4. Tests
- Add Moira FFI tests for profile document save/load and launch-target mutation paths.
- Add Apple binding/view-model tests for target/profile mutation state and selection preservation.
- Keep Core Control lifecycle tests intact.

## Current Execution Slice

Implemented first:

1. Rust FFI:
- `moira_runtime_load_profile_json`
- `moira_runtime_save_profile_json`
- `moira_runtime_load_profile_draft_json`
- `moira_runtime_save_profile_draft_json`
- `moira_runtime_register_known_local_build_json`

2. Apple binding:
- `MoiraRuntimeClient.loadProfileDocument`
- `MoiraRuntimeClient.saveProfileDocument`
- `MoiraRuntimeClient.loadProfileDraft`
- `MoiraRuntimeClient.saveProfileDraft`
- `MoiraRuntimeClient.registerKnownLocalBuild`
- Dynamic `libmoira_ffi.dylib` symbol resolution for the new Clotho functions.

3. Apple UI:
- `MoiraLaunchContextSection` inside Core Control with Create/Edit entry points.
- `MoiraTargetEditorSheet` for known-local-build create/edit.
- `MoiraProfileEditorSheet` for structured profile create/edit.
- Core process state integrated into the Core Control Operations section beside Atropos lifecycle controls.
- Sheet-local draft value types for target registration/update, `core_config` editing, env file rows, and inline environment rows.

4. Verification:
- `cargo test -p moira-runtime --locked`
- `cargo test -p moira-ffi --locked`
- `xcodebuild test -project apple-universal/BelunaApp.xcodeproj -scheme BelunaApp -destination 'platform=macOS' -only-testing:BelunaAppTests/MoiraRuntimeBindingTests`

## Scope Boundaries

- O11y / Lachesis browsing and raw event inspection stay out of this packet.
- Core config schema validation remains Core-owned; this packet may surface validation results when Moira has a typed query path.
- Release producer workflow stays outside this packet.
- Owner/Attach authority coordination stays outside this packet.
- Sandbox and ledger platform adapters stay outside this packet.

## Open Decisions

1. First Apple profile editing uses a structured sheet-local draft: raw `core_config` text, env file rows, and inline environment rows.
2. First target create/update uses known-local-build registration with an explicit executable path and stable `buildId`.
3. Local forge and release install remain follow-on Clotho preparation actions.
4. Selected target/profile refs remain session-local until an explicit persistence slice.
5. Profile validation feedback remains limited until Core schema validation has a typed host path.
