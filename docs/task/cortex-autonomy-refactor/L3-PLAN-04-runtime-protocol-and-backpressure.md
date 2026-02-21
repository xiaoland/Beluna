# L3-04 Runtime Protocol and Backpressure
- Task: `cortex-autonomy-refactor`
- Stage: `L3`

## 1) Afferent-Pathway Protocol

### 1.1 Transport
1. bounded `tokio::mpsc::channel<Sense>`.
2. gateable producer wrapper (`SenseAfferentPathway`).

### 1.2 Producers
1. Main: `Hibernate` on SIGINT/SIGTERM.
2. Spine adapters: domain senses and capability patch/drop senses.
3. Spine runtime: dispatch-failure senses.
4. Continuity: reserved sender ownership (currently no emission in this phase).

### 1.3 Consumer
1. Stem only.

## 2) Efferent-Pathway Protocol
1. Cortex emits ordered `Act[]` per cycle.
2. Stem dispatches each act through middleware chain:
- stem built-in control interceptors
- continuity `on_act`
- spine `on_act`
3. Stage return type: `DispatchDecision::{Continue, Break}`.
4. `Break` drops current act only; next act continues.

## 3) Backpressure and Queue Behavior
1. bounded queue remains authoritative for inbound pressure.
2. senders block when queue is full (native mpsc semantics).
3. no unbounded fallback queue is introduced.
4. stem drains queue non-blocking during active ticks.

## 4) Sleep/Wake Protocol
1. sleep act (`core.control/sleep`) payload includes `seconds >= 1`.
2. stem intercepts and sets local sleeping deadline.
3. during sleeping mode, stem wakes on:
- deadline timeout,
- new sense arrival,
- hibernate sense.
4. wake-cycle runs immediately without extra 1-second tick wait.

## 5) Shutdown Protocol
1. Main closes afferent gate.
2. Main enqueues `Sense::Hibernate`.
3. Stem exits loop deterministically.
4. Continuity flushes persisted state.

## 6) Failure Feedback Protocol
1. Continuity guardrail failure on persist => cortex cycle warning + no state mutation.
2. Spine act dispatch failure => emit correlated failure sense into afferent-pathway.
3. Cortex helper/primary failure => noop fallback within current cycle.

## 7) Ownership Summary
1. Stem owns runtime scheduling and middleware chaining.
2. Cortex owns cognition patch synthesis and patch application into new cognition state.
3. Continuity owns persistence and guardrail validation.
4. Spine owns physical dispatch and dispatch failure signaling.

