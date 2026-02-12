# L3-04 - Runtime Protocol And Backpressure
- Task Name: `cortex-mvp`
- Stage: `L3` detail: runtime integration
- Date: `2026-02-11`
- Status: `DRAFT_FOR_APPROVAL`

## 1) Runtime Wiring Goals
1. Maintain always-on `CortexReactor` task in server runtime.
2. Convert ingress event stream into bounded `ReactionInput` stream.
3. Keep upstream mechanical: event delivery + backpressure only.
4. Preserve existing exit message behavior.

## 2) Protocol Additions (NDJSON)
Add event envelope forms:
1. `{"type":"sense", ...}`
2. `{"type":"env_snapshot", ...}`
3. `{"type":"admission_feedback", ...}`
4. `{"type":"capability_catalog_update", ...}` (optional)
5. `{"type":"cortex_limits_update", ...}` (optional)
6. `{"type":"intent_context_update", ...}` (optional)

Keep existing:
1. `{"type":"exit"}`

Parser strategy:
1. strict enum + deny unknown fields per event struct.
2. invalid message is ignored with log; reactor task remains alive.

## 3) Ingress Assembler
`CortexIngressAssembler` responsibilities:
1. keep latest env snapshots by endpoint key.
2. keep recent feedback window keyed by `attempt_id`.
3. keep latest capability catalog and limits.
4. keep latest intent context payload.
5. on every `sense` event, emit one bounded `ReactionInput`.

Non-responsibilities:
1. no intent arbitration.
2. no route selection.
3. no semantic policy decisions.

## 4) Bounded Channel Design
Channels:
1. `ingress -> reactor`: `mpsc::channel<ReactionInput>(N_inbox)`
2. `reactor -> downstream`: `mpsc::channel<ReactionResult>(N_outbox)`

Behavior:
1. use `send().await` for natural backpressure in normal mode.
2. optional `try_send()` path can return deterministic rejection counters.
3. no unbounded queue fallback.

## 5) Reactor Lifecycle In Server
Server startup flow:
1. load config.
2. construct ai-gateway and cortex adapters.
3. construct bounded channels and ingress assembler.
4. spawn `CortexReactor::run` task.
5. accept socket connections and feed ingress events.

Shutdown flow:
1. on exit/signal, stop accepting new ingress.
2. close inbox sender.
3. await reactor task completion with timeout.
4. cleanup socket path.

## 6) Downstream Bridge
`ReactionResult` handling direction:
1. if `attempts` empty, no Admission call for that cycle.
2. if attempts present, forward to existing Admission/Continuity path.
3. admission outputs feed back as `admission_feedback` ingress events with `attempt_id` correlation.

## 7) Config And Schema Changes
New config section:
1. `cortex.inbox_capacity`
2. `cortex.outbox_capacity`
3. `cortex.default_limits` (all ReactionLimits fields)
4. `cortex.primary_backend_id` (optional override)
5. `cortex.sub_backend_id` (optional override)

Schema validation:
1. capacities >= 1.
2. `max_primary_calls == 1`.
3. `max_repair_attempts <= 1`.
4. all byte/token/time limits >= 1.

## 8) Regression Safeguards
1. `exit` protocol tests remain green.
2. invalid new ingress events do not crash server.
3. backpressure behavior is observable and bounded.
4. reactor loop failure in one cycle does not kill whole server process.

Status: `READY_FOR_L3_REVIEW`
