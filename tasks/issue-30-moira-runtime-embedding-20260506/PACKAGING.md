# Packaging And Embedding

This file captures technical design questions for making Moira backend an internal package embedded by host clients.

## Goal For This Task

Apple Universal builds with Moira backend as an internal dependency and calls a narrow host-facing API needed by the minimum Loom surface.

The package may be present in multiple processes. This task proves the Apple process-local embedding path and surfaces resource conflicts. Shared authority coordination belongs to a later task packet.

## Candidate Package Shapes

1. Rust workspace crate
- Moira backend is extracted into a Rust crate inside the repo.
- `cli` can later depend on it directly.
- Apple Universal needs a generated or hand-written Swift binding layer.

2. Swift Package wrapping Rust artifact
- Apple Universal consumes a local Swift Package.
- The package owns build scripts, headers, module maps, and compiled Rust artifacts.
- Rust remains the source of backend behavior.

3. UniFFI-style binding package
- Rust exports interface definitions and generated Swift types.
- Good fit for typed DTO-heavy command/query APIs.
- Requires deciding whether UniFFI build tooling is acceptable in the repo.

4. C ABI plus Swift wrapper
- Smaller build-tool surface.
- More manual memory, async, and DTO handling.
- Better for a tiny proof, weaker for a growing Loom API.

## Design Pressure

- Moira APIs are async and stateful.
- Lachesis query payloads can become large.
- Apple UI must keep runtime calls off the main actor.
- The first package boundary should support direct package-level tests.
- Build artifacts should remain local and reproducible.

## Package Boundary Questions

1. Should Moira runtime live under current `moira/src-tauri/src` during extraction, or move to a new workspace crate such as `moira/runtime`?
2. Should the Swift binding expose one coarse `MoiraRuntime` handle, or separate `Clotho`, `Lachesis`, and `Atropos` handles?
3. Should event delivery use callbacks, async streams, polling, or a mixed model?
4. Should Apple Universal own app-data path selection, with Moira only receiving explicit `MoiraPaths`?
5. Should the Apple build invoke Cargo directly, consume a prebuilt artifact, or use a checked-in local package script?
6. What is the minimum binding technology that still scales beyond receiver status and tick browsing?
7. Which resource conflicts must appear in the first Swift binding DTOs?
8. Which socket discovery candidates should Apple Universal present for Core body endpoint use?

## Current Working Bias

- Extract a Rust runtime crate first.
- Give hosts explicit path configuration.
- Prefer typed DTO APIs over JSON strings at the host boundary where practical.
- Keep JSON payload fields available for raw OTLP inspection.
- Use the smallest binding approach that preserves async correctness and memory safety.
- Keep future authority coordination below host UI so every Human Interface client can share the same semantics later.

## Current Proof

The first Apple proof uses C ABI plus a Swift dynamic-loader wrapper and macOS Xcode packaging:

- Rust crate: `moira/ffi`
- Cargo artifacts: `target/debug/libmoira_ffi.dylib` and `target/debug/deps/libduckdb.dylib`
- Xcode script: `apple-universal/script/build_moira_ffi.sh`
- Bundled artifacts: `BelunaApp.app/Contents/Frameworks/libmoira_ffi.dylib` and `BelunaApp.app/Contents/Frameworks/libduckdb.dylib`
- ABI functions: `moira_runtime_status_json`, `moira_runtime_shutdown_json`, and `moira_runtime_string_free`
- Swift adapter: `DynamicMoiraRuntimeClient`

This proves that Apple Universal can call `MoiraRuntime.status()` through the host seam, decode the result into `MoiraRuntimeSnapshot`, and load the Rust runtime from the app bundle in Xcode-built macOS products.

The current development path loads either `BELUNA_MOIRA_FFI_DYLIB`, a bundled private framework copy, or the repo-local Cargo artifact. The macOS app target invokes Cargo directly, copies both runtime dylibs into `Contents/Frameworks`, sets the `libmoira_ffi` install name to `@rpath/libmoira_ffi.dylib`, and signs the dylibs when Xcode provides a signing identity.

The BelunaApp target sets `ENABLE_USER_SCRIPT_SANDBOXING = NO` at target scope so Cargo can read the Rust workspace and write `target/` artifacts during Xcode builds.

## Out Of Scope For This Task

- Public package publishing.
- Stable external SDK promise.
- CLI package integration.
- Windows package integration.
- Sandbox and ledger implementation.
