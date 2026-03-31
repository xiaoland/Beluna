# TODO.md -> GitHub Issues Migration Checklist

This note converts the current open items in `TODO.md` into a proposed GitHub issue backlog.
The goal is to avoid importing every line as a separate issue when several items are really one decision surface.

## Do Not Migrate As A Product Issue

- `迁移到 Github Issues & Project`
  - This is backlog operations work, not product/runtime work.
  - Treat it as the migration action itself unless you explicitly want one meta issue to track label/project setup.

## Suggested GitHub Setup

### Labels

Ownership labels:

- `unit:core`
- `unit:cli`
- `unit:apple-universal`
- `unit:monitor`
- `unit:docs`

Cross-cutting labels:

- `ui/ux`
- `o11y`

Simple type labels:

- `type:bug`
- `type:task`
- `type:feature`

Implementation note:

- Because this repository is under a personal account, do not optimize for GitHub organization-only issue type features.
- Keep type classification as labels for now.
- If the repository later moves under an organization with issue types enabled, `type:*` labels can be mapped to native `Type` with low migration cost.

### Project Fields

- `Priority`
  - `P0`: correctness/runtime stability risk; can cause invalid cognition state, runaway loops, or broken core workflows.
  - `P1`: important next; strong leverage on architecture clarity, operability, or wire-contract quality.
  - `P2`: meaningful backlog item; should land after current correctness/architecture blockers.
  - `P3`: cleanup/documentation debt; useful but not urgent.
- `Size`
  - `S`: up to about 1 day.
  - `M`: about 1 to 3 days.
  - `L`: about 3 to 7 days.
  - `XL`: likely multi-PR or requires staged design/implementation.

## Proposed Issues

### Removed As Outdated

- `Improve Cortex IR for tool-call-native cognition`
- `Refactor Apple Universal ChatView and remove embedded core-log observability UI`
- `Attempt connection on send and surface disconnected state in Apple Universal`
- `Refactor documentation mode and ownership boundaries`

These items are intentionally not migrated in the current issue set after the latest backlog sync.

### 1. Stabilize goal-forest mutation, `reset_context`, and prompt ownership

- Merged from:
  - `goal forest patching 总是不成功`
  - `GoalForest 与 context reset 依然不工作`
  - `goal-forest 不应该重复 system-prompt`
- Description:
  - Make `patch-goal-forest` reliable again, including helper output parsing, runtime application, and `reset_context` thread rewrite behavior.
  - Ensure mutable goal/task state has one owner. The goal forest should carry current goal structure, while the system prompt should carry stable invariants rather than duplicating mutable goal content.
  - Remove any remaining legacy text-based patch behavior so goal mutation happens only through explicit tool contracts.
- Labels: `unit:core`, `type:bug`
- Priority: `P0`
- Size: `XL`
- Migration note:
  - Best tracked as one issue or epic, but likely implemented in at least two PRs: reliability/reset first, prompt-ownership cleanup second.

### 2. Stop idle self-talk loops and correct internal self-model inputs

- Merged from:
  - `Cortex 当前的设计存在系统性的问题，导致 spinning in an idle self-talk loop`
  - `act dispatch 情况不是本体感的一部分（但 afferent/efferent pathway 的负载水平可以是）`
- Description:
  - Prevent Primary from spinning on self-generated cognition without fresh external input.
  - Tighten the internal self-model so act dispatch outcomes are not treated as part of body-self or agency state.
  - If internal load signals are needed, prefer bounded pathway-level signals such as afferent/efferent load or queue pressure rather than dispatch-result narration.
- Labels: `unit:core`, `type:bug`
- Priority: `P0`
- Size: `L`

### 3. Clarify Cortex runtime, Cortex, and Primary authority boundaries

- Merged from:
  - `进一步解除 cortex runtime 和 cortex 与 primary 的边界`
- Description:
  - Make authority boundaries explicit between runtime orchestration, Primary cognition, tool execution, and cognition-state mutation.
  - Reduce incidental coupling so runtime control flow and cognition logic can evolve independently without leaking responsibilities across layers.
- Labels: `unit:core`, `type:task`
- Priority: `P1`
- Size: `L`

