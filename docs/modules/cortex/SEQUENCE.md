# Cortex Sequence

## Scope and Lifecycle

`cortex-primary-thread` is a long-lived AI Gateway thread (session/process scope).

A Cortex cycle is a runtime trigger unit (tick/sense/continuation). It is not equal to thread lifetime.

## Normal Turn + Continuation

```mermaid
sequenceDiagram
    participant Tick as "Stem Tick"
    participant Afferent as "Afferent Pathway"
    participant Runtime as "CortexRuntime"
    participant Primary as "Cortex Primary"
    participant Thread as "AI Gateway Thread (cortex-primary-thread)"
    participant Tools as "Primary Tools"

    Tick->>Runtime: tick grant
    Afferent->>Runtime: sense event
    Runtime->>Primary: run cycle (new turn or continuation)

    Primary->>Thread: complete(user message on new turn)
    Thread-->>Primary: assistant (tool_calls)

    Primary->>Tools: execute each tool call
    Tools-->>Primary: tool results
    Primary->>Primary: store continuation state (tool-role messages)
    Primary-->>Runtime: pending_primary_continuation=true

    Runtime->>Primary: next cycle as continuation
    Primary->>Thread: complete(tool-role messages)
    Thread-->>Primary: assistant (next tool_calls or final text)
```

## Failure Path (Observed 2026-03-03)

```mermaid
sequenceDiagram
    participant Primary as "Cortex Primary"
    participant Store as "AI Gateway ThreadStore"
    participant Backend as "LLM Backend"

    Note over Store: "max_turn_context_messages reached"
    Primary->>Store: commit turn N success (assistant has tool_calls)
    Store->>Store: trim_context by count
    Note over Store: "old prefix trimmed into orphan tool messages"

    Primary->>Backend: turn N+1 with full history + tool messages
    Backend-->>Primary: 400 InvalidRequest (tool must follow assistant.tool_calls)
    Note over Primary: "continuation replay then loops on same turn_id"
```

## Post-Fix Invariants

1. ThreadStore trimming removes leading orphan `tool` messages after compaction/mutation.
2. Thread preflight validates tool-message chain before backend dispatch.
3. Cortex Primary self-heals this error class by resetting continuation + primary thread state.

These keep cycle-driven execution while preserving a persistent chat thread.
