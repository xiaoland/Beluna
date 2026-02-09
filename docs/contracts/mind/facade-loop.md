# Facade Loop Contract

## Scope

Defines deterministic ordering guarantees for `MindFacade::step`.

## Required Loop Order

1. ingest command,
2. update `MindState`,
3. run preemption for new-goal competition,
4. delegation planning,
5. evaluation,
6. conflict resolution,
7. memory policy decision,
8. evolution decision,
9. emit typed output.

## Contract Cases

1. Given same input and same starting state
- When `step` executes
- Then output is identical.

2. Given active-goal competition on new goal
- When `step` executes
- Then preemption decision is emitted before delegation plan.

3. Given evaluation and conflicts in the same cycle
- When `step` executes
- Then conflict decision is emitted before evolution decision.

4. Given no-op ports
- When `step` executes
- Then loop completes with no external side effects.