### 4. Clarify Spine runtime <-> adapter boundary and move config ownership into adapters

- Merged from:
  - `Spine Runtime 和 Body Endpoint Adapter 之间的交互给我搞清楚咯`
  - `让 adapter 自己处理自己的 config`
- Description:
  - Define the runtime contract between Spine and body endpoint adapters more sharply.
  - Keep adapter-specific config parsing/ownership inside the adapter boundary instead of letting shared runtime code absorb endpoint-specific config concerns.
- Labels: `unit:core`, `type:task`
- Priority: `P1`
- Size: `L`

### 5. Complete Cortex observability surface with local metrics and structured logs

- Merged from:
  - `Local metrics (cortex-organ-output)`
  - `Cortex 的日志设计还需要优化`
- Description:
  - Add missing local metrics for cortex organ output.
  - Finish the structured log surface so canonical payloads live in event body fields rather than stringified attributes, aligned with the target observability model.
- Labels: `unit:core`, `o11y`, `type:task`
- Priority: `P1`
- Size: `M`

### 6. Simplify AI Gateway adapter abstraction and resilience model

- Merged from:
  - `attempt 是什么鬼`
  - `Tool trait (mainly for o11y)`
  - `retry, budget, relibability 都可以 consolidate 进入 adapter 实现`
- Description:
  - Decide whether `attempt` remains a meaningful cross-backend concept; remove or rename it if it is only accidental transport detail.
  - Consolidate retry, budget, and reliability policy into adapter implementations where that policy actually varies.
  - Add a Tool/adapter seam only if it genuinely improves observability without introducing another unclear abstraction layer.
- Labels: `unit:core`, `o11y`, `type:task`
- Priority: `P1`
- Size: `L`

### 7. Normalize sense payload and descriptor schema for LLM-readable contracts

- Merged from:
  - `sense payload 需要优化，不要重复 metadata 中有的东西`
  - `sense payload 不会携带 uuid 的 act instance id, 带在 metadata`
  - `descriptor 缺少 description 字段`
- Description:
  - Clean up descriptor and sense payload contracts so each piece of information appears once at the correct layer.
  - Reduce duplicated wire data between metadata and payload on endpoint-originated senses, and keep correlation fields such as `act_instance_id` in metadata rather than payload.
  - Add a human-readable `description` field to descriptor definitions and generated schema/runtime surfaces primarily so LLMs and prompt/tooling can consume descriptor intent without relying on opaque ids alone.
  - Remove fields that look like transport leakage or schema confusion, such as payload-local `kind` or duplicated descriptor identity when metadata already owns that responsibility.
- Labels: `unit:core`, `type:task`
- Priority: `P1`
- Size: `M`
- Migration note:
  - The issue is intentionally merged because payload cleanup and descriptor semantics are one contract surface. If implementation starts coupling unrelated changes, split by PR rather than by issue.

### 8. Define Continuity memory taxonomy and retention policy

- Merged from:
  - `被动/主动回忆 与 被动/主动记忆；被动记忆还涉及到 sense 权重；Act其实不用记住，因为 Sense 会回传`
- Description:
  - Separate recall from memory formation, and passive from active paths in both.
  - Define how sense weight affects memory retention.
  - Avoid treating act as first-class memory by default when downstream senses already provide the authoritative feedback path.
- Labels: `unit:core`, `type:task`
- Priority: `P2`
- Size: `L`

### 9. Move Spine tests out of `src/` and document exceptions

- Merged from:
  - `测试应该在 tests/ 下面，有什么特殊的理由要 aside src 吗？`
- Description:
  - Move non-inline tests into `tests/` by default and document any cases that must remain co-located with implementation.
  - The point is not stylistic purity; it is to make the test layout intentional rather than accidental.
- Labels: `unit:core`, `type:task`
- Priority: `P3`
- Size: `S`

## Suggested Creation Order

1. Create labels and project fields first.
2. Create all `P0` issues.
3. Create all `P1` issues.
4. Create `P2` issues that already have a concrete execution shape.
5. Leave the `P3` cleanup item for last unless it starts blocking refactors.
