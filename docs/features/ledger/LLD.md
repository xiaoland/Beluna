# Ledger LLD

State machine:
- `Open -> Settled|Refunded|Expired`

Rules:
- same terminal reference replay is idempotent
- second terminal with different reference is rejected
- expiry uses cycle clock only
