# L0 Plan 01 - Runtime Skeleton and AppContext DI
- Task: `cortex-loop-architecture`
- Micro-task: `01-runtime-skeleton-and-app-context-di`
- Stage: `L0`
- Date: `2026-03-01`
- Status: `DRAFT_FOR_APPROVAL`

## Objective
Reshape runtime boot/control topology so Cortex runs independently while Stem keeps clock authority, using Dependency Injection instead of global singletons.

## Scope
1. Define `stem::afferent_pathway` and `stem::efferent_pathway` as Stem-owned modules.
2. Stem runtime instantiates pathways and returns handles.
3. Build `AppContext` in `main()` as composition root for runtime instances/handles.
4. `main()` starts Cortex runtime thread/task.
5. Enforce invariant: Stem must not call `cortex.cortex(...)` directly.
6. Pass narrow dependency bundles/ports to modules rather than full `AppContext`.

## Current State
1. `main()` instantiates afferent pathway directly.
2. Stem owns outer loop and directly invokes Cortex.
3. No explicit efferent pathway module exists.

## Target State
1. Stem is clock authority and pathway factory.
2. Cortex has independent hybrid runtime thread/task.
3. Runtime ownership is DI-wired by `AppContext` without global static pathway singletons.
4. Modules consume focused dependency sets.

## Key Gaps
1. Boot sequence redesign needed in `main` and `stem`.
2. New handle interfaces needed between Stem and Cortex runtimes.
3. `AppContext` composition and shutdown order must be explicitly defined.

## Risks
1. AppContext can become a god object if passed wholesale.
2. Misordered lifecycle shutdown can still deadlock if dependency graph is unclear.

## L0 Exit Criteria
1. New runtime ownership graph is explicit.
2. No direct Stem->Cortex invocation path remains in design.
3. AppContext lifecycle states are defined (init, running, closing, closed).
4. Narrow dependency bundle boundaries are listed.
