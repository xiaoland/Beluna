# Coordination

## Runtime Coordination Paths

1. Afferent ingress:
- Producers: body endpoints + internal failure signals.
- Path: bounded pathway with deferral controls.
- Consumer: Cortex runtime.

2. Tick coordination:
- Producer: Stem tick runtime.
- Consumer: Cortex runtime admitted-cycle scheduler.

3. Efferent dispatch:
- Producer: Cortex act emission via tools.
- Path: ordered efferent pipeline.
- Middleware order: `Continuity.on_act -> Spine.on_act_final`.

4. Physical-state coordination:
- Owner: Stem control/store.
- Writers: runtime control surfaces and adapter updates routed through Spine runtime.

5. Cognition-state coordination:
- Writer/reader boundary: Cortex + Continuity.
- Continuity provides persistence and guardrails; Cortex owns cognition transformation logic.
