# TODO.md -> GitHub Issues Migration Checklist

This note converts the current open items in `TODO.md` into a proposed GitHub issue backlog.
The goal is to avoid importing every line as a separate issue when several items are really one decision surface.

## Do Not Migrate As A Product Issue

- `迁移到 Github Issues & Project`
  - This is backlog operations work, not product/runtime work.
  - Treat it as the migration action itself unless you explicitly want one meta issue to track label/project setup.

## Suggested GitHub Setup

### Labels

Area labels:

- `area:core`
- `area:cortex`
- `area:stem`
- `area:continuity`
- `area:spine`
- `area:observability`
- `area:ai-gateway`
- `area:body-endpoint`
- `area:apple-universal`
- `area:docs`

Type labels:

- `type:bug`
- `type:enhancement`
- `type:refactor`
- `type:design`
- `type:observability`
- `type:docs`
- `type:test`
- `type:tech-debt`

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

### 1. Stabilize goal-forest mutation, `reset_context`, and prompt ownership

- Merged from:
  - `goal forest patching 总是不成功`
  - `GoalForest 与 context reset 依然不工作`
  - `goal-forest 不应该重复 system-prompt`
- Description:
  - Make `patch-goal-forest` reliable again, including helper output parsing, runtime application, and `reset_context` thread rewrite behavior.
  - Ensure mutable goal/task state has one owner. The goal forest should carry current goal structure, while the system prompt should carry stable invariants rather than duplicating mutable goal content.
  - Remove any remaining legacy text-based patch behavior so goal mutation happens only through explicit tool contracts.
- Labels: `area:cortex`, `area:ai-gateway`, `type:bug`, `type:design`
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
- Labels: `area:cortex`, `area:stem`, `type:bug`, `type:design`
- Priority: `P0`
- Size: `L`

### 3. Clarify Cortex runtime, Cortex, and Primary authority boundaries

- Merged from:
  - `进一步解除 cortex runtime 和 cortex 与 primary 的边界`
- Description:
  - Make authority boundaries explicit between runtime orchestration, Primary cognition, tool execution, and cognition-state mutation.
  - Reduce incidental coupling so runtime control flow and cognition logic can evolve independently without leaking responsibilities across layers.
- Labels: `area:cortex`, `type:refactor`, `type:design`
- Priority: `P1`
- Size: `L`

### 4. Improve Cortex IR for tool-call-native cognition

- Merged from:
  - `IR 改进`
- Description:
  - Refine the input/output IR so text content stays cognition-oriented and action/state mutation continues through tool calls instead of legacy markup conventions.
  - The IR should make it harder for prompt drift to reintroduce pseudo-XML action channels or duplicated state narration.
- Labels: `area:cortex`, `type:design`, `type:refactor`
- Priority: `P1`
- Size: `L`

### 5. Clarify Spine runtime <-> adapter boundary and move config ownership into adapters

- Merged from:
  - `Spine Runtime 和 Body Endpoint Adapter 之间的交互给我搞清楚咯`
  - `让 adapter 自己处理自己的 config`
- Description:
  - Define the runtime contract between Spine and body endpoint adapters more sharply.
  - Keep adapter-specific config parsing/ownership inside the adapter boundary instead of letting shared runtime code absorb endpoint-specific config concerns.
- Labels: `area:spine`, `area:body-endpoint`, `type:design`, `type:refactor`
- Priority: `P1`
- Size: `L`

### 6. Complete Cortex observability surface with local metrics and structured logs

- Merged from:
  - `Local metrics (cortex-organ-output)`
  - `Cortex 的日志设计还需要优化`
- Description:
  - Add missing local metrics for cortex organ output.
  - Finish the structured log surface so canonical payloads live in event body fields rather than stringified attributes, aligned with the target observability model.
- Labels: `area:observability`, `area:cortex`, `type:observability`
- Priority: `P1`
- Size: `M`

### 7. Simplify AI Gateway adapter abstraction and resilience model

- Merged from:
  - `attempt 是什么鬼`
  - `Tool trait (mainly for o11y)`
  - `retry, budget, relibability 都可以 consolidate 进入 adapter 实现`
