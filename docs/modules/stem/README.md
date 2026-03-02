# Stem Module

Stem is the runtime substrate for ticks, pathway ownership, and physical-state mutation.

Code:
- `core/src/stem.rs`
- `core/src/stem/runtime.rs`
- `core/src/stem/afferent_pathway.rs`
- `core/src/stem/efferent_pathway.rs`

Key properties:
1. Stem runtime does not invoke Cortex.
2. Stem emits tick grants consumed by Cortex runtime.
3. Stem owns canonical `Arc<RwLock<PhysicalState>>` writer path via `StemControlPort`.
4. Afferent pathway is Stem-owned and consumed by Cortex runtime.
5. Efferent pathway is Stem-owned FIFO and consumed serially as `Continuity -> Spine`.
6. Shutdown supports bounded efferent drain timeout.

Communication model:
1. Afferent producers: Spine adapters, Spine dispatch-failure emitter, other runtime producers.
2. Afferent consumer: Cortex runtime.
3. Efferent producer: Cortex runtime.
4. Efferent consumer pipeline: Continuity then Spine.
