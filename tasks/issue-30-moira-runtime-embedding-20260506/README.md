# Issue 30 Moira Runtime Embedding

Task packet for <https://github.com/xiaoland/Beluna/issues/30>.

This packet captures the proposed reshaping of Moira from a Tauri desktop app into an embeddable backend/runtime unit.

The execution scope for this issue is intentionally narrow: implement the minimum Moira Loom surface inside `apple-universal` only. Broader Human Interface host coverage, including `cli` and future `win-native`, remains design context and follow-on work.

## Files

- [PLAN.md](./PLAN.md): MVT anchors, scope, governing anchors, and open decisions.
- [BOUNDARY.md](./BOUNDARY.md): target unit/container boundary and role changes.
- [SEQUENCE.md](./SEQUENCE.md): proposed implementation slices and verification gates.
- [PACKAGING.md](./PACKAGING.md): internal package and embedding design questions.
- [SINGLETON.md](./SINGLETON.md): future single local Moira authority problem and current task's smaller runtime model.
- [APPLE-UNIVERSAL-LOOM.md](./APPLE-UNIVERSAL-LOOM.md): minimum Apple Universal Loom scope and UI integration questions.
- [APPLE-UNIVERSAL-UI-INTEGRATION.md](./APPLE-UNIVERSAL-UI-INTEGRATION.md): Apple Universal navigation, body endpoint, and Moira Loom integration design notes.
- [APPLE-UNIVERSAL-BINDING.md](./APPLE-UNIVERSAL-BINDING.md): first Swift Moira host binding seam and Settings integration.
- [APPLE-UNIVERSAL-CLEANUP.md](./APPLE-UNIVERSAL-CLEANUP.md): Apple Universal source cleanup scope before Moira UI integration.
- [RUNTIME-API-BOUNDARY.md](./RUNTIME-API-BOUNDARY.md): Slice 2A host-independent Moira runtime boundary.
- [RUNTIME-API-DTO-SKETCH.md](./RUNTIME-API-DTO-SKETCH.md): typed DTO sketch for the runtime host API.
- [RUNTIME-API-EXTRACTION-MAP.md](./RUNTIME-API-EXTRACTION-MAP.md): historical Tauri command/service extraction map.
- [RUNTIME-API-IMPLEMENTATION.md](./RUNTIME-API-IMPLEMENTATION.md): Slice 2B runtime crate implementation notes and verification.
- [RUNTIME-INTEGRATION-TESTS.md](./RUNTIME-INTEGRATION-TESTS.md): first critical integration tests for `moira/runtime`.
- [TAURI-LOOM-RETIREMENT.md](./TAURI-LOOM-RETIREMENT.md): Slice 6 retirement gate and disposition matrix for the legacy Tauri/Vue Loom.
- [OPEN-QUESTIONS.md](./OPEN-QUESTIONS.md): unresolved technical and product decisions.

## Current Packet Status

Mode: Slice 6 Tauri/Vue Loom retirement and deletion verified locally.

Recorded local verification:

- `git diff --check`
- `cargo metadata --locked --format-version 1`
- `cargo check --manifest-path moira/runtime/Cargo.toml --locked`
- `cargo test --manifest-path moira/runtime/Cargo.toml --locked`
- `cargo check -p moira-ffi --locked`
- `cargo test -p moira-ffi --locked`
- `bash -n apple-universal/script/build_moira_ffi.sh`
- `plutil -lint apple-universal/BelunaApp.xcodeproj/project.pbxproj`
- `xcodebuild test -project apple-universal/BelunaApp.xcodeproj -scheme BelunaApp -destination 'platform=macOS' -only-testing:BelunaAppTests`
- `xcodebuild build -project apple-universal/BelunaApp.xcodeproj -scheme BelunaApp -destination 'platform=macOS'`
- `otool -D /Users/lanzhijiang/Library/Developer/Xcode/DerivedData/BelunaApp-hbfvzmxvgxyigodcpjjrlfecfmtn/Build/Products/Debug/BelunaApp.app/Contents/Frameworks/libmoira_ffi.dylib`
- `otool -L /Users/lanzhijiang/Library/Developer/Xcode/DerivedData/BelunaApp-hbfvzmxvgxyigodcpjjrlfecfmtn/Build/Products/Debug/BelunaApp.app/Contents/Frameworks/libmoira_ffi.dylib`
- Result: passed on 2026-05-08.

