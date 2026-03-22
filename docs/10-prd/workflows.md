# Primary Workflows

## Runtime Workflow

1. Body endpoints and internal producers emit senses into afferent pathway.
2. Stem emits tick grants and owns physical-state mutation.
3. Cortex consumes admitted ticks and buffered senses, then runs one cognition cycle.
4. Act tools dispatch through ordered efferent path: `Continuity -> Spine`.
5. Continuity persists cognition state and enforces guardrails.

## Inference Workflow

1. Core/Cortex opens or reuses AI Gateway thread context.
2. Gateway validates turn invariants and selects one backend deterministically.
3. Adapter streams canonical events.
4. Tool/result bundles are applied deterministically by thread/turn contracts.

## Endpoint Workflow

1. Endpoint authenticates and registers capability descriptors.
2. Endpoint emits domain senses with descriptor identity.
3. Spine routes acts by endpoint key and returns final dispatch status.
