# L2 Plan 01 - Runtime Skeleton and AppContext DI
- Task: `cortex-loop-architecture`
- Micro-task: `01-runtime-skeleton-and-app-context-di`
- Stage: `L2`
- Date: `2026-03-01`
- Status: `DRAFT_FOR_APPROVAL`

## 1) Goal and Non-Goals
Goal:
1. Keep Stem as clock authority (tick emitter only).
2. Make CortexRuntimeLoop independent and self-driven by hybrid triggers (tick + sense + act-side events).
3. Make CortexRuntimeLoop own cycle id issuance.
4. Remove any `Stem -> Cortex` direct invocation path.

Non-goals in micro-task `01`:
1. No afferent deferral policy implementation details (`03` owns that).
2. No final efferent extraction internals (`07` owns that), only producer seam usage.
3. No full tool schema refactor (`04` owns that), only runtime seam for sleep control state.

## 2) Runtime Topology (Corrected)
```text
main() composition root
  |- build pathways via Stem-owned modules
  |- build Cortex + Continuity + Spine instances
  |- spawn StemTickRuntime (tick emitter only)
  |- spawn CortexRuntimeLoop (hybrid execution owner)

StemTickRuntime
  |- owns tick interval timer
  |- emits TickGrant over channel
  |- does NOT issue cycle ids
  |- does NOT consume CycleResult
  |- does NOT schedule wait_for_sense/sleep behavior

CortexRuntimeLoop
  |- owns afferent consume loop
  |- owns cycle_id issuance
  |- owns missed tick handling policy
  |- owns cognition persistence call to Continuity
  |- owns act enqueue to efferent_producer
  |- owns ignore_all_trigger_until gate (sleep tool target state)
```

Hard boundary:
1. Stem runtime has no dependency on `Arc<Cortex>`.
2. Stem runtime has no dependency on Continuity/Spine execution flow.
3. Cortex runtime never receives execution callbacks from Stem.

## 3) Dependency Bundles (Locked)
```rust
pub struct StemDeps {
    pub tick_interval_ms: u64,
    pub tick_grant_tx: tokio::sync::mpsc::Sender<TickGrant>,
}

pub struct CortexDeps {
    pub tick_grant_rx: tokio::sync::mpsc::Receiver<TickGrant>,
    pub afferent_consumer: SenseConsumerHandle,
    pub efferent_producer: ActProducerHandle,
    pub init_cognition_state: CognitionState,
    pub physical_state_reader: Arc<dyn PhysicalStateReadPort>,
    pub cortex_core: Arc<Cortex>,
}
```

Notes:
1. Continuity persistence is direct from Cortex runtime; the concrete Continuity access is resolved inside Cortex-side composition (through `cortex_core` wiring path), not through `StemDeps`.
2. `main()` owns wiring only; no dedicated `core/src/runtime` module is introduced.

## 4) Contract Placement (Locked)
1. `TickGrant` contract lives in `core/src/stem/runtime.rs`.
2. `AppContext` and lifecycle holder live in `core/src/main.rs`.
3. There will be no `core/src/runtime/*` module tree.

## 5) Protocol Contracts
```rust
pub struct TickGrant {
    pub tick_seq: u64,
    pub emitted_at: std::time::Instant,
}
```

Protocol invariants:
1. Stem emits ticks only; it does not interpret missed-tick policy.
2. Cortex may skip/coalesce/catch-up ticks based on its own loop policy.
3. Afferent sense arrivals can trigger Cortex cycles even without an immediate tick.
4. Cycle id is generated only inside Cortex runtime.

## 6) State Machines
### 6.1 App Lifecycle (in `main.rs`)
```rust
enum AppState {
    Init,
    Starting,
    Running,
    Closing,
    Closed,
}
```

### 6.2 StemTickRuntime State Machine
```rust
enum StemTickState {
    Running,
    Closing,
    Closed,
}
```

Behavior:
1. `Running`: interval timer emits `TickGrant { tick_seq, emitted_at }`.
2. On shutdown token: `Closing` then `Closed`.
3. No wait/sleep scheduling logic is allowed in StemTickRuntime.

### 6.3 CortexRuntimeLoop State Machine
```rust
enum CortexLoopState {
    Running,
    IgnoringUntil(std::time::Instant),
    Closing,
    Closed,
}
```

Owned mutable state:
1. `cycle_id: u64`
2. `cognition_state: CognitionState`
3. `pending_senses: VecDeque<Sense>`
4. `ignore_all_trigger_until: Option<Instant>`
5. missed-tick policy counters/markers.

Behavior:
1. On tick (`TickGrant`) and/or sense arrival: decide whether to run a cycle.
2. If cycle runs: increment `cycle_id`, snapshot physical state, invoke `cortex_core`, persist cognition directly, enqueue acts to `efferent_producer`.
3. If `ignore_all_trigger_until` is active and deadline not reached: ignore triggers per Cortex policy.
4. On shutdown token: stop intake, flush bounded operations, exit.

## 7) Wait and Sleep Model (Corrected)
1. `wait_for_sense` is not a Stem scheduling concern.
2. Waiting/deferring behavior is implemented by afferent policy engine (`03`), controlled by Cortex tooling.
3. Remove Stem-level act sleep interception model.
4. Sleep behavior becomes a Cortex Primary tool effect: set `ignore_all_trigger_until`.

## 8) Thread Ownership Matrix
| Owner | Mutable State | Must Not Own |
|---|---|---|
| `main` | app lifecycle + join handles + wiring | loop policy logic |
| `StemTickRuntime` | tick timer + tick_seq | cycle_id, cognition, acts |
| `CortexRuntimeLoop` | cycle_id + cognition + pending senses + ignore gate | Stem scheduler state |

## 9) L2 Freeze Constraints Before L3
1. `StemDeps` and `CortexDeps` exact shapes are frozen as above.
2. `TickGrant` is tick-only contract; no `CycleResult` channel exists.
3. Cortex owns cycle issuance and missed-tick policy.
4. Stem wait/sleep logic is removed from design.
5. Sleep act is removed from Stem; replaced by `ignore_all_trigger_until` control path in Cortex runtime.
6. No `core/src/runtime` module is introduced.

## 10) Exit Criteria (L2-01)
1. Ownership boundaries are unambiguous and match the corrected model.
2. Stem is tick-only and non-driving for Cortex execution.
3. Cortex owns execution, persistence, and efferent enqueue responsibilities.
4. Contract/file placement constraints are explicit for L3 implementation.

Status: `READY_FOR_REVIEW`
