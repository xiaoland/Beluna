# L0 Plan - Cortex MVP
- Task Name: `cortex-mvp`
- Stage: `L0` (request + context analysis only)
- Date: `2026-02-11`
- Status: `DRAFT_FOR_APPROVAL`

## 1) Problem Deconstruction
The requested Cortex MVP changes the Cortex boundary from deterministic command-step planning to an always-on, bounded, asynchronous cognition loop.

Locked requirements from user input:
1. Cortex is a reactor loop, not request/response.
- Consumes inbox stream continuously.
- Emits attempt stream continuously.
- Does not block waiting for world completion.
2. `IntentAttempt` is non-binding and world-addressed.
- Cortex proposes.
- Admission/world realizes or denies.
3. LLM cognition is two-stage.
- Primary LLM outputs prose IR (intent, attention, action sketches).
- Sub-LLM organ(s) compile IR into structured attempts.
- Deterministic validator/clamp is final authority and may drop unknown/unsafe output.
4. Endpoint routing is capability-catalog driven.
- Runtime provides capability catalog.
- Routing is a Cortex cognition concern, not Non-Cortex.
5. Cortex input contract must be bounded and delta-oriented.
- Ordered `Sense` window with stable `sense_id`.
- Truncated endpoint-context snapshots.
- Prior admission outcome codes (non-semantic).
- Limits/budgets and capability catalog.
6. Hard boundedness/timeboxing per reaction cycle.
- Exactly 1 primary LLM call.
- At most `N` subcalls (small, 1-2).
- Strict `max_attempts`, payload caps, time/token budgets.
- At most 1 repair attempt then safe fallback/noop.

## 2) Context Collection (Sub-agent Style)
To reduce cognitive load without a dedicated sub-agent runtime, context was split into parallel tracks:
1. Track A: local codebase and contracts scan (`core/src/*`, `core/tests/*`, `docs/features/*`, `docs/contracts/*`).
2. Track B: external architecture references (Firecrawl) on reactor bounded-stream design and schema-constrained LLM output pipelines.

## 3) Current Codebase Reality
### 3.1 Cortex is command-step and stateful today
- `core/src/cortex/facade.rs` exposes `CortexFacade::step(CortexCommand) -> CortexCycleOutput`.
- Cortex currently owns long-lived `CortexState` (goals, commitments, reports, attempt journal).
- Planning is synchronous and immediate inside `step`.

### 3.2 Current planner is deterministic but not LLM-driven
- `core/src/cortex/planner.rs` resolves affordance/capability via simple rules or `GoalDecomposerPort`.
- No primary IR stage, no extractor/filler sub-LLM stages, no compile/repair loop.
- No explicit deterministic clamp stage that validates against runtime endpoint schemas/catalog.

### 3.3 Runtime loop does not run Cortex yet
- `core/src/server.rs` currently handles only socket lifecycle + `exit` message parsing.
- No end-to-end `Sense -> Cortex react -> Admission` runtime wiring in server path.

### 3.4 Non-Cortex boundary already enforces world realization
- Admission/Continuity/Spine already treat attempts as proposals and enforce realization mechanically.
- This aligns with requirement #2 and should be preserved.

### 3.5 Input contract mismatch
- Current Cortex input is `CortexCommand`, not bounded `Sense` delta + context snapshot + admission outcome codes + budgets + capability catalog.
- No stable `sense_id` type in Cortex input contract.

### 3.6 Boundedness mismatch
- Current planner emits one attempt per active commitment with no explicit per-cycle max-attempt cap.
- No explicit primary/subcall budget accounting and no repair-at-most-once rule.

## 4) Fit-Gap Matrix Against Requested MVP
1. Always-on reactor loop: `GAP`
2. Non-binding world-addressed attempts: `PARTIAL` (semantics present, loop model missing)
3. Primary IR + sub-LLM compiler pipeline + deterministic clamp: `GAP`
4. Capability-catalog-driven routing inside Cortex: `GAP` (routing is static heuristic today)
5. Bounded delta-oriented stateless input contract: `GAP`
6. Hard cycle budgets + one-repair cap: `GAP`

