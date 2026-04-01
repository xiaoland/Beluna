# Scope Realignment After Re-reading Issue #17

## Why this file exists

After re-reading GitHub issue `#17`, the task note set is clearly carrying two different tracks:

1. issue-17 internal refactor work
2. possible future public chat-contract redesign

Those are not the same thing.
Keeping them mixed would make the discussion less readable and would weaken the issue boundary.

Issue `#17` explicitly says:

- keep the existing chat capability surface stable
- improve internal ownership clarity
- avoid turning this work into a prerequisite for Cortex decomposition issue `#14`

Therefore this task needs a scope guard.

## What Issue #17 Actually Requires

The issue is about clarifying internal ownership inside AI Gateway while preserving the current consumer-facing chat surface.

Concretely, the issue asks for:

- clearer adapter/runtime lifecycle and ownership semantics
- clearer inherited-context behavior where the current capability already supports it
- prevention of runtime-only control metadata leaking into provider-thread context
- explicit adapter-local ownership when retry / budget / reliability are truly backend-specific
- removal or renaming of `attempt` if it is not a real cross-backend concept
- observability seams only when they clarify real ownership and lineage

## In Scope For Issue #17

The following lines of work still fit the issue:

### 1. Adapter contract clarification behind the current chat surface

Keep the current `Chat` / `Thread` / `TurnInput` surface intact while tightening:

- what the adapter receives
- what the runtime owns before and after a backend call
- which fields are transport-only versus capability-owned

### 2. `attempt` as a transport/request concept

The current first-principles conclusion still looks valid:

- `attempt` is meaningful only inside one backend request lifecycle
- it belongs in capability-neutral transport observability
- it should not become a chat/thread/turn concept

Issue `#17` may keep it, rename it, or remove it, but only at that layer.

### 3. Retry / budget / reliability ownership split

The useful part of the earlier analysis is still valid:

- shared gateway execution policy should remain shared when it is genuinely cross-backend
- adapter-local policy should exist only where backend-specific behavior actually differs

This means the issue is not "move all reliability into adapters".
It is "stop pretending backend-specific policy is generic when it is not."

### 4. Inherited provider-thread context boundary

Issue `#17` explicitly cares about this.
The main requirement is:

- runtime-only control metadata must not accidentally become persistent provider-thread context

Examples that should stay non-canonical unless proven otherwise:

- `tick`
- `request_id`
- `parent_span_id`
- temporary retry or budgeting control state
- caller-only operational metadata

### 5. Clone semantics and observability lineage

Clone lineage is in scope because the issue explicitly mentions parent vs derived operations.

The important question is not "should the public clone API disappear?"
The important question is:

- how to represent clone lineage and derived-thread provenance clearly in observability
- how to clarify ownership of selected-turn inheritance
- how to avoid ambiguous persistence semantics

### 6. Behavior-preserving internal file/module cleanup

Module and file moves are in scope only when they are behavior-preserving and reduce ownership drift.

That includes:

- separating capability-neutral transport/execution helpers from chat-specific runtime code
- reducing generic names that are actually chat-specific internally

## Out Of Scope For Issue #17

The following ideas may still be useful, but they should not drive this issue:

### 1. Public chat surface redesign

Out of scope for `#17`:

- replacing `complete(TurnInput)` with `append(...)`
- replacing `TurnInput` with `AppendRequest` / `AppendOptions`
- introducing `ThreadSpec` as a new public contract
- removing `clone_thread_with_turns(...)` from the public API
- introducing a new narrower Cortex-facing capability port as a requirement for this issue

These may be sensible future work, but they violate the stated issue boundary if treated as the current target.

### 2. New public snapshot / error contract freeze

Out of scope for `#17` as currently written:

- `ChatError`
- `ThreadSnapshot`
- `TurnSnapshot`
- public restore/snapshot contract redesign

These are public contract changes, not merely internal ownership cleanup.

### 3. External config schema redesign

Potentially useful later, but not a required part of `#17`:

- shared provider inventory as a new top-level config schema
- `chat.bindings` / `chat.routes` as a new external schema
- canonical route syntax freeze such as `<capability>.<alias>`

Internal normalization may change.
External schema redesign should be treated separately unless the issue is widened explicitly.

### 4. Future multi-capability scaffolding

Out of scope for this issue:

- adding placeholder `asr/` or `tts/` trees
- forcing today's refactor to justify future capabilities before they exist

The capability-first idea may still be a useful north star, but it should not force speculative structure into issue `#17`.

## Reclassification Of Existing Task Files

### Still directly useful for issue #17

- `PLAN.md`
  - useful for first-principles ownership analysis
- selected parts of `MIGRATION-MAP.md`
  - only the behavior-preserving internal ownership cleanup

### Useful as future-direction notes, not issue-17 target contracts

- `LOW-LEVEL-DESIGN.md`
- `CONFIG-AND-CHAT-CONTRACT.md`
- `ERROR-AND-SNAPSHOT-CONTRACT.md`

These files contain useful design thought, but large parts of them should now be treated as follow-up candidate work rather than current-issue freeze targets.

## Recommended Next Questions For Issue #17

1. What is the smallest adapter contract that cleanly separates:
   - provider call lifecycle
   - inherited context
   - backend response normalization
   - adapter-local retryability knowledge

2. Which reliability controls are truly shared, and which are actually backend-specific?

3. Which metadata fields are allowed to enter provider-thread inherited context, and which must remain runtime-only?

4. How should clone lineage appear in `ai-gateway.chat.thread` observability without expanding the chat capability surface?

5. Should `attempt` survive as a request-level field, or should it be renamed to a more precise transport term?

## Working Rule Going Forward

For this task, treat all public-surface redesign ideas as follow-up notes unless they can be implemented strictly behind the current external chat surface.
