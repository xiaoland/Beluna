# Status And Next Steps

## Current Stage

The discussion has clearly moved from high-level architecture into early low-level design.

However, after re-reading GitHub issue `#17`, one correction is necessary:

- the task has drifted into both internal-refactor design and public-surface redesign
- issue `#17` only clearly authorizes the first track

So the current stage is:

- internal ownership and observability clarification for issue `#17`
- plus a separate pool of follow-up contract ideas that should not be mistaken for the current issue target

User-selected continuation has now moved the active working direction onto that broader follow-up track.

This means:

- strict issue-17 notes still matter as constraints
- but public contract and config follow-up notes are now active working material rather than demoted side notes

Current state:

- high-level architectural direction is mostly settled
- there is now a clear scope correction between issue-17 work and follow-up design work
- several low-level contract drafts exist, but some of them exceed issue-17 scope
- config and migration shape are still partially unsettled
- freeze 3 and freeze 4 now have a code-grounded narrowing: provider context is not a real
  runtime path yet, and current retry semantics are narrower than the dormant abstraction hooks
  suggest

See:

- [SCOPE-REALIGNMENT.md](/Users/lanzhijiang/Development/Beluna/tasks/issue-17-ai-gateway-architecture-20260401/SCOPE-REALIGNMENT.md)
- [WORKING-SET.md](/Users/lanzhijiang/Development/Beluna/tasks/issue-17-ai-gateway-architecture-20260401/WORKING-SET.md)

## What Is Already Settled

Important scope note:

- the high-level ownership conclusions below still matter for issue `#17`
- some mid-level and low-level items below are exploratory north-star notes, not current issue-17 freeze targets
- config-shape redesign and public chat-contract redesign should be treated as follow-up unless they can stay strictly internal

### High-level

1. `AI Gateway` is an AI capability runtime / SDK-like subsystem.
2. Beluna canonical authority remains with Beluna, not provider-native hidden state.
3. `Cortex` retains orchestration authority.
4. AI Gateway is multi-capability in concept, even though only chat exists today.
5. Internal organization is capability-first, not backend-first.
6. Capabilities and backends are added only when actually needed.

### Mid-level

1. Shared provider inventory is accepted.
2. Capability-local binding config is accepted.
3. Global canonical route syntax is accepted as `<capability>.<alias>`.
4. `Thread / Turn / Message` remains the canonical chat abstraction stack.
5. No extra `ChatDialogue` naming layer is needed right now.
6. `rewrite_context(...)` should be a closed semantic rewrite API, not arbitrary message surgery.

### Low-level bias now in force

1. `Thread` is the main OOP object seen by `Cortex`.
2. `Turn` remains a semantic internal turn-level object plus read-side projection.
3. `Message` should be constructible and inspectable by callers.
4. Direct provider/model syntax is not the canonical route language.

## What Is Still Not Fully Settled

Important scope note:

- the rest of this section contains valuable low-level design thinking
- however, public-surface and external-schema changes described here are not automatically in scope for issue `#17`
- use these notes as future-direction material unless the same outcome can be achieved behind the existing chat capability surface

### Main resolved trade-off

`Turn` is no longer treated as a caller-facing mutable write object.

Accepted reason:

- thread-centric write API is cleaner
- turn invariants stay inside runtime ownership
- callers avoid awareness of internal turn bookkeeping on the write path

Accepted constraint:

- turn still remains visible on the read/inspection path
- hiding it completely would hurt observability and debugging

### Newly settled low-level rules

`Thread` ordinary append input should be narrowed to `UserMessage`, not generic `Message`.

Consequences:

- system mutation goes through thread creation or `rewrite_context(...)`
- assistant/tool_call/tool_result stay runtime-owned on the normal path
- thread write semantics become sharply bounded and easier to reason about
- the API surface becomes honest at the type level instead of relying on runtime rejection

Additional refinement:

- there is no separate public `submit_turn(...)`
- one append call corresponds to one committed internal turn transaction
- `append_message(...)` and `append_messages(...)` are sugar over a thread-centric append request with optional per-append execution options
- there should be no long-lived public open-turn state between calls
- `TurnInput` should not remain the public contract name

### Newly settled result contract

`append(...)` / `append_messages(...)` return a thin result envelope:

- `new_messages: Vec<Message>`
- `finish_reason: Option<FinishReason>`
- `usage: Option<UsageStats>`

This is now preferred over:

- returning `Turn`
- returning only a bare `Vec<Message>`

### Newly settled runtime ownership rule

- there should be no public `pending_tool_call_continuation` protocol leaked to `Cortex`
- `Thread` owns internal orchestration required to finish a tool cycle
- `Turn` owns turn-local invariants and may reject incomplete tool linkage
- if a tool cycle cannot be completed, the append operation should fail rather than commit partial canonical history

### Newly settled canonical history rule

- committed tool activity should use one canonical representation
- explicit `ToolCallMessage` / `ToolCallResultMessage` are preferred over a mixed dual representation
- provider-native embedded tool-call payloads belong in backend normalization, not Beluna canonical history

### Newly settled snapshot / rewrite rule

- snapshots should export only committed canonical turns
- snapshots should not expose half-complete continuation state
- `rewrite_context(...)` should preserve surviving `turn_id` values
- clone/reset style history surgery should not reindex committed turns

### Strict issue-17 open points retained as constraints

These are still useful constraints from the strict issue-17 reading, even though the active
working direction has moved onto the broader follow-up track:

