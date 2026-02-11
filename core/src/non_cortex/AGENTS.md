# AGENTS.md for core/src/non_cortex

Non-cortex is the runtime physics+economics substrate.

## Invariants
- Admission is deterministic and semantic-free.
- Intent attempts are non-binding; only admitted actions can execute.
- Reservation lifecycle is strict: `open -> settled|refunded|expired`.
- External debits apply only when attribution matches.
- Version tuple is carried in ledger entries for upgrade-stable determinism.
