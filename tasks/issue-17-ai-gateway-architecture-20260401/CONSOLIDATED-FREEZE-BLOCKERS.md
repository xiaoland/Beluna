# Consolidated Freeze Blockers

## Status

Historical pre-freeze note.
Its main blockers have now been resolved or normalized by
`CONSOLIDATED-CHAT-CONTRACT-FREEZE.md`.

## Conclusion

This file captured the last real blockers before the consolidated freeze was emitted.
It is retained as historical reasoning, not as the current active status.

## Real Blockers

### 1. Derived-thread / clone semantics are still unresolved

This is the main blocker.

Current notes agree on several negative rules:

- public raw `clone_thread_with_turns(...)` should not survive
- rewrite/clone-style surgery should preserve surviving `turn_id`
- lineage belongs to chat semantics and observability, not adapter transport

But they do not yet define one positive public semantic answer to:

- how a caller creates a derived thread from committed history

Still unresolved:

- keep one public clone/fork semantic operation
- or express derivation through snapshot/restore
- or express it through a higher-level rewrite/derive request

Without that answer, the public thread contract is incomplete.

### 2. Public write-path naming is mostly aligned, but not fully

The broader notes strongly lean toward:

- `append(...)`
- `append_message(...)`
- `append_messages(...)`

But one earlier low-level note still mentions:

- `Thread::advance(...)`

This is not a deep architectural blocker, but it means the final freeze still needs one explicit
choice of canonical public method names.

### 3. Route-key naming is not yet normalized

Current broader notes use both:

- `ChatRouteKey`
- `RouteKey`

The route semantics are mostly aligned:

- canonical grammar is `<capability>.<alias>`
- restore uses resolved canonical route

But the public type naming is still inconsistent.

### 4. `rewrite_context(...)` mutability contract is inconsistent

Current sketches show both:

- `&mut self`
- `&self`

This is not only syntax.
It changes how much semantic mutation is visible in the public object model.

If this is frozen sloppily, readability will degrade because the public surface will hide or
misstate mutation semantics.

### 5. Error and snapshot docs are strong locally, but not yet the single owner

`ERROR-AND-SNAPSHOT-CONTRACT.md` is already fairly concrete.
But the final freeze still needs one place that definitively states:

- the public thread contract
- the public error contract
- the public snapshot/restore contract
- the clone/rewrite relationship

Right now those truths are spread across multiple files.

## What Is Good Enough Already

These do not block consolidation:

1. thread-centric write-path direction
2. `UserMessage` as ordinary append input
3. one append call equals one committed turn transaction
4. canonical tool-call/result message representation
5. system prompt as thread-level canonical state
6. snapshot exports only committed canonical state
7. `attempt` remains transport terminology
8. provider context remains explicit and default-deny

## Minimum Decisions Needed Before Final Consolidated Freeze

1. Choose one positive public answer for derived-thread creation semantics.
2. Choose canonical public write-path method names.
3. Choose canonical route-key type naming.
4. Choose whether semantic mutators on `Thread` use `&mut self` or `&self`.

## Historical Outcome

The consolidated freeze later resolved these points by:

1. replacing raw clone semantics with sibling `derive_context(...)` / `rewrite_context(...)`
2. freezing `append(...)` as the public write-path naming
3. freezing `ChatRouteKey` as the public route type name
4. explicitly not freezing exact Rust receiver mutability as a public semantic requirement
