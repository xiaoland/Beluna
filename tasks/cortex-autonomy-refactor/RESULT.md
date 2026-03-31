# Cortex Autonomy Refactor Result

Date: 2026-02-21
Task: ruthlessly refactor Cortex/Stem/Continuity toward tick-driven autonomy, goal-tree+l1-memory cognition state, and middleware dispatch.

## Implemented

1. Tick-driven Stem loop
- Stem loop now runs on interval ticks (`loop.tick_interval_ms`, default 1000ms).
- Missed ticks use `skip` behavior.
- Stem can run Cortex with empty domain senses.

2. Control semantics
- Removed sleep control sense.
- Added `Sense::Hibernate` and shutdown path uses `send_hibernate_blocking()`.
- Added Stem-provided control act descriptor `core.control/sleep` with payload `{seconds}`.
- Sleep act switches Stem into timed sleeping mode and supports early wake on new sense.

3. Cognition model pivot
- Replaced old goal-stack model with `goal-tree` + `l1-memory`.
- `ROOT_PARTITION` is compile-time constant string array in Cortex.
- `GoalNode` is unified node type for user partition tree.
- L1 memory is ordered `string[]`.
- Patch ops:
  - goal-tree: `sprout | prune | tilt`
  - l1-memory: `append | insert | remove`

4. Cortex boundary and IR
- Runtime boundary remains:
  - input: `senses + physical_state + cognition_state`
  - output: `acts + new_cognition_state`
- Primary Output IR is separated from runtime `CortexOutput`.
- Input IR sections:
  - `<senses>`
  - `<act-descriptor-catalog>`
  - `<goal-tree>`
  - `<l1-memory>`
- Output IR sections:
  - `<acts>`
  - `<goal-tree-patch>`
  - `<l1-memory-patch>`
- Added `goal_tree_helper` input helper with cache on user partition only.
- `l1-memory` input is passthrough ordered list.
- Helper JSON Schema outputs are direct arrays (no wrapper duplication).
- Cortex applies patch arrays internally and returns full `new_cognition_state`.

5. Dispatch chain refactor
- Removed Continuity `pre_dispatch` and `on_spine_event` contracts.
- Per-act middleware chain is:
  - `Continuity.on_act -> Spine.on_act`
- Middleware contract is `Continue | Break`.
- Spine reject/error paths emit failure senses via afferent sender.
- Continuity is store + guardrail and currently returns deterministic `Continue`.

6. Continuity persistence and guardrails
- Added JSON persistence (`core/src/continuity/persistence.rs`) with atomic write pattern.
- Continuity loads/saves cognition state from configured path.
- Guardrails enforce:
  - root partition immutable match
  - user root id is `user-root`
  - unique goal node ids
- Continuity now holds afferent-pathway sender handle (reserved for future sense emission).

7. Config and schema updates
- Added `continuity.state_path`.
- Added `loop.tick_interval_ms` and `loop.tick_missed_behavior`.
- Updated Cortex helper routes:
  - removed `goal_stack_helper`
  - added `goal_tree_helper`, `goal_tree_patch_helper`, `l1_memory_patch_helper`
- Updated `core/beluna.schema.json` accordingly.

8. Prompt/module organization
- Consolidated primary/helper prompt builders in `core/src/cortex/prompts.rs`.
- Primary prompt updated to reflect discrete-tick amnesia, mind-body split, non-performative output, and proactive change-oriented closure.

9. Documentation updates
- Updated feature docs:
  - `docs/features/cortex/*`
  - `docs/features/continuity/*`
  - `docs/features/stem/*`
- Updated module docs:
  - `docs/modules/cortex/README.md`
  - `docs/modules/continuity/README.md`
  - `docs/modules/stem/README.md`
  - `docs/modules/spine/README.md`
- Updated communication/overview docs:
  - `docs/overview.md`
  - `docs/contracts/stem/README.md`
- Updated AGENTS docs:
  - `core/src/cortex/AGENTS.md`
  - `core/src/continuity/AGENTS.md`
  - `core/src/spine/AGENTS.md`
  - `core/AGENTS.md`

## Validation

- Build command: `cargo build` (in `core/`)
- Result: pass

## Notes

- Ledger module remains in repository but is currently short-circuited from Stem dispatch path.
- Existing legacy tests under `core/tests` were not migrated in this task.
