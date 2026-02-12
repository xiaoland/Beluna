# L1 Plan - Cortex MVP (High-Level Strategy)
- Task Name: `cortex-mvp`
- Stage: `L1` (high-level strategy)
- Date: `2026-02-11`
- Status: `DRAFT_FOR_APPROVAL`

## 0) Inputs Locked From L0 Approval
User decisions applied as hard constraints:
1. Cut directly to reactor-only Cortex API now (`step` API is removed, not kept for compatibility).
2. Use real `ai_gateway` calls in MVP runtime; tests must use mocks.
3. Enforce strict stateless Cortex semantics:
- Cortex does not durably persist goals/commitments.
- Goal ownership is distributed:
  - constitutional goals (boot-time),
  - environmental goals (Non-Cortex enforced),
  - emergent/contextual goals (Cortex-evaluated).
- Cortex manages selection/arbitration, not persistence.
4. Fallback after failed repair emits zero attempts (noop).

## 1) Strategy Summary
Implement Cortex as an always-on asynchronous reactor loop with bounded per-cycle cognition:
1. Replace command-style `CortexFacade::step` with `react` semantics.
2. Consume bounded `ReactionInput` deltas from an inbox stream.
3. Run one reaction cycle per input under strict caps:
- 1 primary LLM call,
- `N` sub-LLM calls max,
- optional single repair pass,
- strict attempt/payload/time/token limits.
4. Compile prose IR into `IntentAttempt[]` through cognition organs and deterministic clamp.
5. Emit non-binding attempts to downstream world (Admission) via attempt stream.
6. Keep Cortex stateless across cycles by requiring full bounded decision context in each input.

## 2) Target Architecture

```text
Sense + EnvSnapshot stream
  -> CortexInbox (bounded stream of ReactionInput)
     -> CortexReactor::run()
        -> Intent Arbitration (constitutional + environmental + emergent signals)
        -> Primary LLM (prose IR; exactly 1 call)
        -> Sub-LLM Extractor (IR -> draft attempts)
        -> Optional Sub-LLM Filler (draft -> schema-aware payloads)
        -> Deterministic Clamp (catalog/schema/caps/budgets; final authority)
        -> Optional one-time Repair (if clamp rejects all and budget allows)
     -> CortexAttemptStream (bounded stream of IntentAttemptBatch/Noop)
  -> Admission/world decides realization/denial later
```

## 3) Boundary And Responsibility Design
### Cortex owns
1. Reaction loop orchestration.
2. Intent arbitration from provided constitutional/environmental/emergent context.
3. Capability-driven routing choice (which affordance key best expresses intent).
4. LLM cognition pipeline orchestration.
5. Deterministic clamping before attempt emission.

### Non-Cortex continues to own
1. Mechanical realization/denial (Admission).
2. Economic and hard constraints enforcement.
3. Continuity state construction and feedback sensing.
4. Any persistence of world/operational state.

### Spine remains
1. Admitted-action execution boundary (unchanged by this L1).

## 4) New Canonical Cortex Contract Direction
Replace `CortexCommand`/`CortexCycleOutput` as primary surface with reactor contracts:
1. `ReactionInput` (single bounded delta window)
- ordered `Sense` entries with stable `sense_id`,
- endpoint context snapshot blobs (deterministically truncated),
- recent admission outcome codes (non-semantic),
- capability catalog,
- cycle limits/budgets,
- distributed goal context.
2. `ReactionResult`
- `IntentAttempt[]` (or empty for noop fallback),
- `based_on: [sense_id...]` and `attention_tags` for downstream optional persistence/analytics decisions.
3. `CortexReactor`
- async `run` loop consuming inbox stream and producing attempt stream.

## 5) Goal Model Strategy (Stateless + Distributed)
1. Constitutional goals are loaded at boot and passed as input context each cycle.
2. Environmental goals are represented as runtime/non-cortex constraints and included in input context.
3. Emergent goals are evaluated/arbitrated by Cortex each cycle from current context and primary IR.
4. Cortex outputs reaction grounding (`based_on`) and attention tagging, while persistence decisions remain outside Cortex.

