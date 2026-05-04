# Rust Disk Governance

## MVT Core

- Objective & Hypothesis: Add repo-local Rust storage guardrails so Beluna build artifacts converge on one target surface and can be inspected or swept before they grow unchecked.
- Guardrails Touched: Root AGENTS tooling changes stay localized to repo tooling; existing `core/` source changes remain untouched.
- Verification: `cargo metadata` reports the shared target directory, the maintenance script scans current hot spots, and JSON task files parse.

## Execution Notes

- key findings: `core/target` and `moira/src-tauri/target` dominate current usage; global Cargo and Rust caches are comparatively small.
- decisions made: centralize future Cargo output under repo root `target`, reduce dev/test debug info to line tables, and expose `cargo-sweep` as a preview-only maintenance task.
- final outcome: Repo tooling now points nested Cargo builds at root `target`, reduces default dev/test debug info, and exposes scan plus `cargo-sweep` dry-run entry points.
- review follow-up: Packaging now resolves Cargo's actual target directory, normalizes `dist-dir` before changing cwd, and handles Windows `.exe` outputs.
- install follow-up: `cargo-sweep` is installed locally; maintenance dry-runs now cover both the new root `target` and the pre-existing nested `target` directories.
- closeout: Dev and test profiles now disable incremental compilation to cap future `target/debug/incremental` growth.
- cleanup run: Actual `cargo-sweep --all` cleaned `25.93 GiB` from `core/target`, `170.94 MiB` from `cli/target`, and `21.19 GiB` from `moira/src-tauri/target`; `cargo-sweep` then reported no additional sweepable artifacts.
- remaining cleanup candidate: Legacy incremental directories remain under nested targets because `cargo-sweep` preserves them as current artifacts; a full legacy target reset would be a separate explicit cleanup operation.
- stage 2: Added the minimal root Cargo workspace with `core`, `cli`, and `moira/src-tauri` members using resolver 3.
- stage 2 follow-up: Moira forged local builds now resolve Cargo's target directory through `cargo metadata`; release packaging now uses `--locked`; DuckDB is pinned to the previously locked `1.10501.0` during the workspace migration.
- stage 2 lockfile: Root `Cargo.lock` is now the workspace lockfile; member lockfiles were removed after locked metadata and package checks passed.
- stage 3: Moira now defaults to a prebuilt DuckDB path via `DUCKDB_DOWNLOAD_LIB=1`; bundled source builds moved behind the explicit `duckdb-bundled` feature.
- stage 3 verification: Default Moira metadata leaves `duckdb` without `bundled`; explicit `duckdb-bundled` metadata enables `duckdb/bundled` and `libduckdb-sys/bundled`.
- stage 3 checks: `cargo check -p moira --locked`, `cargo check -p beluna-cli --locked`, `cargo check -p beluna --lib --locked`, `pnpm -C moira build`, `pnpm -C moira exec tauri info`, script syntax checks, and `git diff --check` pass.
- stage 3 storage: Current root `target` is `1.9G`, including `143M` of downloaded DuckDB prebuilt libraries; `cargo-sweep --all --dry-run` would reclaim `1.57 GiB` from the active target after verification builds.
- closeout docs: Durable Moira and deployment docs now record the default prebuilt DuckDB path, the explicit `duckdb-bundled` release/CI path, and the storage sweep commands.
- closeout actions: VS Code and repo-level Codex actions now expose `scripts/rust-storage-maintenance.sh sweep-all` for real Rust storage cleanup.
- closeout cleanup: Running `scripts/rust-storage-maintenance.sh sweep-all` cleaned `1.57 GiB` from the root `target`; post-clean dry-run reports nothing sweepable.
- repo-level action tracking: `.gitignore` now keeps `.codex/environments/environment.toml` trackable while ignoring other `.codex` local state.
