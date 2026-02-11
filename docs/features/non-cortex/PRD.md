# Non-Cortex PRD

## Purpose

Non-cortex is Beluna's physics + economics substrate.

It admits or denies `IntentAttempt[]` mechanically, executes budget reservations, reconciles settlements, and persists continuity independent of cortex internals.

## Requirements

- No semantic intent classification.
- Admission outcomes are explicit:
  - `Admitted { degraded: bool }`
  - `DeniedHard { code }`
  - `DeniedEconomic { code }`
- Reservation terminality is strict (`settle|refund|expire` exactly one).
- External debits require attribution-chain match and reference dedupe.
- Version tuple is included for deterministic replay across upgrades.
