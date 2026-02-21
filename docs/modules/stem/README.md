# Stem Module

Stem is the runtime orchestrator for tick scheduling, sense ingestion, Cortex invocation, and serial act dispatch.

Code:
- `core/src/stem.rs`
- `core/src/afferent_pathway.rs`

Key properties:
- one bounded sense queue with backpressure
- no act queue; inline serial dispatch
- interval/tick-driven loop (default 1s, missed tick skip)
- dispatch stages per act: Continuity -> Spine
- stage decision contract: `Continue` / `Break`
- `Break` affects current act only
- shutdown path gates ingress before blocking hibernate enqueue
- built-in Stem control act: `core.control/sleep` with `seconds`

Communication model:
- Afferent-Pathway:
  - producers: body endpoints (via Spine adapters), Spine failures, Main shutdown, Continuity (reserved)
  - consumer: Stem loop
- Efferent-Pathway:
  - producer: Stem (from Cortex acts)
  - consumers: Continuity middleware then Spine routing

See also:
- [Observability Module](../observability/README.md)
