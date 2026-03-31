# L2 Plan 04 - Queue, Shutdown, And Wire Protocol
- Task Name: `core-cortex-act-stem-refactor`
- Stage: `L2`
- Focus: bounded MPSC, ingress gate, shutdown behavior, and ingress message mapping
- Status: `DRAFT_FOR_APPROVAL`

## 1) Bounded Sense Queue Contract
Queue creation in `main`:

```rust
let (sense_tx, sense_rx) = tokio::sync::mpsc::channel::<Sense>(config.loop.sense_queue_capacity);
```

Rules:
1. only one queue exists for runtime ingress (`SenseQueue`).
2. no `ActQueue`.
3. all producers share same sender wrapper.
4. sender backpressure is native bounded-channel blocking behavior.

## 2) Ingress Gate Contract
Add `core/src/ingress.rs`:

```rust
pub struct SenseIngress {
    gate_open: Arc<std::sync::atomic::AtomicBool>,
    tx: tokio::sync::mpsc::Sender<Sense>,
}
```

APIs:
1. `send(sense)`:
   - if gate closed => reject with `IngressClosed`,
   - if gate open => `tx.send(sense).await` (may block on full queue).
2. `close_gate()`:
   - forbids new senses from producers.
3. `send_sleep_blocking()`:
   - bypasses gate-open check,
   - blocks until `Sense::Sleep` is enqueued.

## 3) Shutdown Algorithm
`main` signal handling algorithm:

```rust
on SIGINT/SIGTERM:
  ingress.close_gate();
  ingress.send_sleep_blocking().await?;
  await stem_task;
  run cleanup hooks (continuity flush, adapter stop, socket cleanup);
```

Hard constraints:
1. ingress is gated before sleep enqueue.
2. sleep enqueue is blocking until success.
3. once gate is closed, producers cannot enqueue additional senses.

## 4) Producer Mapping
Runtime producers and enqueue behavior:
1. Body endpoint ingress (UnixSocket adapter) -> `SenseIngress::send`.
2. Spine-generated senses (if any) -> `SenseIngress::send`.
3. Continuity internal senses (optional diagnostics) -> `SenseIngress::send`.
4. Ledger internal senses (optional diagnostics) -> `SenseIngress::send`.
5. Main shutdown signal path -> `SenseIngress::send_sleep_blocking`.

## 5) Wire Protocol Delta

### 5.1 Keep
1. `sense` message.
2. `body_endpoint_register`.
3. `body_endpoint_unregister`.

### 5.2 Add
1. `new_capabilities`:
   - payload carries incremental capability entries.
2. `drop_capabilities`:
   - payload carries route keys to remove.

### 5.3 Remove/Deprecate Ingress Envelopes
1. `admission_feedback`.
2. any envelope tied to removed admission semantics.

### 5.4 Internal Mapping
1. `body_endpoint_register` can be translated into `Sense::NewCapabilities` patch.
2. `body_endpoint_unregister` / disconnection routes can be translated into `Sense::DropCapabilities`.
3. external `new_capabilities`/`drop_capabilities` messages are optional direct path when producer sends explicit patch senses.

## 6) Capability Patch Conflict Rule
Policy is fixed: arrival-order-wins.

Mechanics:
1. patches are applied in receive order.
2. last patch for same route key overwrites earlier entry.
3. drop patch tombstones route key until a later new patch reintroduces it.

## 7) Config And Schema Delta
`core/src/config.rs` and `core/beluna.schema.json`:
1. retain:
   - `loop.sense_queue_capacity`.
2. remove:
   - `loop.neural_signal_queue_capacity`,
   - `loop.batch_window_ms`,
   - `loop.batch_flush_sense_count`,
   - `loop.batch_max_sense_count`.

## 8) Observability Hooks
Emit structured logs for:
1. ingress gate closed,
2. sleep enqueue start/success,
3. producer send rejected due to closed gate,
4. queue-full wait durations (optional metric),
5. stem shutdown completion.

## 9) L2-04 Exit Conditions
1. queue/backpressure semantics are explicit,
2. shutdown gating and blocking sleep enqueue are unambiguous,
3. wire ingress mapping reflects control-sense contracts,
4. config/schema changes are defined for implementation.

Status: `READY_FOR_REVIEW`
