# Evaluation Contract

## Scope

Defines normative evaluation outputs for Mind.

## Criteria

- `goal_alignment`
- `subsystem_reliability`
- `signal_faithfulness`

## Contract Cases

1. Given active goal context
- When evaluation runs
- Then a goal-alignment judgment is emitted.

2. Given missing signal evidence
- When evaluation runs
- Then signal-faithfulness may be `unknown`.

3. Given any non-pass verdict
- When evaluation is emitted
- Then rationale must be non-empty.

## Output Requirements

- Evaluation output is structured (`EvaluationReport` with `Judgment[]`).
- Judgments include criterion, verdict, confidence, rationale, and evidence refs.
- Confidence is clamped to `[0.0, 1.0]`.
