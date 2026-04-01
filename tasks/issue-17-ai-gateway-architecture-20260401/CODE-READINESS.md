# Code Readiness Assessment

## Status

Exploratory.
Broader-mode readiness assessment.
No production code change is implied by this file.

## Purpose

This file answers two practical questions:

1. how far the broader working set has already progressed
2. what still must be clarified before coding is likely to be readable and maintainable

## Progress Summary

The broader working set is no longer at the "vague architecture concern" stage.

It has already established a fairly strong directional contract in four areas:

1. architecture direction
2. public thread-centric chat API direction
3. canonical snapshot/restore direction
4. internal ownership constraints carried forward from strict issue `#17`

The missing consolidation step has now been completed in:

- [CONSOLIDATED-CHAT-CONTRACT-FREEZE.md](/Users/lanzhijiang/Development/Beluna/tasks/issue-17-ai-gateway-architecture-20260401/CONSOLIDATED-CHAT-CONTRACT-FREEZE.md)

The current state is now better described as:

- strong direction
- one active consolidated contract baseline
- ready for implementation slicing

## What Is Already Stable Enough

These conclusions now look stable enough that changing them would require a deliberate reversal,
not a small refinement.

### Architecture

1. `AI Gateway` is a capability runtime, not merely a transport gateway.
2. Top-level organization is capability-first.
3. Shared provider inventory and capability-local bindings are the right config split.
4. `Cortex` should depend on capability contracts rather than provider transport or backend internals.

### Public chat direction

1. `Thread` is the main long-lived write-path object.
2. Ordinary write operations should be thread-centric.
3. Ordinary write input should be `UserMessage`, not arbitrary `Message`.
4. One append call equals one committed turn transaction.
5. `Turn` is read/inspection-facing, not the normal caller write primitive.

### Canonical state

1. Beluna canonical state remains authoritative.
2. Thread-level system prompt is canonical thread state.
3. Committed turns should not contain `SystemMessage`.
4. Committed tool activity should use one canonical representation.
5. Snapshot/restore should expose only committed canonical state.
6. Rewrite/clone-style history surgery should preserve surviving `turn_id` values.

### Internal boundary constraints

1. `attempt` is transport lifecycle language, not chat semantics.
2. provider context must remain explicit and default-deny
3. runtime metadata must not silently become provider context
4. retry semantics must not pretend to be richer than implemented reality

## What Is Directionally Settled But Not Yet Consolidated

These parts are now directionally frozen in the consolidated baseline, but they still require
careful implementation slicing.

### 1. Unified public `Thread` contract

One single source of truth now exists, but implementation still needs discipline around:

- `ThreadSpec`
- `ThreadExecutionDefaults`
- `AppendOptions`
- `AppendMessagesResult`
- `rewrite_context(...)`
- `inspect_turns(...)`
- `snapshot()`

### 2. Snapshot and restore alignment

The main rules are now consolidated, but implementation still needs to preserve:

- exact route field semantics
- exact turn snapshot invariants
- restore validation and failure mapping
- clone-versus-restore semantic boundary

### 3. Error contract alignment

`ChatError` direction is now consolidated, but implementation still needs to preserve:

- final operation taxonomy
- exact `GatewayError` translation
- where backend diagnostics stop
- how invariant failures are distinguished from backend normalization failures

### 4. Clone/rewrite semantics

This used to be the biggest public-surface ambiguity.
The consolidated freeze now resolves it as:

- stable `turn_id`
- no public raw clone-by-storage-surgery primitive
- sibling `derive_context(...)` / `rewrite_context(...)` operations sharing one higher-level context-control request family

Implementation still needs to keep that semantic distinction readable.

## Readiness Bands

## Band A: Ready To Code Now

These code changes can start now with relatively low design risk:

1. behavior-preserving internal cleanup that makes chat visibly capability-owned
2. internal naming cleanup that reduces fake genericity
3. quarantining or trimming dormant retry abstractions that have no real implementation behind them
4. policy guardrails that keep runtime metadata out of any future provider-context path
5. observability cleanup that preserves current semantics while clarifying ownership

Why these are ready:

- they mostly follow already-settled constraints
- they do not require final public API freeze
- they reduce confusion even if broader contract details continue evolving

## Band B: Ready After Implementation Slicing

These areas are now ready in principle, but should still start only after one explicit
implementation slice is chosen:

1. new public thread-centric API surface
2. snapshot / restore public contract
3. `ChatError` public contract
4. clone / rewrite public semantics

What is missing is no longer consolidation.
What is missing is a deliberately narrow first slice.

## Band C: Not Ready To Code Yet

These should not drive implementation now:

1. rich provider-context channel design
2. rich phase-aware retry model tied to streaming/resume
3. speculative future capability scaffolding for `asr` or `tts`
4. exact production migration sequence for all files before the public contract is coherent

These still risk fake abstraction and avoidable rework.

## Distance To Writing Code

The answer depends on which code we mean.

### If the target is internal cleanup only

Distance is short.

It is reasonable to begin now, as long as the slice is explicitly constrained to:

- behavior-preserving
- no public contract freeze implied
- no speculative provider-context or retry overdesign

### If the target is broader public-surface work

Distance is now short-to-moderate.

The task is no longer blocked by missing consolidation.
It is blocked only if the first implementation slice is allowed to become too wide.

In practical terms, it now looks like:

- choose one narrow first slice
- restate invariants and affected files
- then code

## Minimum Pre-Code Checklist For Broader Public-Surface Work

Before coding the broader follow-up design, the task should first commit to the consolidated
baseline and the slice boundary:

1. use one authoritative public `Thread` contract
2. use one authoritative snapshot/restore contract
3. use one authoritative `ChatError` contract
4. keep the frozen derive-versus-rewrite semantics intact

Then, before code starts, restate:

1. target
2. affected files
3. invariants that must remain unchanged
4. which existing tests must keep passing
5. which new tests define the new contract

## Recommended Next Move

Do not jump from the consolidated freeze into a large multi-axis refactor.

That would still lower readability and maintainability.

The next best move is:

1. keep the consolidated freeze as the active baseline
2. pick one first implementation slice against it

## Best First Implementation Slice After Consolidation

Now that one consolidated freeze exists, the safest first broader-mode implementation slice is
likely:

1. introduce the canonical public thread/snapshot/error types without changing provider-context or retry architecture
2. keep adapter/runtime internals mostly stable
3. preserve observability semantics while translating them onto the new public surface

This sequence minimizes simultaneous change across:

- public API
- canonical state model
- transport/retry semantics

and therefore keeps the refactor more readable.
