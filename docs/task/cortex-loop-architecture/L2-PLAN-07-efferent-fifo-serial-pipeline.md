# L2 Plan 07 - Efferent FIFO Serial Pipeline
- Task: `cortex-loop-architecture`
- Micro-task: `07-efferent-fifo-serial-pipeline`
- Stage: `L2`
- Date: `2026-03-02`
- Status: `DRAFT_FOR_APPROVAL`

## 1) Goal and Scope
Goal:
1. Freeze the efferent pathway as a dedicated FIFO dispatch pipeline owned by Stem modules and consumed serially as `Continuity -> Spine`.
2. Keep Cortex runtime as producer-only through DI handle (`ActProducerHandle`) and remove any hidden dispatch coupling.
3. Lock ordering, backpressure, and shutdown-drain behavior.

In scope:
1. Efferent queue contracts and envelope schema.
2. Producer/consumer API boundaries and ownership.
3. Serial dispatch worker algorithm and status emission policy.
4. Shutdown drain and timeout behavior.

Out of scope:
1. Afferent deferral/wait-gate behavior (`03`, `04`).
2. Full docs/contracts rewrite (`08`).

## 2) Ownership and Module Contract Freeze
1. Efferent pathway remains under Stem namespace and is instantiated by runtime wiring, then injected to Cortex runtime.
2. `main()` is composition root only; it does not build pathway internals directly.
3. Cortex never calls Continuity/Spine dispatch APIs directly; it only enqueues efferent envelopes.
4. Efferent consumer is single-threaded and serial for deterministic dispatch order.

## 3) Data Model Freeze
1. Queue envelope fields are canonical:
- `cycle_id: u64` (issued by Cortex runtime loop).
- `act_seq_no: u64` (monotonic within one cycle, starts at `1`).
- `act: Act`.
2. `act_instance_id` remains the cross-component correlation key.
3. `act_seq_no` assignment authority is Cortex runtime at enqueue time.

## 4) API Contract Freeze
Producer surface:
1. Producer must support enqueueing one act envelope at a time with explicit `act_seq_no`.
2. Enqueue failure is observable (closed queue/backpressure failure logging path is deterministic).
3. Producer intake can be closed for shutdown.

Consumer surface:
1. Consumer reads envelopes strictly FIFO from bounded `mpsc`.
2. For each envelope, dispatch stages are fixed:
- stage 1: Continuity gate (`Continue|Break`).
- stage 2: Spine final dispatch (only if stage 1 is `Continue`).
3. Terminal status emission is required per accepted envelope:
- `ACK | REJECTED | LOST`.

## 5) Dispatch Algorithm Freeze
Per-envelope deterministic flow:
1. Emit `DISPATCHING` status in Stem proprioception namespace.
2. Run Continuity gate with `DispatchContext { cycle_id, act_seq_no }`.
3. If Continuity returns `Break`, short-circuit this act and mark terminal `REJECTED`.
4. Otherwise dispatch to Spine and map result to terminal status:
- `Acknowledged -> ACK`
- `Rejected -> REJECTED`
- `Lost | error -> LOST`
5. Emit terminal status and maintain bounded retention (FIFO drop of old status keys).

Queue ordering:
1. Accepted enqueue order defines dispatch order.
2. Continuity break of one act never blocks queue progression for following acts.

## 6) Shutdown and Drain Contract Freeze
1. Shutdown must be two-phase:
- stop producer intake (or producer task exits and drops sender),
- drain remaining queue within bounded timeout.
2. If drain timeout is reached:
- stop worker,
- report dropped envelope count with warning telemetry.
3. No infinite drain wait is allowed.

## 7) Configuration and Observability Freeze
Config additions:
1. `loop.efferent_queue_capacity` (already present via Cortex outbox capacity mapping; final source must be explicit and documented).
2. `loop.efferent_shutdown_drain_timeout_ms` (new).
3. Optional retry knobs for dispatch-stage transient failures are deferred unless explicitly required in implementation review.

Observability minimums:
1. enqueue rejected/queue-closed logs.
2. per-act dispatch stage result logs with `cycle_id`, `act_seq_no`, `act_instance_id`.
3. shutdown drain summary: processed, remaining, dropped_on_timeout.

## 8) File and Interface Targets for L3
1. `core/src/stem/runtime.rs` (extract/move efferent section).
2. `core/src/stem/efferent_pathway.rs` (new module).
3. `core/src/stem.rs` (re-export updates).
4. `core/src/cortex/runtime.rs` (explicit `act_seq_no` assignment + new enqueue API usage).
5. `core/src/main.rs` (shutdown/drain wiring).
6. `core/src/config.rs`
7. `core/beluna.schema.json`

## 9) Risks and Constraints
1. Risk: current seq numbering resets to `1` for each single-act enqueue call.
Mitigation: explicit `act_seq_no` in producer API and cycle-local monotonic assignment in Cortex runtime.
2. Risk: immediate cancel currently can bypass queue drain.
Mitigation: enforce two-phase shutdown with bounded drain timeout.
3. Risk: over-designing retries increases coupling.
Mitigation: keep retry optional and default to no retry in this micro-task unless a concrete failure mode requires it.

## 10) L2 Exit Criteria (07)
1. Efferent envelope schema and sequence authority are unambiguous.
2. Serial `Continuity -> Spine` algorithm is deterministic and fully specified.
3. Backpressure and bounded drain behavior are defined.
4. File-level L3 implementation surface is frozen.

Status: `READY_FOR_REVIEW`
