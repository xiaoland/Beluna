# Apple Universal Binding

This note captures the first Apple Universal host integration slice after `moira/runtime` extraction.

## Implemented Shape

New Swift namespace:

- `apple-universal/BelunaApp/Moira`

Files:

- `MoiraRuntimeModels.swift`
- `MoiraRuntimeClient.swift`
- `MoiraRuntimeDynamicClient.swift`
- `MoiraOperationsViewModel.swift`
- `apple-universal/script/build_moira_ffi.sh`

Settings integration:

- `BelunaAppApp` owns a `MoiraOperationsViewModel`.
- `SettingView` receives the Moira view model explicitly.
- `MoiraOperationsSection` shows runtime lifecycle, receiver state, Core phase, counts, and degraded/conflict/fault resources.

## Boundary Decision

Apple UI depends on `MoiraRuntimeClient`.

The current macOS default client attempts to load `libmoira_ffi.dylib` and resolve `moira_runtime_status_json`.

SwiftUI state, refresh behavior, and tests remain independent from ABI mechanics because the C ABI is hidden behind `DynamicMoiraRuntimeClient`.

The dynamic loader searches:

- `BELUNA_MOIRA_FFI_DYLIB`
- `Bundle.main.privateFrameworksURL/libmoira_ffi.dylib`
- repo-local `target/debug/libmoira_ffi.dylib`
- repo-local `target/release/libmoira_ffi.dylib`

Packaging integration:

- BelunaApp macOS builds run the `Build Moira FFI` Xcode script phase.
- Debug builds use Cargo's dev profile; Release builds use `--release`.
- iOS, iPadOS, and visionOS builds exit the script cleanly.
- The script bundles `libmoira_ffi.dylib` and DuckDB's `libduckdb.dylib` under `BelunaApp.app/Contents/Frameworks`.
- The script rewrites `libmoira_ffi.dylib`'s install name to `@rpath/libmoira_ffi.dylib`.
- Xcode signing signs both bundled dylibs when a signing identity is available.
- The BelunaApp target disables user script sandboxing at target scope for Cargo workspace access.

## Verification

Passed on 2026-05-07:

- `xcodebuild test -project apple-universal/BelunaApp.xcodeproj -scheme BelunaApp -destination 'platform=macOS' -only-testing:BelunaAppTests`
- `xcodebuild build -project apple-universal/BelunaApp.xcodeproj -scheme BelunaApp -destination 'platform=macOS'`

Passed on 2026-05-08:

- `cargo check -p moira-ffi --locked`
- `cargo test -p moira-ffi --locked`
- `cargo build -p moira-ffi --lib --locked`
- `bash -n apple-universal/script/build_moira_ffi.sh`
- `plutil -lint apple-universal/BelunaApp.xcodeproj/project.pbxproj`
- `xcodebuild build -project apple-universal/BelunaApp.xcodeproj -scheme BelunaApp -destination 'platform=macOS'`
- `codesign --verify --deep --strict /Users/lanzhijiang/Library/Developer/Xcode/DerivedData/BelunaApp-hbfvzmxvgxyigodcpjjrlfecfmtn/Build/Products/Debug/BelunaApp.app`
- `otool -D /Users/lanzhijiang/Library/Developer/Xcode/DerivedData/BelunaApp-hbfvzmxvgxyigodcpjjrlfecfmtn/Build/Products/Debug/BelunaApp.app/Contents/Frameworks/libmoira_ffi.dylib`
- `otool -L /Users/lanzhijiang/Library/Developer/Xcode/DerivedData/BelunaApp-hbfvzmxvgxyigodcpjjrlfecfmtn/Build/Products/Debug/BelunaApp.app/Contents/Frameworks/libmoira_ffi.dylib`
- `xcodebuild test -project apple-universal/BelunaApp.xcodeproj -scheme BelunaApp -destination 'platform=macOS' -only-testing:BelunaAppTests/BelunaAppTests`
- `xcodebuild test -project apple-universal/BelunaApp.xcodeproj -scheme BelunaApp -destination 'platform=macOS' -only-testing:BelunaAppTests/MoiraRuntimeBindingTests`
- `xcodebuild test -project apple-universal/BelunaApp.xcodeproj -scheme BelunaApp -destination 'platform=macOS' -only-testing:BelunaAppTests`

Bundle inspection on 2026-05-08:

- `libmoira_ffi.dylib`: bundled, signed, `@rpath/libmoira_ffi.dylib`, exports `moira_runtime_status_json`, `moira_runtime_shutdown_json`, and `moira_runtime_string_free`.
- `libduckdb.dylib`: bundled and signed for the `@rpath/libduckdb.dylib` dependency.
- `dynamicClientLoadsBundledMoiraFFI` proves the Xcode-built test host can load the bundled dylib and call real Moira runtime status.

Latest full Apple scheme attempt:

- `xcodebuild test -project apple-universal/BelunaApp.xcodeproj -scheme BelunaApp -destination 'platform=macOS'`
- Result: `BelunaAppTests` passed; `BelunaAppUITests-Runner` timed out while enabling automation mode.
- Result bundle: `/Users/lanzhijiang/Library/Developer/Xcode/DerivedData/BelunaApp-hbfvzmxvgxyigodcpjjrlfecfmtn/Logs/Test/Test-BelunaApp-2026.05.07_22-17-47-+0800.xcresult`

Focused test runner note:

- A second focused test run after final DTO/UI polish compiled targets, launched the test host, then stopped producing output.
- The orphaned DerivedData test host and xcodebuild process were terminated manually. The subsequent Apple app build passed.

Focused tests added:

- `moiraOperationsViewModelLoadsRuntimeSnapshot`
- `moiraOperationsViewModelKeepsSnapshotAndReportsRefreshError`
- `decodesMoiraRuntimeStatusJSON`
- `dynamicClientLoadsBundledMoiraFFI`

## Next Binding Step

Grow the host API beyond runtime status:

- Clotho launch target and profile read surfaces.
- Lachesis wake/tick list and raw record query surfaces.
- Atropos wake/stop operation surfaces.
- Future typed binding generation when the Loom API grows beyond the first status proof.