Implication:
- Remove durable goal/commitment stores from Cortex (`CortexState` persistence model is retired for reactor path).

## 6) LLM And Routing Strategy
1. Production path uses real `ai_gateway::AIGateway` calls.
2. Cortex depends on inference via ports so tests can swap mocks.
3. Primary model output is prose IR only (no direct structured attempts from primary call).
4. Sub-LLM extractor/filler calls are bounded and cannot invent intent beyond primary IR scope.
5. Capability-driven routing is selected using runtime-provided capability catalog; unknown affordances are dropped by deterministic clamp.

## 7) Boundedness And Timeboxing Policy (Non-Negotiable)
Per reaction cycle:
1. `primary_calls == 1` strictly.
2. `sub_calls <= N` (`N` configured small, default 2).
3. `repair_calls <= 1`.
4. `attempt_count <= max_attempts`.
5. payload bytes per attempt <= configured cap and endpoint schema cap.
6. cycle time and token consumption <= configured budgets.
7. on bound violation or irreparable compile failure: emit noop (zero attempts), never unbounded retry.

## 8) Always-On Progression Rule
1. CortexReactor is always running.
2. Inbox arrival advances cycles; no external request/response call controls progression.
3. Upstream responsibilities are limited to event delivery and mechanical backpressure handling.
4. Upstream must not inject semantic planning decisions into Cortex.

## 9) Dependency Requirements
1. Reuse existing `ai_gateway` module as inference backend in runtime path.
2. Keep deterministic validation in Rust (schema/caps/catalog checks) independent from model output correctness.
3. Preserve async runtime foundation (`tokio`) for always-on reactor loop.
4. Introduce explicit Cortex-internal ports for:
- primary inference,
- extractor inference,
- filler inference,
- deterministic schema/catalog clamp.

## 10) Migration Strategy
1. Replace old Cortex API as canonical (reactor-only cutover now).
2. Remove/retire old `step`-centric Cortex domain surfaces from public usage.
3. Keep Admission/Continuity/Spine contracts unchanged where possible to reduce blast radius.
4. Add adapter layer from `ReactionResult.attempts` to existing Admission intake path.

## 11) Primary Risks And Mitigations
1. Risk: stateless Cortex loses cross-cycle intent continuity.
- Mitigation: require complete bounded goal/context inputs each cycle; externalize persistence to Continuity/runtime.
2. Risk: LLM pipeline latency breaks always-on loop.
- Mitigation: strict cycle budget and immediate noop fallback on timeout.
3. Risk: sub-LLM introduces intent drift.
- Mitigation: deterministic clamp verifies each attempt aligns with IR-scoped action sketch and catalog.
4. Risk: direct ai_gateway dependency complicates tests.
- Mitigation: all cognition calls through traits; tests use mock implementations.
5. Risk: hard cut from `step` API breaks existing tests/users.
- Mitigation: update tests/contracts in same implementation wave; no mixed API phase.

## 12) L2 Deliverables Expected
L2 should define:
1. exact types for `ReactionInput`, `ReactionResult`, goal context, capability catalog view, and bounded limits.
2. async reactor interface and inbox/attempt stream wiring semantics.
3. primary/sub-LLM port interfaces and ai_gateway-backed adapters.
4. deterministic clamp algorithms (schema, affordance existence, caps, budget checks, unknown drop rules).
5. repair-cycle state machine and noop fallback conditions.
6. contract-test matrix for boundedness, statelessness, routing, and no-intent-drift guarantees.
7. explicit contract fields:
- `IntentAttempt` MUST include `attempt_id` and `based_on: [sense_id...]`.
- feedback path MUST carry `attempt_id` correlation for world realization/denial.

## 13) L1 Exit Criteria
L1 is complete when accepted:
1. Reactor-only API cutover is approved.
2. Real ai_gateway runtime + mock test strategy is approved.
3. Stateless Cortex semantics and distributed-goal ownership are encoded.
4. Boundedness/repair/noop policy is fixed as an architectural invariant.
5. Attempt-world relativity/correlation fields are locked in the contract direction.
6. L2 can proceed without redefining top-level architecture.

Status: `READY_FOR_L2_APPROVAL`
