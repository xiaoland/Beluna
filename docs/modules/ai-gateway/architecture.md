# Architecture

## High-Level Components

- `Chat`: public facade API (`new`, `open_thread`, `open_thread_with_route`, `clone_thread_with_turns`, `query_turns`)
- `Thread`: backend-bound conversation aggregate, stores ordered `Vec<Turn>`
- `Turn`: atomic unit with ordered `Vec<Message>`
- `Message`: explicit message layer (`SystemMessage`, `UserMessage`, `AssistantMessage`, `ToolCallMessage`, `ToolCallResultMessage`)
- `ToolScheduler`: executes `ToolCallMessage` and appends `ToolCallResultMessage`
- `ResilienceEngine`: retry/backoff/circuit + per-backend concurrency/rate + timeout bound
- `BackendAdapter`: transport + dialect mapping (unchanged boundary)

## Layering Rule

- Routing + credential resolution happen only at thread open/clone time.
- `Thread.complete()` does not perform turn-level backend routing.
- Backend switching requires creating/cloning a new thread.
- Token budget enforcement is not inside AI Gateway; usage is reported for caller-level policy.

## Runtime Invariants

- `Turn` is the atomic composition unit for thread history.
- `Thread` keeps ordered in-memory turns (`Vec<Turn>`), no external turn/thread store in this version.
- Turn message operations are atomic (`append_one`, `truncate_one`).
- If appending `ToolCallMessage`, tool execution is scheduled immediately and paired result message is appended in the same atomic operation.
- Tool-call/result completeness is mandatory turn integrity condition.
