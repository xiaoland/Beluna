# Mind PRD

## Product Statement

Mind is Beluna's meta-level control system.
It orchestrates goals, evaluates runtime behavior, coordinates helper delegation, and emits controlled evolution proposals.

## MVP Scope

- Strict internal module boundary (`src/mind/*`).
- No Unix socket/protocol interaction.
- One active goal at a time with layered goals (high/mid/low).
- Explicit preemption dispositions (`pause`, `cancel`, `continue`, `merge`).
- Normative evaluation across alignment/reliability/faithfulness.
- Trait-based delegation and memory policy ports.
- Proposal-only evolution decisions (`no_change` or `change_proposal`).

## User Stories

- As Beluna runtime, I need one active-goal authority so behavior is coherent.
- As Beluna runtime, I need explicit preemption decisions so goal switching is auditable.
- As Beluna runtime, I need deterministic conflict resolution so helper disagreement does not cause nondeterminism.
- As Beluna runtime, I need proposal-only evolution output so unsafe automatic mutation is avoided in MVP.

## Functional Requirements

1. Goal management
- keep multiple goals in state,
- enforce single active goal,
- support register/activate/pause/cancel/merge lifecycle transitions.

2. Preemption
- run preemption when new goal competes with active goal,
- output one disposition from closed set,
- attach safe point and optional checkpoint token.

3. Evaluation
- emit criterion-based judgments with rationale/evidence.

4. Delegation and conflict handling
- plan delegation only through trait port,
- resolve owned conflict classes deterministically.

5. Memory policy boundary
- invoke `MemoryPolicyPort` each cycle,
- keep decision trace in-process in MVP.

6. Evolution
- emit proposal-only evolution decision using thresholded failure patterns.

## Non-Goals (MVP)

- No direct integration with Unix socket runtime.
- No persistent memory store.
- No helper-process execution runtime.
- No automatic execution of evolution actions.

## Acceptance Criteria

- single-active-goal invariant holds after each step,
- preemption disposition is always one of pause/cancel/continue/merge,
- checkpoint token is invalid when safe point is non-preemptable,
- deterministic loop order is maintained,
- evolution remains proposal-only.

## Glossary

- MindState: in-process continuity state for Mind cycles.
- GoalManager: invariant-enforcing goal lifecycle controller.
- Safe Point: preemptability snapshot with optional checkpoint token.
- Preemption Disposition: one of pause/cancel/continue/merge.
- MemoryPolicyPort: trait boundary for remember/forget policy decisions.
