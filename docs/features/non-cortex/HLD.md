# Non-Cortex HLD

## Pipeline

1. Sort attempts deterministically by `attempt_id`.
2. Resolve affordance profile.
3. Evaluate hard constraints.
4. Evaluate economic affordability.
5. Reserve budget and materialize admitted action or deny with code.
6. Dispatch admitted actions to Spine.
7. Reconcile ordered spine events into ledger settlements.
8. Ingest external debit observations by attribution.

## Components

- `AdmissionResolver`
- `SurvivalLedger`
- `NonCortexFacade`
- `ExternalDebitSourcePort`
