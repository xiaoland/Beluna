# Intake Protocol

Beluna treats incoming requests, incidents, and artifacts as perturbations to a living decision network.

This protocol governs how to classify input, contain volatility, and promote only stable truth.

## 1. Typed Input Taxonomy

Classify every non-trivial perturbation before planning:

- `Intent`: new behavior request, feature wish, policy request, UX direction.
- `Constraint`: budget/platform/team/performance limits or stack constraints.
- `Reality`: bug, runtime incident, user failure report, metric anomaly.
- `Artifact`: code snippet, schema, logs, screenshots, draft docs.

If input is mixed, mark the primary type and list secondary types in the task packet.

## 2. Intake Steps

1. Capture perturbation:
- record raw signal and hard constraints without early promotion.

2. Localize impact:
- identify primary hit layer, likely secondary hits, confidence, and unknowns.

3. Contain in task packet:
- create/update `docs/task/<task>/` before changing durable docs or code.

4. Resolve conflicts when needed:
- escalate if new input conflicts with durable anchors, changes user-visible behavior, or creates irreversible trade-offs.

5. Execute under governance:
- implement with governing anchors and explicit acceptance criteria.

6. Verify:
- validate intent, design consistency, behavior, and operational safety.

7. Decide outcome:
- choose one: `promote`, `complete_without_promotion`, `defer`, `reject`, `experiment`.

8. Promote stable truth:
- promote only recurring and stable conclusions into `10/20/30/40` layers.

## 3. Task Packet Fields (Non-Trivial Tasks)

Use this structure in task plans/results:

- `Perturbation`
- `Input Type` (`Intent` / `Constraint` / `Reality` / `Artifact`)
- `Governing Anchors`
- `Intended Change`
- `Impact Hypothesis` (primary hit, secondary hits, confidence, unknowns)
- `Temporary Assumptions`
- `Negotiation Triggers`
- `Acceptance Criteria`
- `Guardrails Touched`
- `Evidence Expected`
- `Outcome`
- `Promotion Candidates`

## 4. Promotion Boundary

- Task files are transient and non-authoritative.
- Stable truths belong in exactly one authoritative layer.
- Executable guardrails (tests/schemas/checks) should enforce promoted contracts.
