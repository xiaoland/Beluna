# Product Glossary

- Cortex: Deliberative layer that owns goals, commitments, and attempt planning.
- Continuity: Operational state owner that ingests feedback and builds non-semantic situation views.
- Admission: Mechanical effectuation gate from intent attempts to admitted actions.
- Ledger: Survival budget subsystem that reserves, settles, refunds, expires, and records debits.
- Spine: Control substrate that executes admitted actions and emits ordered feedback.
- Goal: Semantic identity of what cortex wants.
- Commitment: Operational declaration that cortex is actively pursuing a goal.
- IntentAttempt: Non-binding proposal emitted by cortex.
- AdmittedAction: Post-admission executable action accepted by spine.
- AdmissionReport: Per-attempt outcome feedback (`Admitted`, `DeniedHard`, `DeniedEconomic`) with mechanical why fields.
- SituationView: Non-semantic operational snapshot sent from continuity to cortex.
- Survival ledger: Global budget ledger that reserves/settles/refunds/expires action costs and applies external debits.
- Cost attribution id: End-to-end identifier linking attempt -> admitted action -> gateway telemetry -> external debit observation.
- Reservation terminality: Exactly one terminal transition per reservation (`Settled`, `Refunded`, or `Expired`), idempotent by reference.
