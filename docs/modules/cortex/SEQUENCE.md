# Cortex Sequence

## Scope and Lifecycle

`cortex-primary-thread` is a long-lived AI Gateway thread (session/process scope).

A Cortex cycle is an admitted tick unit. It is not equal to thread lifetime.

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
    Afferent->>Runtime: sense event (buffer only)
    Runtime->>Runtime: drain buffered senses for current tick
    Runtime->>Primary: run cycle for admitted tick

    Primary->>Thread: complete(user message with current tick senses)
    Thread-->>Primary: assistant (tool_calls)

    Primary->>Tools: execute each tool call
    Tools-->>Primary: tool results
    Primary->>Thread: append tool results within the same turn
    Primary->>Primary: store continuation state only for cycle control
    Primary-->>Runtime: pending_primary_continuation=true

    Runtime->>Primary: next admitted tick cycle
    Primary->>Thread: complete(current tick user message)
    Thread-->>Primary: assistant (next tool_calls or final text)
```

## Context Reset Path

```mermaid
sequenceDiagram
    participant Primary as "Cortex Primary"
    participant Thread as "Source Thread"
    participant Chat as "AI Gateway Chat"
    participant Clone as "Cloned Thread"

    Primary->>Thread: find_turns(query)
    Thread-->>Primary: selected turn ids
    Primary->>Chat: clone_thread_with_turns(source, ordered_turn_ids)
    Chat-->>Primary: cloned thread with deep-copied turns/messages
    Primary->>Primary: swap active thread reference
```

## Post-Fix Invariants

1. Tool-call/result linkage is preserved inside a single turn.
2. Turn append/truncate operations keep the turn structurally complete at all times.
3. Cortex resets context by picking turns and cloning a new thread instead of mutating old thread history in place.

These keep cycle-driven execution while preserving a persistent chat thread.
