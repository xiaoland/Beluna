# Mind Policies

## Preemption Policy

- allowed dispositions: `pause`, `cancel`, `continue`, `merge`,
- requires safe point snapshot,
- checkpoint token allowed only when preemptable.

## Conflict Policy

Owned conflicts only:

1. helper-output conflicts for same intent,
2. evaluator conflicts for same criterion window,
3. merge compatibility conflicts.

Tie-breaks are deterministic.

## Evolution Policy

- proposal-only (`no_change` or `change_proposal`),
- threshold-based on repeated failures,
- low-confidence failure evidence blocks change proposals.

## Memory Policy

- invoked through `MemoryPolicyPort` every cycle,
- no-op policy is valid in MVP,
- directive trace is maintained in process state.
