# L1 Plan - Mind Layer MVP (High-Level Strategy)

- Task Name: `mind-layer-mvp`
- Stage: `L1` (high-level strategy)
- Date: 2026-02-08
- Status: `DRAFT_FOR_APPROVAL`

## 0) Inputs Locked From L0 Approval

User decisions applied:

1. Mind does not interact with Unix socket directly and should never do so.
2. MVP evolution mode is proposal-only (Body components are not ready).
3. Helpers are trait-based for now, and helper implementation is not part of Mind.
4. Goal preemption is decided by Mind through explicit dispositions constrained by policy:
   - `pause`
   - `cancel`
   - `continue`
   - `merge`
5. Delivery must prioritize change isolation and low coupling for fast iteration.

## 1) Strategy Summary

Implement a standalone `mind` core module as a strict meta-control boundary that can be embedded by runtime layers later.

High-level strategy:

- Introduce `src/mind/*` as a pure control-domain module.
- Keep Mind independent from socket protocol, server loop, and AI Gateway internals.
- Represent goals, evaluations, delegation outcomes, preemption decisions, and evolution proposals as strict typed domain objects.
- Use dependency inversion: Mind depends on trait ports, not concrete helper/runtime implementations.
- Enforce deterministic decision policies in core state transitions.

This keeps Mind evolvable in isolation while preserving Beluna's strict protocol philosophy.

## 2) Target High-Level Architecture

```text
Runtime Integrator (future)
   -> MindFacade (application service)
      -> MindState (in-process continuity state)
      -> GoalManager + Active Goal Invariant
      -> SafePointPolicy (preemptability + optional checkpoint token)
      -> PreemptionDecider (policy-constrained)
      -> NormativeEvaluator (goal alignment / reliability / signal faithfulness)
      -> DelegationCoordinatorPort (trait only; external impl)
      -> MemoryPolicyPort (trait, no-op allowed in MVP)
      -> ConflictResolver (deterministic, scoped ownership)
      -> EvolutionDecider (proposal-only)
      -> MindDecision/Event Output (typed)
```

Boundary principle:

- Mind is a control kernel.
- Execution backends (socket, helper processes, model calls, memory engines) stay outside Mind and enter only through ports.

## 3) Core Technical Decisions

1. Hard boundary from transport/runtime
- `src/mind/*` must not import `src/server.rs`, `src/protocol.rs`, or Unix socket abstractions.
- No direct NDJSON protocol logic in Mind.

2. Layered internal organization for isolation
- `domain`: goal graph/state, judgments, proposals, invariants.
- `policy`: preemption and evolution decision rules.
- `application`: orchestration flow that calls policy + ports.
- `ports`: helper/evidence/memory policy interfaces.

3. Explicit continuity state container
- Mind must own typed in-process continuity state (`MindState`) for iterative reasoning across turns/cycles.
- `MindFacade` orchestrates transitions but must not become the implicit state bag.
- `MindState` minimally includes active-goal pointer, goal lifecycle map, pending intents, and recent judgment/delegation snapshots.

4. Goal model with single-active invariant
- Multiple goals may exist in backlog/graph.
- At most one goal is active at any time.
- State transitions are explicit and validated before commit.

5. Preemption is first-class decision
- On goal competition, Mind runs `PreemptionDecision`.
- Output must be one of four dispositions: `Pause`, `Cancel`, `Continue`, `Merge`.
- Policy constraints can disallow dispositions for specific goal states.
- Decision output carries safe-point metadata:
  - `preemptable: bool` (minimal preemptability signal)
  - optional `checkpoint_token` for resumable continuation.

6. Normative evaluation model
- Evaluation outputs structured judgments, not just scalar scores.
- Each judgment includes explicit criterion, verdict, and rationale/evidence summary.
- Initial criteria families:
  - goal alignment
  - subsystem reliability
  - signal faithfulness

7. Delegation via ports only
- Mind can request helper work through a trait contract.
- Mind receives helper outputs as typed observations.
- Mind owns aggregation and conflict resolution; helpers do not own final authority.

8. Memory policy via dedicated port
- Add `MemoryPolicyPort` even in MVP; default implementation can be explicit no-op.
- Mind uses this port for remember/forget policy decisions so this concern does not leak into evaluator or resolver logic.

