# L2-02 Stem Scheduler and Dispatch
- Task: `cortex-autonomy-refactor`
- Stage: `L2`

## 1) Runtime State Machine
```rust
enum StemMode {
    Active,
    SleepingUntil(Option<std::time::Instant>), // None = wait for new sense only
}
```

Terminal condition:
- `Sense::Hibernate`.

## 2) Tick Configuration
Config additions:
1. `loop.tick_interval_ms` (default `1000`)
2. `loop.tick_missed_behavior` (`skip` only)

Runtime:
```rust
let mut interval = tokio::time::interval(Duration::from_millis(tick_interval_ms));
interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
```

## 3) Sense Model
`core/src/types.rs` target:
```rust
pub enum Sense {
    Domain(SenseDatum),
    Hibernate,
    NewNeuralSignalDescriptors(NeuralSignalDescriptorPatch),
    DropNeuralSignalDescriptors(NeuralSignalDescriptorDropPatch),
}
```

No `Sense::Sleep`.

## 4) Afferent / Efferent Communication Model

### 4.1 Afferent-Pathway
Implementation:
- bounded `tokio::mpsc::Sender<Sense>` + single `Receiver<Sense>`.

Producers:
1. Body endpoints (through Spine adapters)
2. Continuity (holds sender handle)
3. Spine (holds sender handle)
4. Main (shutdown injects `Hibernate`)

Consumer:
1. Stem only

### 4.2 Efferent-Pathway
Implementation:
- in-process serialized dispatch pipeline in Stem over each `Act`.

Producer:
1. Cortex (`acts` output)

Consumers (middleware chain):
1. Stem built-in control act handler
2. Continuity `on_act`
3. Spine `on_act`

No return payloads across pathway boundaries; failure/feedback travels back as `Sense` through afferent-pathway.

## 5) Sleep Act Contract
Built-in control act descriptor provided by Stem:
1. endpoint: `core.control`
2. act: `sleep`
3. payload schema:
```json
{
  "type": "object",
  "properties": {
    "seconds": { "type": "integer", "minimum": 1 }
  },
  "required": ["seconds"],
  "additionalProperties": false
}
```

Behavior:
1. Stem intercepts sleep act before middleware chain.
2. Stem sets `StemMode::SleepingUntil(Some(now + seconds))`.
3. sleep act is not forwarded to Continuity/Spine.

## 6) Dispatch Middleware Model (Per-Act)
Dispatch contract:
```rust
enum DispatchDecision {
    Continue,
    Break,
}

trait ActMiddleware {
    fn on_act(&mut self, act: &Act, ctx: &DispatchContext) -> Result<DispatchDecision, Error>;
}
```

Per-act pipeline:
1. evaluate Stem control act interceptors.
2. `continuity.on_act(act, ctx)` -> `Continue|Break`.
3. if `Continue`, `spine.on_act(act)` -> `Continue|Break`.
4. stop current act on first `Break`.
5. continue next act regardless.

No `ActDispatchResult` contract required in this phase.

## 7) Active Mode Algorithm
1. wait tick.
2. drain queue with `try_recv` (may be empty).
3. if any `Hibernate`: terminate loop.
4. apply capability patch/drop senses.
5. invoke Cortex with drained domain senses (can be empty).
6. `continuity.persist_cognition_state(new_cognition_state)`.
7. dispatch each act through per-act middleware chain.
8. loop.

## 8) Sleeping Mode Algorithm
`SleepingUntil(deadline)` behavior:
1. wait for whichever comes first:
- new sense arrival,
- deadline timeout.
2. if `Hibernate`: terminate.
3. if deadline reached and no hibernate: switch to `Active`.
4. if new domain/capability sense arrives before deadline: switch to `Active` and process immediately (wake-cycle).

`SleepingUntil(None)` behavior (future optional):
1. only wake on new sense.

## 9) Continuity and Spine on_act (Current Phase)
1. `continuity.on_act` currently returns `Continue` and performs no side effect.
2. `spine.on_act` performs dispatch; on error/rejection it emits correlated failure sense via afferent-pathway sender.

## 10) Main Shutdown Path
1. signal handler closes afferent gate.
2. enqueue `Sense::Hibernate`.
3. await stem completion.
4. flush continuity JSON.

