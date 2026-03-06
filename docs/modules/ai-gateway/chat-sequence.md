# AI Gateway Chat Sequence

## 1. Open Thread

```mermaid
sequenceDiagram
    participant Caller
    participant Chat as Chat Facade
    participant Router as BackendRouter
    participant Creds as CredentialProvider

    Caller->>Chat: open_thread(options)
    Chat->>Router: select(route_or_alias/default)
    Router-->>Chat: SelectedBackend
    Chat->>Creds: resolve(profile.credential)
    Creds-->>Chat: ResolvedCredential
    Chat->>Chat: bind backend adapter value into Thread
    Chat-->>Caller: Thread
```

## 2. Complete Turn

```mermaid
sequenceDiagram
    participant Caller
    participant Thread
    participant Runtime as ChatRuntime
    participant Resilience as ResilienceEngine
    participant Adapter as BackendAdapter
    participant Backend

    Caller->>Thread: complete(TurnInput)
    Thread->>Thread: build payload from prior turns + input
    Thread->>Runtime: dispatch_complete(bound_backend, payload)
    Runtime->>Resilience: pre_dispatch/ensure_backend_allowed
    Runtime->>Adapter: complete(AdapterContext, payload)
    Adapter->>Backend: provider call
    Backend-->>Adapter: provider response
    Adapter-->>Runtime: BackendCompleteResponse
    Runtime->>Resilience: record_success/release
    Runtime-->>Thread: TurnResponse
    Thread->>Thread: build new Turn and append atomically
    Thread-->>Caller: TurnOutput
```

## 3. Tool Call Append Atomicity

```mermaid
sequenceDiagram
    participant Thread
    participant Turn
    participant Scheduler as ToolScheduler
    participant Executor as ToolExecutor

    Thread->>Turn: append_one(ToolCallMessage)
    Turn->>Scheduler: execute_tool_call(call)
    Scheduler->>Executor: execute_call(request)
    alt tool success
        Executor-->>Scheduler: payload
    else tool failure
        Executor-->>Scheduler: error
    end
    Scheduler-->>Turn: ToolCallResultMessage (success/error payload)
    Turn->>Turn: append call + result in one logical unit
```

## 4. Pick + Clone Thread

```mermaid
sequenceDiagram
    participant Cortex
    participant Chat
    participant Source as Source Thread
    participant New as New Thread

    Cortex->>Source: find_turns(query)
    Source-->>Cortex: selected turn ids
    Cortex->>Chat: clone_thread_with_turns(source, ordered_ids, options)
    Chat->>Chat: deep copy selected turns/messages
    Chat->>Chat: reindex turn_id as thread-local monotonic sequence
    Chat-->>Cortex: New Thread
```

## Key Guarantees

- Routing happens only on `open_thread`/`clone_thread_with_turns`.
- Thread backend binding is fixed for the thread lifecycle.
- `Turn` invariants enforce tool call/result linkage completeness.
- Gateway resilience handles retry/backoff/circuit/concurrency/rate/timeout.
- Gateway returns usage data but does not enforce caller budget policy.
