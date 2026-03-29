# Coordination Model

## Runtime Coordination Paths (Inside `core`)

1. Afferent ingress
- Producers: body endpoints and internal failure signals.
- Path: bounded pathway with deferral controls.
- Consumer: cognition runtime.

2. Tick coordination
- Producer: tick runtime.
- Consumer: admitted-cycle cognition scheduler.

3. Efferent dispatch
- Producer: cognition act emission.
- Path: ordered efferent pipeline.
- Middleware order: `Continuity.on_act -> Spine.on_act_final`.

4. Physical-state coordination
- Owner: Stem control/store boundary.
- Writers: runtime control surfaces and adapter updates routed through Spine runtime.

5. Cognition-state coordination
- Writer/reader boundary: Cortex + Continuity.
- Continuity persists and guardrails cognition state; Cortex owns transformation logic.

## Cross-Unit Coordination

1. `cli` and `apple-universal` coordinate with `core` through the external endpoint protocol over Unix socket NDJSON.
2. `moira` coordinates with `core` through local artifact/profile selection, bounded process supervision, and OTLP log-consumer boundaries.
3. `moira` does not bypass core runtime authority; local control-plane actions remain bounded to prepare, wake, stop, and inspect flows.
4. External endpoint units do not bypass core authority boundaries.
5. Dispatch terminal outcomes are explicit and flow back to endpoint surfaces.
