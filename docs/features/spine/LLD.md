# Spine LLD

## Contract Rules

- Action identifiers are stable strings.
- `seq_no` ordering is the single source of reconciliation order.
- Transport arrival order is ignored by non-cortex reconciliation.

## MVP Implementation

- Deterministic noop spine used for baseline behavior and tests.
- Contract is ready for body endpoint integrations without changing non-cortex ledger invariants.
