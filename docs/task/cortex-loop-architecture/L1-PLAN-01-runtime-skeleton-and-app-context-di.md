# L1 Plan 01 - Runtime Skeleton and AppContext DI
- Task: `cortex-loop-architecture`
- Micro-task: `01-runtime-skeleton-and-app-context-di`
- Stage: `L1`
- Date: `2026-03-01`
- Status: `DRAFT_FOR_APPROVAL`

## High-Level Strategy
1. Split runtime execution into two independent tasks.
- Stem runtime owns tick generation and pathway instantiation.
- Cortex runtime owns hybrid loop execution and sense consumption.
2. Move all startup wiring into a single composition root (`main()` + `AppContext`).
3. Replace direct `Stem -> Cortex::cortex(...)` call path with channel/port based runtime contracts.
4. Preserve clock authority in Stem by sending tick events to Cortex through a clock port, not function invocation.

## Architectural Design
1. `main()` composes `AppContext` with:
- runtime shutdown primitives
- stem-built pathway handles
- continuity/spine/cortex service handles
- shared physical state lock handle.
2. Stem exposes a pathway/runtime bootstrap surface that returns handles.
3. Cortex runtime receives:
- afferent consumer handle
- tick subscription handle
- efferent producer handle
- continuity persistence port
- physical state reader.
4. Module dependencies are one-way:
- `main` wires all.
- `stem` provides clocks/pathways/state mutation.
- `cortex` consumes inputs and emits acts.

## Key Technical Decisions
1. `AppContext` is wiring-only and passed only at bootstrap time.
2. Each runtime receives a narrow dependency bundle (`StemDeps`, `CortexDeps`) to avoid god-object coupling.
3. Lifecycle states are explicit (`init`, `running`, `closing`, `closed`) and shared across runtimes.
4. Shutdown order is deterministic:
- close ingress
- stop new ticks
- drain inflight work
- terminate runtimes.

## Dependency Requirements
1. Existing Stem dispatch logic must be isolated so it can be moved to efferent pathway later.
2. A stable handle contract is required before micro-tasks `03`, `04`, `07`.
3. Build-time breakage due to runtime split is expected and accepted during migration.

## L1 Exit Criteria
1. New runtime topology is explicit and DI-first.
2. No design path allows Stem to invoke Cortex directly.
3. `AppContext` scope and anti-coupling guardrails are defined.