1. define the minimal adapter/runtime contract behind the existing chat surface
2. define which metadata may participate in inherited provider-thread context and which must remain runtime-only
3. define the ownership split for shared reliability policy versus adapter-local retry/budget/reliability details
4. clarify clone lineage and derived-thread observability semantics without requiring public API expansion
5. trim the migration map to behavior-preserving internal ownership cleanup first

## Artifacts Produced So Far

1. High-level exploration:
- [PLAN.md](/Users/lanzhijiang/Development/Beluna/tasks/issue-17-ai-gateway-architecture-20260401/PLAN.md)

2. Low-level architecture:
- [LOW-LEVEL-DESIGN.md](/Users/lanzhijiang/Development/Beluna/tasks/issue-17-ai-gateway-architecture-20260401/LOW-LEVEL-DESIGN.md)

3. Config and chat contract refinement:
- [CONFIG-AND-CHAT-CONTRACT.md](/Users/lanzhijiang/Development/Beluna/tasks/issue-17-ai-gateway-architecture-20260401/CONFIG-AND-CHAT-CONTRACT.md)

4. File migration map draft:
- [MIGRATION-MAP.md](/Users/lanzhijiang/Development/Beluna/tasks/issue-17-ai-gateway-architecture-20260401/MIGRATION-MAP.md)

5. Chat error and snapshot freeze proposal:
- [ERROR-AND-SNAPSHOT-CONTRACT.md](/Users/lanzhijiang/Development/Beluna/tasks/issue-17-ai-gateway-architecture-20260401/ERROR-AND-SNAPSHOT-CONTRACT.md)

6. Scope correction after re-reading the issue:
- [SCOPE-REALIGNMENT.md](/Users/lanzhijiang/Development/Beluna/tasks/issue-17-ai-gateway-architecture-20260401/SCOPE-REALIGNMENT.md)

7. Minimal in-scope adapter boundary draft:
- [ADAPTER-CONTRACT-BOUNDARY.md](/Users/lanzhijiang/Development/Beluna/tasks/issue-17-ai-gateway-architecture-20260401/ADAPTER-CONTRACT-BOUNDARY.md)

8. Four-question freeze with issue-14 constraints:
- [FOUR-QUESTION-FREEZE.md](/Users/lanzhijiang/Development/Beluna/tasks/issue-17-ai-gateway-architecture-20260401/FOUR-QUESTION-FREEZE.md)

9. Code-grounded refinement for provider-context and retry semantics:
- [PROVIDER-CONTEXT-AND-RETRY-GROUNDING.md](/Users/lanzhijiang/Development/Beluna/tasks/issue-17-ai-gateway-architecture-20260401/PROVIDER-CONTEXT-AND-RETRY-GROUNDING.md)

10. Broader-mode working-set index:
- [WORKING-SET.md](/Users/lanzhijiang/Development/Beluna/tasks/issue-17-ai-gateway-architecture-20260401/WORKING-SET.md)

11. Broader-mode coding readiness assessment:
- [CODE-READINESS.md](/Users/lanzhijiang/Development/Beluna/tasks/issue-17-ai-gateway-architecture-20260401/CODE-READINESS.md)

12. Consolidated-freeze blocker note:
- [CONSOLIDATED-FREEZE-BLOCKERS.md](/Users/lanzhijiang/Development/Beluna/tasks/issue-17-ai-gateway-architecture-20260401/CONSOLIDATED-FREEZE-BLOCKERS.md)

13. Active broader-mode consolidated contract freeze:
- [CONSOLIDATED-CHAT-CONTRACT-FREEZE.md](/Users/lanzhijiang/Development/Beluna/tasks/issue-17-ai-gateway-architecture-20260401/CONSOLIDATED-CHAT-CONTRACT-FREEZE.md)

## Recommended Next Steps

The user-selected direction is now the broader working set, not strict issue-17-only discipline.

Use [WORKING-SET.md](/Users/lanzhijiang/Development/Beluna/tasks/issue-17-ai-gateway-architecture-20260401/WORKING-SET.md) as the active index for that mode.

The broader-mode public contract now has one active consolidated baseline:

- [CONSOLIDATED-CHAT-CONTRACT-FREEZE.md](/Users/lanzhijiang/Development/Beluna/tasks/issue-17-ai-gateway-architecture-20260401/CONSOLIDATED-CHAT-CONTRACT-FREEZE.md)

Use [PROVIDER-CONTEXT-AND-RETRY-GROUNDING.md](/Users/lanzhijiang/Development/Beluna/tasks/issue-17-ai-gateway-architecture-20260401/PROVIDER-CONTEXT-AND-RETRY-GROUNDING.md) as the code-grounded companion to freeze 3 and freeze 4.

That note narrows two important conclusions:

- provider context is currently absent by implementation, and should stay explicit rather than be
  smuggled through metadata
- current retry safety is only trustworthy for full-request replay before canonical commit, not
  for a richer post-output or post-tool model

### Step 1

Treat [CONSOLIDATED-CHAT-CONTRACT-FREEZE.md](/Users/lanzhijiang/Development/Beluna/tasks/issue-17-ai-gateway-architecture-20260401/CONSOLIDATED-CHAT-CONTRACT-FREEZE.md) as the public contract baseline.

### Step 2

Pick the first implementation slice against that baseline.

Recommended bias:

- start with public thread/snapshot/error types and lineage-aware observability
- avoid simultaneous provider-context or retry-architecture redesign

### Step 3

Only after the first slice is stabilized, return to migration sequencing and deeper internal cleanup.