Current local verification notes:

- Sandboxed `cargo test` runs cannot bind local TCP ports; elevated reruns passed for `moira/runtime` and `moira-ffi`.
- Standalone strict `codesign --verify --deep --strict` returned `CSSMERR_TP_NOT_TRUSTED` for the local Apple Development certificate; Xcode build/test still built, signed, ran tests, bundled the FFI dylibs, and loaded the bundled FFI in `dynamicClientLoadsBundledMoiraFFI`.

Latest Slice 5 proof:

- `MoiraRuntime::loom_snapshot(selection)` aggregates runtime status, launch targets, profiles, wakes, ticks, and selected tick raw records.
- `moira_runtime_loom_json` exposes that snapshot through the Apple FFI boundary.
- `MoiraOperationsSection` renders the minimum Settings-integrated Loom surface through `MoiraOperationsViewModel`.

Latest Slice 6 decision:

- Retirement readiness is contract-based against the issue #30 minimum Apple Universal Loom.
- Remaining useful Tauri/Vue ideas are assigned to follow-on packets.
- Legacy Tauri/Vue frontend/container code has been removed from the active Moira surface.
- Active workspace metadata, maintenance scripts, and durable docs now point at `moira/runtime`, `moira/ffi`, and Apple Universal host integration.

Latest bundle proof:

- `BelunaApp.app/Contents/Frameworks/libmoira_ffi.dylib` is bundled, signed, and has install name `@rpath/libmoira_ffi.dylib`.
- `BelunaApp.app/Contents/Frameworks/libduckdb.dylib` is bundled and signed for DuckDB runtime loading.
- `dynamicClientLoadsBundledMoiraFFI` opens the bundled FFI dylib, calls real `MoiraRuntime::loom_snapshot(selection)`, and shuts the FFI runtime down through `moira_runtime_shutdown_json`.

Latest full Apple scheme attempt:

- `xcodebuild test -project apple-universal/BelunaApp.xcodeproj -scheme BelunaApp -destination 'platform=macOS'`
- Result: `BelunaAppTests` passed; `BelunaAppUITests-Runner` exited with code 65 after timing out while enabling automation mode.
- Result bundle: `/Users/lanzhijiang/Library/Developer/Xcode/DerivedData/BelunaApp-hbfvzmxvgxyigodcpjjrlfecfmtn/Logs/Test/Test-BelunaApp-2026.05.07_22-17-47-+0800.xcresult`

Focused Apple test runner note:

- A second focused test run after final DTO/UI polish compiled targets, launched the test host, then stopped producing output.
- The orphaned DerivedData test host and xcodebuild process were terminated manually. The subsequent Apple app build passed.

Previous Apple Universal verification:

- `xcodebuild test -project apple-universal/BelunaApp.xcodeproj -scheme BelunaApp -destination 'platform=macOS' -only-testing:BelunaAppTests`
- Result: passed on 2026-05-07.
- Real app smoke test with Computer Use: app launched from the Xcode build product, main window rendered, Settings opened through the toolbar gear button, and Connection / Chat / Status / Moira sections rendered.

Previous full scheme note:

- `xcodebuild test -project apple-universal/BelunaApp.xcodeproj -scheme BelunaApp -destination 'platform=macOS'`
- Result: passed on 2026-05-07 after removing Apple Universal process singleton guarding.
- Follow-up decision: Apple Universal process singleton guarding was removed. Core/Spine-assigned runtime endpoint ids now carry the multi-instance disambiguation responsibility.

This packet is tactical. Durable truths should be promoted into Product TDD and affected Unit TDD docs after human confirmation.

## Follow-On Packets

- [Apple Core Control](../issue-30-apple-core-control-20260508/README.md): standalone Apple-native Core Control panel for wake, stop, force-kill, and terminal supervision state through embedded Moira.
- [Apple O11y / Lachesis](../issue-30-apple-o11y-lachesis-20260508/README.md): Apple-native observability and investigation panel for wake/tick browsing, raw event inspection, Cortex timeline, and owner-specific drilldown.
