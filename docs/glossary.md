# Product Glossary

- Cortex: Deliberative layer that owns goals, commitments, and attempt planning.
- Continuity: Operational state owner that ingests feedback and builds non-semantic situation views.
- Admission: Mechanical effectuation gate from intent attempts to admitted actions.
- Ledger: Survival budget subsystem that reserves, settles, refunds, expires, and records debits.
- Spine: Transport-ignorant execution substrate that routes admitted actions by table lookup and emits ordered feedback.
- Sense: Canonical Body Endpoint -> Spine/Cortex ingress datum representing newly sensed world input.
- Affordance Key: Semantic action kind (`what`) used in routing and policy.
- Capability Handle: Concrete executable channel (`which`) implementing an affordance.
- Route Key: Composite (`affordance_key`, `capability_handle`) routing identity in Spine.
- Goal: Semantic identity of what Cortex wants.
- Commitment: Operational declaration that Cortex is actively pursuing a goal.
- IntentAttempt: Non-binding proposal emitted by Cortex.
- AdmittedAction: Post-admission executable action accepted by Spine.
- AdmissionReport: Per-attempt outcome feedback (`Admitted`, `DeniedHard`, `DeniedEconomic`) with mechanical why fields.
- SituationView: Non-semantic operational snapshot sent from Continuity to Cortex.
- Survival Ledger: Global budget ledger that reserves/settles/refunds/expires action costs and applies external debits.
- Cost Attribution Id: End-to-end identifier linking attempt -> admitted action -> gateway telemetry -> external debit observation.
- Reservation Terminality: Exactly one terminal transition per reservation (`Settled`, `Refunded`, or `Expired`), idempotent by reference.