## 5) External Source Findings (Firecrawl, primary references)
1. Reactor bounded-stream guidance (Project Reactor docs)
- Source: `https://projectreactor.io/docs/core/release/reference/coreFeatures/sinks.html`
- Relevant finding: unicast/multicast sinks support backpressure and bounded-queue rejection semantics instead of unbounded buffering.
- Design implication: Cortex inbox/attempt streams should use explicit bounded channels and deterministic overflow policy (drop/reject/degrade), not unbounded queues.

2. Structured output and schema adherence (OpenAI docs)
- Source: `https://platform.openai.com/docs/guides/structured-outputs`
- Relevant finding: structured outputs enforce schema adherence; function-calling is preferred when model output drives tool/system actions.
- Design implication: use primary prose IR for intent expression, then enforce structured compile stage with schema targets and deterministic validation clamp before emitting attempts.

3. JSON Schema specification baseline
- Source: `https://json-schema.org/specification`
- Relevant finding: schema validation is formally split into core/validation vocabularies with stable meta-schema references.
- Design implication: endpoint payload clamp should be driven by explicit schema validation and deterministic truncation/error handling rules.

## 6) Architectural Trade-offs Identified
1. Stateful Cortex vs stateless reactor contract
- Keeping current stateful Cortex simplifies migration but violates "Cortex stays stateless via bounded input contract."
- Moving state responsibility outward aligns with requirement but requires significant API/test changes.

2. Real AI Gateway integration now vs LLM port abstraction first
- Direct integration gives immediate realism but increases implementation and reliability complexity.
- Port abstraction enables deterministic tests and phased backend rollout.

3. Where routing logic lives
- Moving routing into Cortex cognition aligns with requirement #4.
- Keeping routing in Admission would violate intent/cognition ownership and blur boundaries.

4. Boundedness enforcement location
- Enforcing only at admission is too late; cycle limits must be enforced inside Cortex before emitting attempts.
- Dual-layer enforcement (Cortex pre-emit + Admission hard gate) is safest.

5. Output strategy on compiler/clamp failure
- Failing the cycle hard is simple but destabilizes always-on loop.
- Emitting safe noop/safe-message fallback with one repair attempt preserves liveness under strict bounds.

## 7) L0 Recommendation For L1 Strategy
For L1, target a new Cortex reactor surface while preserving Non-Cortex semantics:
1. Add a new `react`-style Cortex API and types without immediately deleting existing `step` API.
2. Introduce bounded `ReactionInput` contract (sense window + context snapshot + prior outcome codes + budgets + capability catalog).
3. Introduce cognition pipeline ports:
- `PrimaryReasonerPort` (prose IR),
- `AttemptExtractorPort` (IR -> draft attempts),
- `PayloadFillerPort` (optional schema fill),
- deterministic `AttemptClamp` (final authority).
4. Enforce per-cycle hard caps (primary=1, subcalls<=N, max_attempts, payload limits, budget/time caps, repair<=1).
5. Emit attempts as non-binding stream items and keep realization with Admission/Continuity/Spine.
6. Add contract tests around boundedness, routing correctness from capability catalog, and repair fallback behavior.

## 8) Open Questions Requiring User Decision
1. API migration scope
- Should `CortexFacade::step` remain temporarily for compatibility, or can we cut directly to reactor-only API in this task?
2. LLM integration depth
- Do you want real `ai_gateway` calls in MVP, or deterministic mockable ports with adapter wiring in a follow-up?
3. Statelessness strictness
- Should goal/commitment state be fully externalized from Cortex in this MVP, or can limited internal cache remain while input contract is implemented?
4. Safe fallback semantics
- For "fallback to noop / safe message", should emitted fallback be:
  - zero attempts, or
  - one explicit `IntentAttempt` targeting a safe affordance key (e.g. `observe.state`)?

## 9) Working Assumptions (If Not Overridden)
1. Keep existing `step` path temporarily and add new reactor path.
2. Implement LLM cognition via ports first (testable), with optional AI Gateway adapter.
3. Treat statelessness as contract-first: reactor path consumes complete bounded input each cycle.
4. Default fallback behavior: emit zero attempts (noop) after one failed repair.

## 10) L0 Exit Criteria
L0 is complete when:
1. requirements are decomposed into enforceable technical constraints,
2. current implementation fit-gaps are explicit,
3. trade-offs and migration risks are documented,
4. external references support key design constraints,
5. blocking decisions for L1 are listed.

Status: `READY_FOR_L1_APPROVAL`
