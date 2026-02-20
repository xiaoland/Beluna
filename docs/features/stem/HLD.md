# Stem Runtime HLD

## Pipeline

1. Receive next sense from bounded queue.
2. Handle control senses (`sleep`, `new_capabilities`, `drop_capabilities`).
3. Compose `PhysicalState` (ledger snapshot + merged capabilities).
4. Load `CognitionState` snapshot from Continuity.
5. Invoke Cortex.
6. Persist returned cognition state.
7. Dispatch acts serially through:
   - Ledger pre-dispatch
   - Continuity gate
   - Spine dispatch
8. Settle ledger and notify continuity with resulting spine event.

## Components

- `SenseIngress` (gate + bounded sender wrapper)
- `StemRuntime`
- `LedgerStage`
- `ContinuityEngine`
- `SpineExecutorPort`
