# Mind Execution Flow

Per `MindFacade::step`:

1. receive `MindCommand`,
2. update `MindState` base effects,
3. run preemption for competing goals,
4. plan delegation through port,
5. evaluate normative judgments,
6. resolve scoped conflicts,
7. run memory policy decision,
8. run evolution decision,
9. emit `MindCycleOutput`.

Output includes:

- typed events (`MindEvent`),
- typed decisions (`MindDecision`),
- cycle id.
