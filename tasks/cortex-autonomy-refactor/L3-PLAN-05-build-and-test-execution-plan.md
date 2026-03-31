# L3-05 Build and Test Execution Plan
- Task: `cortex-autonomy-refactor`
- Stage: `L3`

## 1) Required Verification Gate
Per workspace rule, required gate is build success.

Commands:
```bash
cd /Users/lanzhijiang/Development/Beluna/core
cargo build
```

Pass criteria:
1. build exits `0`.
2. no unresolved symbol or schema/config compile regressions.

## 2) Optional Focused Test Runs
Run only when needed to de-risk changed boundaries:
```bash
cd /Users/lanzhijiang/Development/Beluna/core
cargo test --test stem_bdt
cargo test --test cortex_bdt
cargo test --test continuity_bdt
```

## 3) Failure Triage Order
1. type/contract errors (`types.rs`, cortex cognition structs).
2. runtime wiring errors (`stem.rs`, `main.rs`, `config.rs`).
3. IR/helper parser errors (`cortex/ir.rs`, `helpers_*`).
4. persistence errors (`continuity/persistence.rs`).
5. protocol/dispatch errors (`spine/runtime.rs`, stem middleware chain).

## 4) Build Iteration Policy
1. implement by workstream.
2. run `cargo build` after each completed workstream.
3. avoid large unverified change bundles.

## 5) Result Recording Requirements
`RESULT.md` must include:
1. commands executed,
2. build/test outcomes,
3. skipped optional tests (if any) with explicit reason.

