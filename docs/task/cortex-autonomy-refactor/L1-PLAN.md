# L1 Plan - Cortex Autonomy Refactor (High-Level Strategy)
- Task Name: `cortex-autonomy-refactor`
- Stage: `L1` (high-level strategy)
- Date: `2026-02-21`
- Status: `REVISED_WITH_L2_ALIGNMENT`

## 0) Locked Decisions
1. Tick cadence default: `1s`.
2. Missed ticks: `skip`.
3. Sleep control is an Act (not a Sense).
4. Stop control is `Sense::Hibernate`.
5. Dispatch chain is middleware-style, per-act: `Continuity on_act -> Spine on_act`.
6. Ledger is temporarily bypassed.
7. Goal-tree patch ops: `sprout` / `prune` / `tilt`.
8. L1-memory is `string[]`; patch ops: `append` / `insert` / `remove`.
9. Root partition is compile-time Rust constant `string[]`.
10. Continuity persistence is direct JSON with guardrails.
11. Goal-tree/L1-memory types are owned by Cortex.
12. Prompts for Primary + Helpers live in one module.

## 1) Architecture Summary
1. Stem runs autonomous ticks (`Active`) and can transition to sleeping mode via sleep act.
2. Cortex pipeline remains helper-based and returns final `acts + new_cognition_state`.
3. Continuity is storage+guardrail for full cognition state persistence.
4. Spine remains terminal act dispatcher; failures are returned as senses via afferent-pathway.

## 2) Boundary Summary
1. Cortex input:
- `senses`
- `physical_state` (act descriptor catalog)
- `cognition_state` (`goal-tree`, `l1-memory`)
2. Cortex output:
- `acts`
- `new_cognition_state`
3. Primary Output IR (internal):
- `acts`
- `goal-tree-patch`
- `l1-memory-patch`

## 3) Communication Model
1. Afferent-pathway: bounded sense queue, single consumer (Stem).
2. Producers to afferent-pathway:
- Body endpoints via Spine adapters
- Continuity (reserved, currently unused)
- Spine (dispatch failure/event senses)
- Main (`Hibernate`)
3. Efferent-pathway: in-process per-act middleware dispatch in Stem.

## 4) Sleep/Hibernate Model
1. `Sense::Hibernate`: immediate loop termination.
2. Sleep act (`core.control/sleep`) with payload `seconds` sets Stem sleeping duration.
3. Sleep act is handled in Stem and not forwarded to Continuity/Spine.

## 5) Continuity Role
1. load/save cognition JSON,
2. validate root partition immutability,
3. validate user tree structure and l1-memory array type,
4. middleware hook `on_act` currently returns `Continue`.

## 6) Prompt Strategy
1. Primary prompt will embed the invariants provided by user (amnesia, dualism, non-performative style, teleology).
2. These invariants are prompt-level instructions, not hard-coded behavior branches.
3. Goal-tree input helper handles only user partition and uses cache.
4. L1-memory is passthrough (no helper).

## 7) Stage Output
This L1 is the architectural baseline for approved L2 package under `docs/task/cortex-autonomy-refactor/`.

