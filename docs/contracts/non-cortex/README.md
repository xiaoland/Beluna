# Non-Cortex Contracts

Boundary:
- input: `IntentAttempt[]`
- output: `AdmissionReport` and admitted action dispatch to Spine

Must hold:
- deterministic, semantic-free admission
- complete disposition set
- strict reservation terminality and idempotency
- attribution-matched external debit only
- versioned policy tuple in ledger audit entries
