# Four-Question Freeze

## Status

Proposed freeze for continued design discussion.
Exploratory, non-authoritative.

This file intentionally combines:

- the current AI Gateway discussion
- the near-term Cortex restructuring constraints from GitHub issue `#14`

But it does so in one direction only:

- issue `#14` constrains what AI Gateway must not make impossible
- AI Gateway should not absorb Cortex-specific orchestration semantics that belong above it

That boundary matters.

## Why this freeze exists

The discussion now has enough low-level context.
What it lacks is a small set of hard invariants.

Without that freeze, we will keep drifting between:

- better naming
- nicer public APIs
- speculative config cleanup
- future multi-capability stories

while the real correctness boundary stays blurry.

The four questions to freeze are:

1. canonical history
2. one-call transaction semantics
3. provider-context admission
4. retry-safety contract

## Issue-14 Constraints That Matter Here

Issue `#14` adds several useful constraints:

- `Primary / Attention / Cleanup` will become real runtime owners, not prompt-only names
- `Attention` and `Cleanup` are derived chats created from committed `Primary` history
- derived chats must inherit committed `Primary` history, tool results, and the effective system-prompt state
- runtime-only control outputs such as `break`, `sleep`, and `reset_context` must be explicit and must not leak through inheritable message text

This means AI Gateway must preserve one clean distinction:

- committed inheritable chat state
- runtime-only control state

If that distinction stays blurry, issue `#14` becomes harder, not easier.

## Freeze 1: Canonical History

### Decision

Beluna canonical chat history consists of:

- thread-level effective state
- ordered committed turns
- canonical committed messages inside those turns

Committed tool activity uses one canonical representation:

- explicit `ToolCallMessage`
- explicit `ToolCallResultMessage`

Not two equal-status committed representations.

### Explicit exclusion

The following are not canonical chat history:

- `break` signal
- `sleep` signal
- `reset_context` intent before it is applied
- tick/request/span identifiers
- retry counters
- backend transient transport state
- pending continuation control flags

These may exist as runtime control or observability data.
They must not become inheritable chat history.

### System prompt rule

The effective system prompt is canonical thread-level state.

However, I do **not** recommend freezing "full system prompt history as ordinary message history."
That would make the model dirtier, not clearer.

If issue `#14` truly needs reconstructable prompt lineage, represent it as:

- thread-level rewrite provenance

not as:

- fake `SystemMessage` turns mixed into ordinary dialogue history

This is the first deliberate challenge to the current instinct.
Keeping prompt lineage and ordinary dialogue as one timeline would hurt readability and future cleanup ownership.

### Why this freeze helps issue 14

Derived `Attention` / `Cleanup` chats can then inherit:

- committed turns
- committed tool results
- current effective system prompt

without inheriting:

- `Primary`-only break state
- sleep routing state
- pre-application cleanup control

## Freeze 2: One Public Call Equals One Committed Turn Transaction

### Decision

One caller-visible `Thread.complete(TurnInput)` call equals:

- at most one committed turn transaction

Internally, runtime may still perform multiple steps.
But caller semantics stay:

- either one valid committed turn is produced
- or nothing is committed

### Allowed committed terminal shapes

A committed turn may end as:

- ordinary assistant output
- tool-call terminal state with fully paired canonical tool-call/result messages

It must not end as:

- dangling tool calls
- half-normalized provider payload
- partially committed control metadata

### Continuation rule

Cross-tick continuation may still exist in runtime behavior.
Issue `#14` strongly suggests it will continue to exist.

But that continuation state is:

- runtime-owned
- non-canonical
- non-inheritable by derived chats

So the important freeze is not "continuation disappears."
The important freeze is:

- continuation state is not canonical history

This is the second deliberate challenge to the current instinct.
Trying to eliminate continuation too early would force AI Gateway design to pretend Cortex tick semantics do not exist.

### Failure rule

If tool execution or backend normalization cannot produce a valid committed turn:

- commit nothing

This is necessary to keep derived-chat cloning in issue `#14` source-grounded and deterministic.

## Freeze 3: Provider-Context Admission Is Explicit And Default-Deny

### Decision

There are three different classes of state:

1. canonical chat state
2. runtime control / observability metadata
3. provider-context state

They must not be mixed through one free-form bag.

Provider-context inheritance must be:

- explicit
- admitted
- backend-aware
- default-deny

### Never-admit examples

The following must not enter provider-context inheritance by accident:

- `tick`
- `request_id`
- `parent_span_id`
- `organ_id`
- `cortex_stage`
- break / sleep / reset control state
- retry / budget counters
- pending continuation flags
- local-only observability metadata

### Admit-only examples

The only plausible admitted provider context is something like:

- provider-native thread id
- provider-native resumable handle
- provider-native continuation token
- other backend-managed context explicitly blessed by runtime policy

### Why this freeze helps issue 14

Derived `Attention` / `Cleanup` chats may need derived provider context eventually.
But if provider inheritance is not explicit, then `Primary` runtime control state will leak downward by accident.

That would directly violate issue `#14`'s ownership goal.

## Freeze 4: Retry-Safety Needs A Real Classification, Not A Boolean Story

### Decision

Keep shared retry orchestration in the shared execution layer:

- breaker
- concurrency
- timeout shaping
- backoff
- top-level retry loop

But adapters must provide a more precise retry-safety classification than:

- `GatewayError.retryable`
- `supports_tool_retry()`

That current pair is too weak.

### Minimum classification the design should support

Whether the final internal type is enum-like or field-based, it must distinguish at least:

- retryable before visible provider output
- retryable after partial output only with resumable provider context
- retryable after tool emission only when adapter proves it is safe
- not retryable

### Runtime rule

Shared runtime still decides whether to retry.
But it must decide using:

- shared policy
- current execution phase
- adapter-provided retry-safety knowledge

not just one coarse boolean.

### Why this freeze helps issue 14

Issue `#14` introduces explicit `Primary / Attention / Cleanup` control routing.
That makes duplicated or replayed control effects more dangerous.

Therefore automatic retry must never casually re-drive:

- control-like tool effects
- cleanup side effects
- sleep-like routing outcomes

unless runtime plus adapter can prove safe resume or idempotence.

This is the third deliberate challenge to the current instinct:

- do not optimize first for "more retries"
- optimize first for "no duplicated semantic effects"

## Combined Result

If these four freezes hold, then the design gains a much cleaner foundation:

- inherited chat state is committed and reconstructable
- runtime-only control remains explicit and non-inheritable
- provider context cannot absorb random metadata
- retries stop pretending all failures share one safety profile

And issue `#14` gets a cleaner path to:

- derived-chat creation
- explicit break / sleep / cleanup routing
- maintainable separation of `Primary / Attention / Cleanup`

without forcing AI Gateway to become Cortex orchestration logic.

## What Should Not Be Frozen Yet

Still avoid freezing these now:

- new public API names such as `append(...)`
- public replacement of `TurnInput`
- public `ChatError` / `ThreadSnapshot` redesign
- external config schema redesign
- full multi-capability folder layout beyond the minimum needed for current clarity

Those may still be right later.
They are just not the real frontier right now.
