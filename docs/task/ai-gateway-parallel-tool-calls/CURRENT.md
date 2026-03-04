# AI Gateway Chat Tool Calls — Current State

## Scope

This document describes the current implementation of AI Gateway Chat tool-call flow, including where tool calls are parsed, where they are executed, and how continuation works with Cortex ticks.

## Current Sequence (Non-stream Turn)

```mermaid
sequenceDiagram
    participant Tick as "Stem Tick"
    participant Runtime as "CortexRuntime"
    participant Primary as "Cortex::cortex()"
    participant Thread as "AI Gateway Thread.complete()"
    participant Store as "AI Gateway ThreadStore"
    participant Dispatch as "ChatDispatcher"
    participant Adapter as "BackendAdapter"
    participant Backend as "LLM Backend"
    participant Tools as "Cortex Internal Tool Executor"

    Tick->>Runtime: on_tick()
    Runtime->>Runtime: drain buffered senses
    Runtime->>Primary: cortex(senses, physical_state)

    Primary->>Thread: complete(TurnInput)
    Thread->>Store: prepare_turn(history + input)
    Thread->>Dispatch: complete(TurnPayload)
    Dispatch->>Adapter: complete(adapter_ctx, payload)
    Adapter->>Backend: infer request
    Backend-->>Adapter: assistant output (+ optional tool_calls)
    Adapter-->>Dispatch: BackendCompleteResponse { tool_calls }
    Dispatch-->>Thread: TurnResponse { output_text, tool_calls }
    Thread->>Store: commit_turn_success(assistant message with tool_calls)
    Thread-->>Primary: TurnResponse

    alt tool_calls is empty
        Primary-->>Runtime: output_text, pending_primary_continuation=false
    else tool_calls exists
        Primary->>Tools: run_primary_internal_tool_calls() (sequential loop)
        Tools-->>Primary: tool-result payloads
        Primary->>Primary: store pending_tool_messages in PrimaryContinuationState
        Primary-->>Runtime: pending_primary_continuation=true
        Note over Runtime: No immediate backend turn. Continuation waits for next admitted tick.
    end

    Note over Runtime,Primary: Next admitted tick injects pending_tool_messages + new tick user message.
```

## Current Topology (Tool-call Relevant)

```mermaid
flowchart LR
    subgraph AI_Gateway["AI Gateway (core/src/ai_gateway)"]
        CF["ChatFactory"]
        TH["Thread.complete"]
        TS["ThreadStore"]
        CD["ChatDispatcher"]
        AD["Adapters (openai_compatible / ollama / copilot)"]
    end

    subgraph Cortex["Cortex (core/src/cortex/runtime)"]
        CR["CortexRuntime (tick-driven)"]
        CP["Primary turn runner"]
        CT["run_primary_internal_tool_calls (sequential)"]
        CS["PrimaryContinuationState\npending_tool_messages + merged context"]
    end

    subgraph External["External"]
        LLM["LLM Backend"]
        EFF["Efferent dispatch / Continuity / Spine"]
    end

    CR --> CP
    CP --> TH
    TH --> TS
    TH --> CD
    CD --> AD
    AD --> LLM
    LLM --> AD
    AD --> CD
    CD --> TH
    TH --> CP

    CP --> CT
    CT --> EFF
    CT --> CS
    CS --> CP
```

## Responsibility Split Today

| Concern | Current Owner | Notes |
|---|---|---|
| Parse backend tool-call wire payload into structured calls | AI Gateway adapters | `parse_tool_calls_from_message(...)` in adapters |
| Keep assistant `tool_calls` in message history | AI Gateway Thread API + ThreadStore | Assistant message committed with `tool_calls` |
| Validate tool-message linkage before dispatch | AI Gateway Thread API | `validate_tool_message_chain(...)` |
| Execute tool calls (actual function invocation) | Cortex Primary | `run_primary_internal_tool_calls(...)` |
| Tool-call batch execution strategy | Cortex Primary | Sequential `for` loop |
| Continuation timing policy | CortexRuntime | Tick-admitted only (no immediate micro-turn) |

## Concrete Code Anchors

- AI Gateway turn orchestration: `core/src/ai_gateway/chat/api.rs`
- AI Gateway backend invocation and response mapping: `core/src/ai_gateway/chat/dispatcher.rs`
- Adapter tool-call parsing:
  - `core/src/ai_gateway/adapters/openai_compatible/chat.rs`
  - `core/src/ai_gateway/adapters/ollama/chat.rs`
- Cortex primary tool-call execution: `core/src/cortex/runtime/primary.rs`
- Tick gate / continuation admission: `core/src/cortex/runtime/mod.rs`

## Current Constraints Relevant to Parallelization

1. AI Gateway currently has no tool executor abstraction and does not invoke tools.
2. Tool execution logic depends on Cortex-local mutable state (`PrimaryTurnState`, goal-forest state, continuation state).
3. Continuation is already next-tick based in runtime behavior.
4. There is currently no uniqueness guard that forbids duplicated tool names within a single assistant tool-call batch.
