# Environments

## Primary Runtime Environment

- Core process runs as foreground binary with JSONC config.
- Body endpoints may be inline (same process) or external (Unix socket).

## Current Release Consumer Environment

1. Moira is currently the first-party release consumer for local Core activation.
2. The current release-consumer assumption is macOS-first.
3. The first supported release-consumer target is `aarch64-apple-darwin`.
4. Release assets consumed by Moira must follow the Product TDD packaging contract.
5. The first release-producer workflow is pinned to a GitHub-hosted macOS arm64 runner (`macos-14`) to match the current first supported consumer target.

## Release And CI Build Environment

1. Local Moira backend checks use the default prebuilt DuckDB path.
2. Release and CI jobs that need source-bundled DuckDB pass `--features duckdb-bundled` to the Moira Rust build command.
3. The baseline bundled runtime verification command is `cargo check --manifest-path moira/runtime/Cargo.toml --locked --features duckdb-bundled`.
4. The baseline bundled Apple binding verification command is `cargo check -p moira-ffi --locked --features duckdb-bundled`.

## Configuration Model

1. One unified config file input (`beluna.jsonc`).
2. Typed config structs define shape/defaults/validation.
3. Schema generation is code-driven and explicit.