- Description:
  - Decide whether `attempt` remains a meaningful cross-backend concept; remove or rename it if it is only accidental transport detail.
  - Consolidate retry, budget, and reliability policy into adapter implementations where that policy actually varies.
  - Add a Tool/adapter seam only if it genuinely improves observability without introducing another unclear abstraction layer.
- Labels: `area:ai-gateway`, `type:refactor`, `type:design`, `type:observability`
- Priority: `P1`
- Size: `L`

### 8. Trim duplicated sense payload fields and relocate instance identifiers to metadata

- Merged from:
  - `sense payload 需要优化，不要重复 metadata 中有的东西`
  - `sense payload 不会携带 uuid 的 act instance id, 带在 metadata`
- Description:
  - Reduce duplicated wire data between metadata and payload on endpoint-originated senses.
  - Remove fields that look like transport leakage or schema confusion, such as payload-local `kind` or duplicated descriptor identity when metadata already owns that responsibility.
  - Keep act instance correlation in metadata instead of payload.
- Labels: `area:body-endpoint`, `area:spine`, `type:refactor`
- Priority: `P1`
- Size: `M`

### 9. Add descriptor descriptions to schema and runtime surfaces

- Merged from:
  - `descriptor 缺少 description 字段`
- Description:
  - Add a human-readable `description` field to descriptor definitions and generated schema/runtime surfaces so tooling, docs, and UI no longer depend on opaque ids alone.
- Labels: `area:core`, `type:enhancement`
- Priority: `P2`
- Size: `S`

### 10. Define Continuity memory taxonomy and retention policy

- Merged from:
  - `被动/主动回忆 与 被动/主动记忆；被动记忆还涉及到 sense 权重；Act其实不用记住，因为 Sense 会回传`
- Description:
  - Separate recall from memory formation, and passive from active paths in both.
  - Define how sense weight affects memory retention.
  - Avoid treating act as first-class memory by default when downstream senses already provide the authoritative feedback path.
- Labels: `area:continuity`, `area:cortex`, `type:design`
- Priority: `P2`
- Size: `L`

### 11. Refactor Apple Universal ChatView and remove embedded core-log observability UI

- Merged from:
  - `需要一个大重构，主要是 ChatView 且移除 core logs o11y`
- Description:
  - Restructure ChatView around clearer view/state boundaries and stop using core log observability as a first-class chat UI surface.
  - Keep user-facing connection/status signals, but separate operator observability from the conversation experience.
- Labels: `area:apple-universal`, `type:refactor`, `type:design`
- Priority: `P2`
- Size: `XL`
- Migration note:
  - This is another good epic-style issue. It will likely want a short design note before implementation.

### 12. Attempt connection on send and surface disconnected state in Apple Universal

- Merged from:
  - `send 的时候尝试连接一次，失败了就告诉 Beluna is disconnected`
- Description:
  - On user send, perform one connect attempt if the app is currently disconnected, and explicitly surface the disconnected result if the attempt fails.
  - Do not let send silently assume an already-valid transport state.
- Labels: `area:apple-universal`, `type:bug`
- Priority: `P2`
- Size: `S`

### 13. Refactor documentation mode and ownership boundaries

- Merged from:
  - `文档模式再重构`
- Description:
  - Revisit the current documentation mode/model so it better matches the layered doc system and promotion rules.
  - The issue should start by clarifying what concrete failure or friction the current mode creates; otherwise it will stay too vague to execute well.
- Labels: `area:docs`, `type:docs`, `type:design`
- Priority: `P3`
- Size: `M`
- Migration note:
  - Open this only if you are willing to write a sharper problem statement than the current TODO wording.

### 14. Move Spine tests out of `src/` and document exceptions

- Merged from:
  - `测试应该在 tests/ 下面，有什么特殊的理由要 aside src 吗？`
- Description:
  - Move non-inline tests into `tests/` by default and document any cases that must remain co-located with implementation.
  - The point is not stylistic purity; it is to make the test layout intentional rather than accidental.
- Labels: `area:spine`, `type:test`, `type:tech-debt`
- Priority: `P3`
- Size: `S`

## Suggested Creation Order

1. Create labels and project fields first.
2. Create all `P0` issues.
3. Create all `P1` issues.
4. Create `P2` issues that already have a concrete execution shape.
5. Leave the vague `P3` doc/test cleanup items for last unless they unblock near-term work.
