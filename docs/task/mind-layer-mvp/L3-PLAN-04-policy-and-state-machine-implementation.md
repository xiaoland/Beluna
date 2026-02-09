# L3-04 - Policy And State Machine Implementation

- Task Name: `mind-layer-mvp`
- Stage: `L3` detail: policy/state-machine implementation
- Date: 2026-02-09
- Status: `DRAFT_FOR_APPROVAL`

## 1) Preemption Decision Procedure

Input:

- active goal (optional)
- incoming goal
- safe point snapshot
- recent reliability judgments for active goal

Ordered rules:

1. if no active goal: `Continue` (incoming will be activated by base flow)
2. if `safe_point.preemptable == false`: `Continue`
3. if merge-compatible(active, incoming): `Merge`
4. if incoming priority > active priority: `Pause`
5. if active reliability has repeated `Fail` across threshold window: `Cancel`
6. otherwise: `Continue`

Output requirements:

- include rationale string,
- include safe point,
- include `merge_goal_id` only for `Merge`.

## 2) Safe Point / Checkpoint Rules

1. safe point is computed only via `SafePointPolicy`.
2. `checkpoint_token` can be `Some` only when `preemptable` is true.
3. checkpoint token generation for tests must be deterministic.
4. safe point data is attached to preemption decision and can be copied into intent metadata.

## 3) Merge Compatibility Predicate

`merge-compatible(active, incoming)` returns true only if all hold:

1. same goal level or adjacent hierarchy level,
2. same parent scope (or both roots with equivalent intent namespace),
3. neither goal is terminal (`Completed/Cancelled/Merged`),
4. combined metadata does not produce key conflicts without deterministic preference.

Merge construction:

1. merged goal id = deterministic composition from sorted source ids,
2. merged goal priority = max(source priorities),
3. source goals set to `Merged`, merged goal set to `Active`.

## 4) Conflict Resolver Ownership Map

Resolver owns only three conflict classes:

1. helper outputs for same intent,
2. evaluator verdicts for same criterion window,
3. merge compatibility disputes.

Resolver tie-breaks:

1. helper conflict:
- highest confidence,
- then lexical `helper_id`.

2. evaluator conflict:
- conservative verdict ranking: `Fail > Borderline > Unknown > Pass`,
- then highest confidence.

3. merge conflict:
- deterministic compatibility predicate,
- no heuristic randomness.

## 5) Evolution Trigger Procedure (Proposal-only)

Input:

- current/active goal context,
- recent evaluation window,
- latest memory directive

Algorithm:

1. derive failure pattern vector:
- alignment failures,
- reliability failures,
- faithfulness failures,
- memory mismatch indicators.

2. apply threshold:
- if persistent pattern length < threshold: `NoChange`.

3. map target:
- reliability dominant -> `Model` or `PerceptionPipeline`,
- faithfulness dominant -> `PerceptionPipeline`,
- memory mismatch dominant -> `MemoryStructure`.

4. map action:
- config drift -> `Reconfigure`,
- persistent deficit -> `Replace`,
- explicit data insufficiency -> `Retrain`.

5. emit `ChangeProposal` with rationale, confidence, evidence refs.

No direct execution path is implemented.

## 6) State Machine Guardrails

1. single-active-goal invariant validated:
- after any goal lifecycle mutation,
- before output emission.

2. terminal goal statuses are immutable:
- `Completed`, `Cancelled`, `Merged` cannot return to `Active`.

3. all confidence fields are clamped into `[0.0, 1.0]`.

4. cycle id monotonic increment per step.

## 7) Determinism Guardrails

1. use `BTreeMap`/stable ordering for all map iteration paths.
2. sort conflict cases before resolution.
3. avoid wall-clock/time/random usage in core decisions.
4. make token generators injectable or fixed in tests.

Status: `READY_FOR_L3_REVIEW`
