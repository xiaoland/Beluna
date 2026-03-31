# L3 Plan 04 - Queue, Shutdown, And Wire Cutover
- Task Name: `core-cortex-act-stem-refactor`
- Stage: `L3`
- Focus: concrete cutover plan for queue semantics, shutdown gating, and wire protocol
- Status: `DRAFT_FOR_APPROVAL`

## 1) Queue Cutover Steps
1. replace `unbounded_channel` usage in runtime ingress with bounded `mpsc::channel`.
2. remove continuity-owned runtime queues (`sense_queue`, `neural_signal_queue`) from active control flow.
3. route all producer sends through `SenseIngress`.

Gate:
1. no runtime path enqueues sense outside `SenseIngress`.

## 2) Ingress Gate Implementation Steps
1. add `gate_open: AtomicBool` to ingress wrapper.
2. `send()` checks `gate_open` before `tx.send().await`.
3. `close_gate()` flips `gate_open` to false.
4. `send_sleep_blocking()` bypasses gate check and always tries enqueue.

Gate:
1. after `close_gate()`, producer `send()` returns deterministic ingress-closed error.

## 3) Shutdown Sequence Steps
1. signal handler task catches `SIGINT`/`SIGTERM`.
2. call `close_gate()`.
3. call `send_sleep_blocking().await`.
4. await stem task completion.
5. run adapter/continuity cleanup.

Gate:
1. if queue is full, shutdown waits until `sleep` successfully enqueued.

## 4) Wire Protocol Migration Steps
1. in `spine/adapters/wire.rs`, remove `AdmissionFeedback` ingress variants.
2. add parsing/encoding for:
   - `new_capabilities`,
   - `drop_capabilities`.
3. map register/unregister lifecycle messages to patch/drop senses as needed.
4. keep correlated domain `sense` validation logic for execution echo fields.

Gate:
1. old admission-feedback messages are rejected by parser.

## 5) Config/Schema Cutover Steps
1. remove old loop batching config fields from:
   - `core/src/config.rs`,
   - `core/beluna.schema.json`.
2. keep `loop.sense_queue_capacity`.
3. ensure config deserialization defaults remain valid.

Gate:
1. config load passes with updated schema and runtime uses only new fields.

## 6) Observability/Diagnostics Steps
1. log ingress gate closure.
2. log blocking sleep enqueue start/finish.
3. log send rejections due to closed gate.
4. log stem shutdown completion.

Gate:
1. shutdown trace can be followed deterministically from logs.

Status: `READY_FOR_EXECUTION`
