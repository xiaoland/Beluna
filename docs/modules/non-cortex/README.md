# Stem Runtime Module

Stem is the runtime orchestrator for sense ingestion, cortex invocation, and serial act dispatch.

Code:
- `core/src/stem.rs`
- `core/src/ingress.rs`
- `core/src/runtime_types.rs`

Key properties:
- one bounded sense queue with backpressure
- no act queue; inline serial dispatch
- dispatch stages: Ledger -> Continuity -> Spine
- stage decision contract: `Continue` / `Break`
- `Break` affects current act only
- shutdown path gates ingress before blocking sleep enqueue
