# L3 Plan 01 - Runtime Skeleton and AppContext DI
- Task: `cortex-loop-architecture`
- Micro-task: `01-runtime-skeleton-and-app-context-di`
- Stage: `L3`
- Date: `2026-03-01`
- Status: `DRAFT_FOR_APPROVAL`

## 1) Scope and Goal
Goal:
1. Implement tick-emitter Stem runtime and hybrid Cortex runtime with Cortex-owned cycle issuance.
2. Remove any Stem-driven Cortex execution and `CycleResult` feedback path.
3. Keep DI composition in `main.rs` without introducing `core/src/runtime/*`.

In scope:
1. `StemDeps`, `CortexDeps`, and `TickGrant` contract implementation.
2. Cortex runtime ownership of cognition persistence and efferent enqueue.
3. Runtime support for `ignore_all_trigger_until` state (sleep tool target behavior).

Out of scope:
1. Afferent policy engine details (`03`).
2. Structured tool schema and emission model finalization (`04`).
3. Full efferent module extraction internals (`07`).

## 2) Implementation Roadmap
### Step 1 - Contract Reshape in `stem/runtime.rs`
1. Define `TickGrant` in `core/src/stem/runtime.rs`:
```rust
pub struct TickGrant {
    pub tick_seq: u64,
    pub emitted_at: std::time::Instant,
}
```
2. Define `StemDeps` exactly as locked:
```rust
pub struct StemDeps {
    pub tick_interval_ms: u64,
    pub tick_grant_tx: tokio::sync::mpsc::Sender<TickGrant>,
}
```
3. Remove Stem-side `CycleResult` receive path and related state.

### Step 2 - StemTickRuntime Simplification
1. Refactor Stem runtime loop to timer-emitter only:
- run interval
- increment `tick_seq`
- send `TickGrant`
- exit on shutdown.
2. Remove Stem scheduling concerns:
- no `wait_for_sense`
- no `SleepingUntil`
- no act-sleep interception.

Pseudo-code:
```rust
loop {
    tokio::select! {
        _ = tick.tick() => {
            tick_seq += 1;
            let _ = tick_grant_tx.send(TickGrant { tick_seq, emitted_at: Instant::now() }).await;
        }
        _ = shutdown.cancelled() => break,
    }
}
```

### Step 3 - CortexDeps and Runtime Construction
1. Define `CortexDeps` exactly as locked:
```rust
pub struct CortexDeps {
    pub tick_grant_rx: tokio::sync::mpsc::Receiver<TickGrant>,
    pub afferent_consumer: SenseConsumerHandle,
    pub efferent_producer: ActProducerHandle,
    pub init_cognition_state: CognitionState,
    pub physical_state_reader: Arc<dyn PhysicalStateReadPort>,
    pub cortex_core: Arc<Cortex>,
}
```
2. Move afferent consumer ownership fully to Cortex runtime.
3. Keep Continuity persistence call inside Cortex-side execution path (wired through Cortex composition, not Stem).

### Step 4 - Implement `core/src/cortex/runtime.rs`
1. Add independent Cortex loop with owned mutable state:
- `cycle_id`
- `cognition_state`
- `pending_senses`
- `ignore_all_trigger_until`.
2. Handle hybrid triggers:
- tick trigger via `tick_grant_rx`
- sense trigger via `afferent_consumer`
- act-side feedback triggers via incoming senses (same afferent channel).
3. Missed tick policy stays in Cortex runtime (skip/coalesce/catch-up decision local).

Pseudo-code:
```rust
loop {
    tokio::select! {
        Some(grant) = tick_grant_rx.recv() => on_tick(grant),
        Some(sense) = afferent_consumer.recv() => on_sense(sense),
        _ = shutdown.cancelled() => break,
    }

    if should_run_cycle(now, ignore_all_trigger_until, pending_senses, last_tick_state) {
        cycle_id += 1;
        let physical = physical_state_reader.snapshot(cycle_id).await?;
        let senses = collect_cycle_senses(&mut pending_senses);
        let output = cortex_core.cortex(&senses, &physical, &cognition_state).await?;

        cognition_state = persist_cognition_directly(output.new_cognition_state).await?;
        efferent_producer.enqueue(output.acts).await?;
    }
}
```

### Step 5 - Sleep Model Migration
1. Remove `core.control/sleep` interception path from Stem runtime.
2. Add Cortex runtime field and setter path:
- `ignore_all_trigger_until: Option<Instant>`
3. Define runtime behavior:
- when set and `now < deadline`, Cortex loop ignores triggers by policy.
4. Actual Primary tool wiring that sets this field is completed in micro-task `04`; micro-task `01` provides runtime seam only.

### Step 6 - Main Composition in `main.rs` (No `core/src/runtime`)
1. Move/define in `main.rs`:
- app lifecycle state holder
- `AppContext` struct
- bootstrap and shutdown orchestration.
2. Build pathways, then construct:
- `StemDeps`
- `CortexDeps`
3. Spawn two tasks:
- `StemTickRuntime`
- `CortexRuntimeLoop`
4. Ensure no global runtime module is introduced.

### Step 7 - Shutdown Sequencing
1. On signal: transition lifecycle to `Closing`.
2. Close afferent ingress.
3. Cancel Stem and Cortex runtime loops.
4. Join Cortex and Stem tasks with bounded timeouts.
5. Flush continuity and shutdown spine/adapters.
6. Transition to `Closed`.

## 3) File-Level Change Plan
1. `core/src/main.rs`
- add lifecycle + AppContext definitions and wiring
- spawn both runtimes
- remove reliance on Stem-driven Cortex execution lifecycle.
2. `core/src/stem/runtime.rs` (new/extracted)
- define `TickGrant`
- define `StemDeps`
- implement tick-emitter runtime.
3. `core/src/stem.rs` or `core/src/stem/mod.rs`
- remove legacy scheduler/wait/sleep behavior paths.
4. `core/src/cortex/runtime.rs` (new)
- implement hybrid loop, cycle issuance, persistence+efferent responsibilities.
5. `core/src/afferent_pathway.rs`
- ensure split handles support Cortex-owned consumer handle.
6. `core/src/lib.rs`
- update module exports for refactored paths.

## 4) Risk Control Gates
1. Gate A: build after contract reshape (`TickGrant`, deps).
2. Gate B: build after Stem simplification (tick-only emitter).
3. Gate C: build after Cortex runtime loop integration.
4. Gate D: grep guards:
```bash
rg -n "\.cortex\(" core/src/stem
rg -n "wait_for_sense|SleepingUntil|core.control/sleep" core/src/stem
```
Expected:
1. no direct Stem call to Cortex.
2. no Stem wait/sleep scheduling remnants.

## 5) Build Verification
Per workspace rule, build only:
```bash
cargo build -p beluna-core
cargo build -p beluna-cli
```

## 6) Completion Criteria
1. Stem runtime is tick emitter only.
2. Cortex runtime owns cycle issuance and hybrid execution decisions.
3. Cortex runtime directly handles cognition persistence and efferent enqueue.
4. No `core/src/runtime` module exists.
5. Build passes for core and CLI.

Status: `READY_FOR_REVIEW`
