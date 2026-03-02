# L3 Plan 07 - Efferent FIFO Serial Pipeline
- Task: `cortex-loop-architecture`
- Micro-task: `07-efferent-fifo-serial-pipeline`
- Stage: `L3`
- Date: `2026-03-02`
- Status: `DRAFT_FOR_APPROVAL`

## 1) Objective
Implement a dedicated Stem-owned efferent FIFO pipeline with deterministic serial dispatch (`Continuity -> Spine`), explicit per-cycle sequence numbering, and bounded shutdown drain behavior.

## 2) Execution Steps
### Step 1 - Extract Efferent Pathway Module
1. Move efferent queue/data types and worker logic from `stem/runtime.rs` into new `stem/efferent_pathway.rs`.
2. Keep public API minimal:
- `ActProducerHandle`
- `EfferentActEnvelope`
- pathway constructor
- worker spawn entrypoint.
3. Update `stem.rs` re-exports and internal imports accordingly.

### Step 2 - Harden Envelope and Producer API
1. Rename envelope sequence field to `act_seq_no` for dispatch-context clarity.
2. Change producer enqueue surface to explicit single-envelope enqueue (or equivalent API that cannot silently reset sequence).
3. Ensure queue-closed/backpressure failure path logs deterministic warning with identifiers.

### Step 3 - Fix Sequence Assignment in Cortex Runtime
1. In `cortex/runtime.rs`, assign `act_seq_no` monotonically per cycle when iterating `output.emitted_acts`.
2. Preserve emitted-act order from Primary output as queue order.
3. Pass `cycle_id + act_seq_no + act` to efferent producer.

### Step 4 - Preserve Serial Dispatch Pipeline
1. Keep worker order per envelope:
- emit `DISPATCHING` status
- Continuity `on_act` gate
- Spine `on_act_final` when allowed
- emit terminal status.
2. Keep terminal retention ring and FIFO key cleanup behavior.
3. Preserve `DispatchContext` propagation with `cycle_id` + `act_seq_no`.

### Step 5 - Add Bounded Shutdown Drain
1. Add config `loop.efferent_shutdown_drain_timeout_ms`.
2. Rework worker shutdown to prefer queue drain before forced stop.
3. On timeout, stop worker and log dropped envelope count.
4. Ensure shutdown ordering in `main.rs` supports drain:
- stop Cortex producer side first,
- then wait for efferent worker drain completion (bounded).

### Step 6 - Config and Schema Wiring
1. Extend `ConfigLoop` in `config.rs` with new drain-timeout field.
2. Add schema field and defaults in `beluna.schema.json`.
3. Thread config value through `main.rs` to efferent worker bootstrap.

### Step 7 - Runtime Result Notes
1. Update `docs/task/cortex-loop-architecture/RESULT.md` with:
- sequence numbering fix,
- drain timeout behavior,
- invariants retained (`Continuity -> Spine`, FIFO serial).

## 3) File-Level Change Map
1. `core/src/stem/runtime.rs`
2. `core/src/stem/efferent_pathway.rs` (new)
3. `core/src/stem.rs`
4. `core/src/cortex/runtime.rs`
5. `core/src/main.rs`
6. `core/src/config.rs`
7. `core/beluna.schema.json`
8. `docs/task/cortex-loop-architecture/RESULT.md`

## 4) Verification Gates
### Gate A - Efferent Extraction + API Surface
```bash
rg -n "EfferentActEnvelope|ActProducerHandle|spawn_efferent_runtime|act_seq_no" core/src/stem core/src/cortex/runtime.rs
```
Expected:
1. efferent types live under Stem module surface.
2. `act_seq_no` is explicit and propagated.

### Gate B - Serial Dispatch Order
```bash
rg -n "on_act\\(|on_act_final|DISPATCHING|ACK|REJECTED|LOST" core/src/stem
```
Expected:
1. stage order remains `Continuity -> Spine`.
2. terminal status mapping is deterministic.

### Gate C - Shutdown Drain
```bash
rg -n "efferent_shutdown_drain_timeout_ms|drain|dropped" core/src/main.rs core/src/stem
```
Expected:
1. bounded drain timeout is implemented.
2. timeout drop path is observable.

### Gate D - Build
```bash
cd core && cargo build
cd ../cli && cargo build
```

## 5) Completion Criteria (07)
1. Efferent pathway is modularized under Stem and DI-wired cleanly.
2. `act_seq_no` is monotonic per cycle and no longer resets unexpectedly.
3. Serial dispatch remains `Continuity -> Spine` with deterministic status updates.
4. Shutdown drains queue with explicit bounded timeout semantics.
5. Core and CLI build successfully.

Status: `READY_FOR_REVIEW`
