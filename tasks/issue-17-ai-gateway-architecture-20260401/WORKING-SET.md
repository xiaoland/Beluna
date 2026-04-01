# Broader Working Set Index

## Status

Exploratory.
User-selected broader mode.
This task is now continuing beyond strict issue `#17` scope on purpose.

## Why this file exists

The current task folder contains two different layers of work:

1. strict issue-17 internal ownership clarification
2. broader follow-up design for public chat/config/snapshot contracts

That overlap is useful, but only if the active frontier is explicit.
Without one index, the working set becomes hard to read and easy to misapply.

## Operating Rule

For this task, the active direction is now the broader follow-up track.

That means:

- public chat capability contract redesign is back in scope for discussion
- config shape redesign is back in scope for discussion
- snapshot / restore / error surface design is back in scope for discussion

But these strict-issue-17 conclusions still constrain the broader work:

- transport and adapter terms must not leak upward without a real semantic need
- provider-context inheritance must stay explicit and default-deny
- retry semantics must not pretend to be richer than implemented reality
- observability must follow real ownership boundaries

## Active Files By Role

### Primary broader-mode notes

- `CONSOLIDATED-CHAT-CONTRACT-FREEZE.md`
  - active contract baseline
  - merges public chat/config/snapshot/error direction
  - includes observability enhancement direction

- `LOW-LEVEL-DESIGN.md`
  - architecture north star
  - capability-first structure
  - dependency direction
  - shared-vs-capability-local ownership

- `CONFIG-AND-CHAT-CONTRACT.md`
  - primary public chat/config contract draft
  - thread-centric OOP surface
  - append/rewrite/snapshot direction
  - route and binding shape

- `ERROR-AND-SNAPSHOT-CONTRACT.md`
  - public error and snapshot freeze proposal
  - canonical restore boundary
  - turn/thread snapshot invariants

- `FOUR-QUESTION-FREEZE.md`
  - low-level invariants that broader public contracts must respect

### Supporting boundary notes

- `ADAPTER-CONTRACT-BOUNDARY.md`
  - minimal internal adapter/runtime seam
  - useful constraint source, but not the active main artifact

- `PROVIDER-CONTEXT-AND-RETRY-GROUNDING.md`
  - code-grounded correction for provider-context and retry assumptions
  - prevents follow-up design from overbuilding fake genericity

### Navigation / historical notes

- `PLAN.md`
  - original exploration baseline

- `SCOPE-REALIGNMENT.md`
  - explains why strict issue-17 and follow-up must be distinguished

- `STATUS.md`
  - current summary and next-step routing

- `MIGRATION-MAP.md`
  - implementation-oriented move map
  - useful only after contract direction is clearer

- `CODE-READINESS.md`
  - progress summary
  - coding readiness bands
  - minimum pre-code checklist

- `CONSOLIDATED-FREEZE-BLOCKERS.md`
  - exact reasons the final freeze is not yet honest
  - minimum decisions needed before emitting it

## Current Broader-Mode Core Conclusions

These are the conclusions that now appear stable enough to treat as the active working baseline.

1. `AI Gateway` is an AI capability runtime, not merely a provider transport gateway.
2. Top-level organization is capability-first, not backend-first.
3. Shared provider inventory and capability-local bindings/config are the right split.
4. `Cortex` should depend on capability contracts, not gateway internals or provider transport.
5. `Thread` is the main long-lived OOP write-path object.
6. Ordinary write-path operations should be thread-centric: `append(...)`, `append_message(...)`, `append_messages(...)`.
7. Ordinary write-path input should be `UserMessage`, not arbitrary `Message`.
8. One append call equals one committed turn transaction.
9. `Turn` remains an internal semantic unit plus read/inspection surface, not the normal caller write primitive.
10. Committed tool activity should use one canonical representation: explicit `ToolCallMessage` plus `ToolCallResultMessage`.
11. Thread-level system prompt is canonical thread state and should not be duplicated as committed `SystemMessage` turns.
12. Snapshot/restore should expose only committed Beluna-canonical state.
13. Rewrite/clone-style context surgery should preserve surviving `turn_id` values rather than densely reindexing them.

## Important Constraints Carried Forward From The Internal Boundary Notes

These are not the broader-mode headline, but they remain hard constraints:

1. `attempt` is transport lifecycle language, not chat semantics.
2. `TurnPayload.metadata` is currently runtime metadata, not provider context.
3. Provider-context inheritance is not a real implemented channel today and must stay explicit if added later.
4. Current retry safety is only trustworthy for whole-request replay before canonical commit.
5. Clone lineage belongs in chat/runtime semantics and observability, not adapter transport abstractions.

## Active Frontier

The broader-mode frontier is now narrower than "design everything."

It should focus on integrating the existing drafts into one coherent public chat contract.

### Frontier A: Unified public thread contract

Need to reconcile and freeze in one place:

- `ThreadSpec`
- `ThreadExecutionDefaults`
- `AppendOptions`
- `AppendMessagesResult`
- `rewrite_context(...)`
- `inspect_turns(...)`
- `snapshot()`

### Frontier B: Canonical snapshot and restore semantics

Need to make fully consistent across notes:

- stable route representation
- stable turn identity
- no system messages in committed turns
- no partial continuation state
- restore validation failure mapping

### Frontier C: Error contract alignment

Need to confirm:

- `ChatErrorKind`
- operation taxonomy
- translation from `GatewayError`
- backend diagnostics attachment boundary

### Frontier D: Clone/rewrite lineage semantics

Need to define without degrading readability:

- how derived threads are represented semantically
- whether clone remains a public operation or is subsumed by `rewrite_context(...)` plus snapshot/restore
- what observability fields carry lineage
- what is lineage only versus canonical state

## Things That Are Still Too Early To Freeze

1. Future `asr/` and `tts/` public contracts.
2. Exact folder/file migration sequence in production code.
3. Rich provider-context channel design.
4. Rich phase-aware retry contract unless streaming/resume becomes real.
5. Any abstraction added only for naming symmetry.

## Recommended Next Step

Do not widen the working set further.

Use [CONSOLIDATED-CHAT-CONTRACT-FREEZE.md](/Users/lanzhijiang/Development/Beluna/tasks/issue-17-ai-gateway-architecture-20260401/CONSOLIDATED-CHAT-CONTRACT-FREEZE.md) as the active contract baseline, then choose the first implementation slice against it.