9. Deterministic conflict resolution with scoped ownership
- ConflictResolver owns only:
  - helper-output conflicts for the same subtask/intent,
  - evaluator verdict conflicts over the same criterion window,
  - preemption compatibility conflicts for `Merge` candidates.
- ConflictResolver does not own goal selection policy or evolution triggering policy.
- Define stable tie-break ordering and merge/reject behavior.
- Same inputs must produce same resolution result.

10. Proposal-only self-evolution
- Evolution output is a decision artifact:
  - `NoChange`
  - `ChangeProposal` (`replace`, `retrain`, `reconfigure`)
- No mutation/execution of Body components in MVP.

11. Minimal integration surface
- Expose a compact Mind API for future runtime adapters.
- Keep API focused on input snapshot + decision/evolution outputs.

12. Deterministic Mind loop in facade
- `MindFacade` must implement this deterministic sequence:
  1. ingest input (`MindCommand`)
  2. update `MindState`
  3. if new goal exists, run `PreemptionDecider` and produce disposition (`Pause`/`Cancel`/`Continue`/`Merge`)
  4. plan/delegate via `DelegationCoordinatorPort` if required
  5. evaluate results with `NormativeEvaluator`
  6. resolve owned conflicts with `ConflictResolver`
  7. emit typed outputs (`MindEvent`, `MindDecision`)

## 4) Coupling-Control Plan (Fast Iteration Requirement)

To maintain change isolation and low coupling:

1. Dependency direction rule
- `mind` depends only on std + local `mind` modules.
- No dependency from Mind core to runtime/socket/ai_gateway modules.

2. Port isolation rule
- Any external action (helper delegation, evidence retrieval) must be trait-mediated.
- Adapter implementations live outside core logic.

3. State isolation rule
- Mind state lives in an explicit `MindState` type and is manipulated through invariant-preserving methods only.
- No external mutable access to active-goal pointer.

4. Decision isolation rule
- Preemption/evolution policies are pure and testable functions.
- Policy changes should not require transport/runtime changes.

5. Test isolation rule
- Unit tests focus on Mind invariants and policy determinism using test doubles.
- No socket/process/network dependency in Mind tests.
- Include loop-determinism tests to ensure identical command/state inputs produce identical decisions/events.

## 5) Dependency Requirements (High-Level)

MVP dependency strategy:

- Prefer no new runtime dependencies for Mind core.
- Reuse existing project primitives (`anyhow` only where boundary wrapping is required; domain core should favor typed errors).
- Keep async/runtime dependencies out of policy/domain where possible.

## 6) Risks and Mitigations

1. Ambiguous preemption semantics
- Risk: `Merge` can become underspecified and destabilize behavior.
- Mitigation: require explicit merge compatibility checks and deterministic fallback behavior.

2. Hidden continuity state in facade
- Risk: orchestration code turns into implicit state store with unclear invariants.
- Mitigation: enforce explicit `MindState` domain type and transition APIs.

3. Over-coupling to helper implementation details
- Risk: Mind leaks process/model/tool specifics.
- Mitigation: strict trait contracts and opaque helper payload wrappers.

4. Evaluation drift
- Risk: judgments become inconsistent across cycles.
- Mitigation: explicit criterion registry and deterministic evaluation schema.

5. Premature evolution execution
- Risk: unsafe direct changes to runtime components.
- Mitigation: enforce proposal-only output type in MVP.

## 7) Deliverables Expected from L2

L2 should define:

- exact module/file map in `src/mind/*`,
- core trait interfaces and boundaries,
- domain data model for `MindState`, goals, safe points, judgments, delegation results, proposals,
- preemption decision algorithm and disposition constraints,
- deterministic loop algorithm in `MindFacade`,
- deterministic conflict-resolution policy with explicit conflict ownership,
- `MemoryPolicyPort` contract and no-op MVP adapter,
- evolution trigger criteria and proposal schema,
- test matrix covering invariants and policy behaviors,
- documentation/contract file map under `docs/features`, `docs/contracts`, and `docs/modules`.

## 8) L1 Exit Criteria

L1 is complete when the following are accepted:

- Mind remains fully decoupled from Unix socket runtime.
- Preemption disposition model (`pause/cancel/continue/merge`) is accepted.
- Proposal-only evolution strategy is accepted.
- Trait-based helper boundary is accepted.
- Isolation/low-coupling architecture rules are accepted.

Status: `READY_FOR_L2_APPROVAL`
