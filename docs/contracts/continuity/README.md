# Continuity Contracts

Must hold:
- cognition state snapshot/persist operations are deterministic.
- capability patch/drop semantics are arrival-order-wins.
- dropped route can be reintroduced by later patch.
- pre-dispatch gate decision contract is `Continue` or `Break`.
- spine event ingestion is deterministic for same input sequence.
