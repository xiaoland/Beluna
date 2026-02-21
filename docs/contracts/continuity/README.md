# Continuity Contracts

Must hold:

- cognition state snapshot/persist operations are deterministic.
- capability patch/drop semantics are arrival-order-wins.
- dropped route can be reintroduced by later patch.
- on-act middleware decision contract is `Continue` or `Break`.
