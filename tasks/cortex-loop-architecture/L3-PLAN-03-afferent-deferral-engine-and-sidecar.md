# L3 Plan 03 - Afferent Deferral Engine and Sidecar
- Task: `cortex-loop-architecture`
- Micro-task: `03-afferent-deferral-engine-and-sidecar`
- Stage: `L3`
- Date: `2026-03-01`
- Status: `DRAFT_FOR_APPROVAL`

## 1) Objective
Implement `stem::afferent_pathway` as a deferral-aware scheduler with:
1. single-rule overwrite,
2. full reset,
3. FIFO deferral buffer with overflow eviction,
4. observe-only sidecar,
while keeping Cortex as the afferent consumer owner.

## 2) Execution Steps
### Step 1 - Move Afferent Pathway Under Stem
1. Move `core/src/afferent_pathway.rs` into `core/src/stem/afferent_pathway.rs`.
2. Update `core/src/stem.rs` to expose `pub mod afferent_pathway` and re-exports.
3. Rewrite imports from `crate::afferent_pathway::*` to `crate::stem::afferent_pathway::*`.

### Step 2 - Introduce Rule-Control and Sidecar Contracts
1. Add `AfferentRuleControlPort` and command/result types.
2. Add `AfferentSidecarPort`, subscription handle, and event enum.
3. Ensure control methods are pathway-owned and independent of Cortex tooling layer.

Pseudo-code:
```rust
enum RuleCommand {
    OverwriteOne { input, reply_tx },
    ResetAll { reply_tx },
    Snapshot { reply_tx },
}
```

### Step 3 - Implement Deferral State and Match Engine
1. Add `DeferralState { revision, rules_by_id, deferred_fifo }`.
2. Implement `compile_rule(input) -> DeferralRuleRuntime` with validation.
3. Implement `matches(rule, sense, fq_sense_id)` where `min_weight` match means `sense.weight < min_weight`.
4. Implement `matching_rule_ids(state, sense)`.

### Step 4 - Implement Scheduler Loop
1. Internal task owns mutable state.
2. `tokio::select!` over ingress, control commands, shutdown.
3. Ingress handling:
- matched => defer + sidecar deferred event.
- unmatched => forward to consumer queue.
4. Overflow handling:
- while `deferred_fifo.len() > max_deferring_nums`, evict front.
- log warning + sidecar evicted event.

Pseudo-code:
```rust
if matching_rule_ids.is_empty() {
    egress_tx.send(sense).await?;
} else {
    deferred_fifo.push_back(entry);
    enforce_cap_and_evict_oldest();
}
```

### Step 5 - Implement Single-Rule Overwrite + Reset
1. `overwrite_rule`:
- validate + compile rule.
- `rules_by_id.insert(rule_id, compiled_rule)`.
- increment revision.
- emit `RuleOverwritten`.
- call `release_fifo_front_while_unblocked()`.
2. `reset_rules`:
- clear `rules_by_id`.
- increment revision.
- emit `RulesReset`.
- call `release_fifo_front_while_unblocked()`.

Release algorithm:
```rust
loop {
    let Some(front) = deferred_fifo.front() else { break };
    if still_matches_any_rule(front.sense) { break; }
    let entry = deferred_fifo.pop_front().unwrap();
    egress_tx.send(entry.sense).await?;
    emit_released(entry);
}
```

### Step 6 - Wire Cortex Primary Integration Boundary
1. Inject `Arc<dyn AfferentRuleControlPort>` into Cortex runtime/primary dependency wiring.
2. Do not add tool schema here; only provide callable port boundary for micro-task `04`.

### Step 7 - Sidecar Observe-Only Guarantees
1. Implement subscribe-only API.
2. Use non-blocking sidecar publish strategy.
3. On lag/drop, log warning but never block scheduler.

## 3) File-Level Change Map
1. Move/create: `core/src/stem/afferent_pathway.rs`.
2. Update: `core/src/stem.rs`.
3. Update imports in:
- `core/src/main.rs`
- `core/src/spine/runtime.rs`
- `core/src/continuity/engine.rs`
- other direct users.
4. Update dependency structs if needed to pass `AfferentRuleControlPort` boundary.

## 4) Verification Gates
### Gate A - Module Ownership
```bash
rg -n "mod afferent_pathway|stem::afferent_pathway" core/src/stem.rs core/src -g'*.rs'
```
Expected:
1. afferent pathway implemented under `stem` module.
2. no remaining `crate::afferent_pathway` imports.

### Gate B - Rule Semantics
```bash
rg -n "overwrite_rule|reset_rules|RuleOverwritten|RulesReset" core/src/stem/afferent_pathway.rs
```
Expected:
1. overwrite targets one rule by `rule_id`.
2. reset clears all rules.

### Gate C - No Remove API
```bash
rg -n "remove_rule|delete_rule" core/src/stem/afferent_pathway.rs
```
Expected: no remove/delete rule API.

### Gate D - Sidecar Scope
```bash
rg -n "AfferentSidecar|subscribe" core/src/stem/afferent_pathway.rs
```
Expected: sidecar API is observe-only and contains no mutator methods.

### Gate E - Build
```bash
cd core && cargo build
cd ../cli && cargo build
```

## 5) Test Plan (Implementation-Time)
1. Unit: rule validation rejects invalid regex and invalid weight.
2. Unit: overwrite upserts one rule and preserves others.
3. Unit: reset clears all rules.
4. Unit: deferral matching by weight.
5. Unit: deferral matching by regex.
6. Unit: FIFO release stops at first still-blocked entry.
7. Unit: overflow eviction removes oldest deferred entries first.
8. Unit: sidecar publish failure/lag does not block ingress/control processing.

## 6) Completion Criteria (03)
1. Afferent pathway runs with deterministic deferral scheduling.
2. Rule-control port is pathway-owned and ready for Primary tool wrapping.
3. Single-rule overwrite + full reset behavior is enforced.
4. Sidecar is observe-only and non-blocking.
5. `core` and `cli` build successfully after import/module rewiring.

Status: `READY_FOR_REVIEW`
