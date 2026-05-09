# Apple Universal Verification

## Verification Scope

This file defines Apple Universal local verification contracts and evidence expectations.

Cross-unit contracts, system authority ownership, and decomposition policy live in Product TDD.

## Local Verification Contracts

1. Connection lifecycle contract:
- Manual connect/disconnect/retry behavior and socket-path apply-on-reconnect remain explicit and deterministic.
- Evidence homes: `apple-universal/BelunaApp/App/*` lifecycle logic and focused lifecycle tests.

2. Protocol compatibility contract:
- Request/response encoding/decoding and correlation metadata handling remain typed and explicit.
- Core-assigned runtime endpoint id behavior remains covered when endpoint-name handling changes.
- Evidence homes: `apple-universal/BelunaApp/BodyEndpoint/*`, protocol decode/encode tests.

3. Local history contract:
- Sense/act local persistence and bounded in-memory buffering behavior remain stable across relaunch and pagination flows.
- Evidence homes: `apple-universal/BelunaApp/App/LocalSenseActHistoryStore.swift`, `ChatViewModel` tests when behavior changes.

4. UI responsiveness contract:
- Network/protocol operations remain off main thread and interaction surfaces stay responsive.
- Evidence homes: runtime behavior checks and targeted unit/UI tests for reconnect and large-history scenarios.

5. Multi-instance contract:
- App launch must allow concurrent Apple Universal instances. Runtime resource conflicts surface through Core/Moira status.
- Evidence homes: app launch smoke checks, UI test launch checks, and future multi-instance endpoint registration tests.

6. Minimum Moira Loom contract:
- Settings-integrated operations panel shows embedded Moira runtime status, receiver status, and degraded/conflict/fault resources.
- Wake list, tick list, and selected tick raw-first inspection are available through Moira host APIs.
- Launch-target/profile read context is available inside the first Core context section.
- Acceptance follows the issue #30 minimum Apple contract and Apple-native host design.
- Evidence homes: `apple-universal/BelunaApp/App`, `apple-universal/BelunaApp/Moira`, `apple-universal/BelunaAppTests/MoiraRuntimeBindingTests.swift`, `apple-universal/BelunaAppTests/MoiraRuntimeBindingFixtures.swift`, Moira binding DTO tests, and targeted view-state tests.

7. Apple Moira panel split contract:
- Core Control owns Clotho/Atropos operation UI as a panel parallel to Settings.
- Core Control includes the first Clotho target/profile Create/Edit entry points around the same selected target/profile context used by wake.
- Target/profile editors use sheet-local drafts with `Cancel` and `Save` as their commit surface.
- Core Control Operations co-locates Core process state with wake, stop, and force-kill controls.
- Profile management covers `core_config`, environment file rows, and inline environment variable rows through Moira-owned draft APIs.
- O11y / Lachesis owns observability browsing and investigation UI as a panel parallel to Settings, with wake/tick navigation and raw event inspection in the first slice.
- Settings owns Moira configuration, receiver/socket defaults, refresh policy, diagnostics policy, and host-local preferences.
- Evidence homes: `MoiraCoreControlPanel`, `MoiraLaunchContextSection`, `MoiraTargetEditorSheet`, `MoiraProfileEditorSheet`, `MoiraO11yPanel`, `MoiraO11yViewModel`, `MoiraOperationsViewModel` lifecycle and Clotho management tests, O11y view-model tests, follow-on task packets, Apple navigation/view-model tests, and panel-level UI smoke checks.

8. Moira FFI packaging contract:
- macOS BelunaApp builds run the Moira FFI build script, bundle required Rust dylibs in `Contents/Frameworks`, and keep the app signature valid.
- Evidence homes: `apple-universal/script/build_moira_ffi.sh`, `apple-universal/BelunaApp.xcodeproj/project.pbxproj`, `codesign --verify`, `otool`, and `dynamicClientLoadsBundledMoiraFFI`, including the bundled `moira_runtime_loom_json`, lifecycle, profile document, profile draft, and known-local-build symbol paths.

9. Socket discovery contract:
- Configured path, recent successful path, app-local runtime candidate, deployment-supported platform candidates, and Moira-reported paths are available where supported.
- Evidence homes: socket discovery model tests and settings UI smoke checks.

## Expected Guardrails

1. Protocol shape changes require synchronized updates to Product TDD contract docs and endpoint unit docs.
2. Reconnect and pagination regressions must be validated in nominal and failure-oriented scenarios.
3. Local persistence format/logic changes must include restore compatibility checks.
4. Moira UI integration must keep body endpoint connection behavior covered by lifecycle checks.

## Escalation Rules

Escalate to Product TDD when a change affects:

1. Cross-unit protocol or identity semantics (`docs/20-product-tdd/cross-unit-contracts.md`).
2. Cross-unit authority ownership (`docs/20-product-tdd/system-state-and-authority.md`).
3. Unit boundary/container mapping decisions (`docs/20-product-tdd/unit-boundary-rules.md`, `docs/20-product-tdd/unit-to-container-mapping.md`).
