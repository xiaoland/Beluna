# AI Gateway Topology (Current)

Last verified: 2026-04-06

Source anchors:

- core/src/ai_gateway/*
- core/src/main.rs
- core/src/cortex/runtime/primary.rs
- core/tests/ai_gateway/*

## Module Topology

```mermaid
flowchart LR
    Main["core/src/main.rs\nChat::new(...)"] --> ChatFacade
    Main --> Cortex["core/src/cortex/runtime/primary.rs\nCortex"]
    Cortex --> ChatFacade

    subgraph Gateway["core/src/ai_gateway"]
        ChatFacade["chat/api_chat.rs\nChat facade + thread registry"]
        Thread["chat/thread.rs\nThread aggregate + complete()"]
        Turn["chat/turn.rs\nturn/message invariants"]
        Runtime["chat/runtime.rs\ndispatch/retry/bound backend"]
        Caps["chat/capabilities.rs\ncapability guard"]
        Router["router.rs\nroute alias/key -> backend/model"]
        Resilience["resilience.rs\ncircuit/rate/concurrency/backoff"]
        Creds["credentials.rs\nCredentialProvider"]
        ToolScheduler["chat/tool_scheduler.rs"]
        ToolExec["chat/executor.rs\nToolExecutor trait"]
        Adapters["adapters/mod.rs\nBackendAdapter registry"]
        Types["types.rs + chat/types.rs\nconfig/domain/wire types"]
        Errors["error.rs\nGatewayError"]
        Telemetry["telemetry.rs\nGatewayTelemetryEvent"]
        Obs["core/src/observability/runtime/ai_gateway.rs\ncontract event emitters"]

        ChatFacade --> Thread
        ChatFacade --> Runtime
        Thread --> Runtime
        Thread --> Turn
        Thread --> ToolScheduler

        Runtime --> Caps
        Runtime --> Router
        Runtime --> Resilience
        Runtime --> Creds
        Runtime --> Adapters
        Runtime --> Telemetry
        Runtime --> Obs

        Thread --> Obs
        ChatFacade --> Obs

        Thread --> Errors
        Runtime --> Errors

        ChatFacade --> Types
        Runtime --> Types
        Thread --> Types
    end

    subgraph AdapterImpl["core/src/ai_gateway/adapters/*"]
        OpenAI["openai_compatible/chat.rs + wire.rs"]
        Ollama["ollama/chat.rs + wire.rs"]
        Copilot["github_copilot/chat.rs"]
        CopilotRpc["github_copilot/rpc.rs"]
        HttpShared["http_stream.rs + http_errors.rs + wire.rs"]

        Adapters --> OpenAI
        Adapters --> Ollama
        Adapters --> Copilot

        OpenAI --> HttpShared
        Ollama --> HttpShared
        Copilot --> CopilotRpc
    end

    OpenAI --> OpenAIEndpoint["OpenAI-compatible HTTP endpoint"]
    Ollama --> OllamaEndpoint["Ollama HTTP endpoint"]
    CopilotRpc --> CopilotProcess["Copilot language server process"]

    ToolScheduler --> ToolExec
    ToolExec -. implemented by .-> Cortex
```

## Request Flow Used Today (Complete Path)

1. `core/src/main.rs` constructs `Chat::new(config.ai_gateway, EnvCredentialProvider)` and injects `Arc<Chat>` into `Cortex::from_config`.
2. `core/src/cortex/runtime/primary.rs` opens or reuses a thread via `Chat::open_thread`.
3. `Thread::complete` builds `TurnPayload` from system prompt + thread history + input messages + tool config + limits + metadata.
4. `ChatRuntime::dispatch_complete` handles dispatch:
   - `CapabilityGuard::assert_supported`
   - `ResilienceEngine::pre_dispatch` + circuit/rate/concurrency checks
   - `BackendAdapter::complete` on selected adapter
   - retry/backoff through `ResilienceEngine::can_retry`
   - telemetry and observability event emission
5. `Thread::complete` commits the turn and assistant message to in-memory thread state.
6. If `tool_executor` exists and adapter returns tool calls, `ToolScheduler` executes each call through `ToolExecutor` (implemented by Cortex), appends tool results, and marks `pending_tool_call_continuation = true`.

## Important Current Boundaries

- Runtime surface is object-oriented: `Chat -> Thread -> Turn`.
- `Thread::stream` is currently not implemented and returns `UnsupportedCapability`.
- Adapter contract already exposes both `complete` and `stream`.
  - OpenAI-compatible and Ollama adapters implement both.
  - Copilot adapter `complete` currently consumes its own `stream` output internally.
- Routing is deterministic and config-driven (`AIGatewayConfig.backends[].models[].aliases`) and requires alias `default` to exist.
- Observability is emitted at three levels:
  - request/attempt transport events
  - chat-turn lifecycle events
  - chat-thread snapshot events

## Test Mapping (Current)

- `core/tests/ai_gateway/router.rs`: route alias/key resolution behavior.
- `core/tests/ai_gateway/thread.rs`: context derive/rewrite behavior and turn-id sequencing expectations.
- `core/tests/ai_gateway/turn.rs`: tool call/result linkage invariants and `ToolScheduler` behavior.
