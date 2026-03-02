# Continuity Contracts

Must hold:
1. Cognition state snapshot/persist operations are deterministic.
2. Persisted cognition state must satisfy goal-forest invariants (id uniqueness, topology validity, numbering validity, weight in `[0,1]`).
3. `on_act` middleware decision contract is `Continue` or `Break`.
4. Continuity scope is cognition persistence and act gating only.
