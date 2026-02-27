# AI Gateway — Chat Capability Sequence Diagrams

## 1. `chat_once()` — Synchronous Single-Turn

```mermaid
sequenceDiagram
    participant Caller as Caller (Cortex/Stem)
    participant GW as AIGateway
    participant RN as RequestNormalizer
    participant Router as BackendRouter
    participant Creds as CredentialProvider
    participant Cap as CapabilityGuard
    participant Budget as BudgetEnforcer
    participant Rel as ReliabilityLayer
    participant Adapter as BackendAdapter
    participant Backend as AI Backend (HTTP/RPC)
    participant Tel as Telemetry

    Caller->>GW: chat_once(ChatRequest)
    GW->>RN: normalize_chat(request, stream=false)
    RN-->>GW: CanonicalRequest

    GW->>GW: dispatch_once(CanonicalRequest)
    GW->>Router: select(&canonical_request)
    Router-->>GW: SelectedBackend {backend_id, profile, resolved_model}

    GW->>Creds: resolve(credential_ref, profile)
    Creds-->>GW: ResolvedCredential

    GW->>GW: lookup adapter by dialect
    GW->>Cap: assert_supported(&canonical_request, &capabilities)
    Cap-->>GW: Ok

    GW->>Budget: pre_dispatch(&canonical_request, backend_id)
    Budget-->>GW: BudgetLease {effective_timeout, permit}

    GW->>Tel: emit RequestStarted

    loop retry loop (max_retries)
        GW->>Tel: emit AttemptStarted
        GW->>Rel: ensure_backend_allowed(backend_id)
        Rel-->>GW: Ok

        GW->>Adapter: invoke_once(AdapterContext, CanonicalRequest)
        Adapter->>Backend: HTTP POST /chat/completions (or /api/chat)
        Backend-->>Adapter: JSON response
        Adapter-->>GW: BackendOnceResponse

        alt Success
            GW->>Rel: record_success(backend_id)
            GW->>Budget: observe_event (Usage)
            GW->>Budget: release(lease)
            GW->>Tel: emit RequestCompleted
            GW-->>Caller: ChatResponse
        else Retryable Error
            GW->>Rel: record_failure(backend_id)
            GW->>Tel: emit AttemptFailed
            GW->>GW: sleep(backoff_delay)
            Note over GW: continue loop
        else Terminal Error
            GW->>Rel: record_failure(backend_id)
            GW->>Budget: release(lease)
            GW->>Tel: emit RequestFailed
            GW-->>Caller: GatewayError
        end
    end
```

## 2. `chat_stream()` — Streaming

```mermaid
sequenceDiagram
    participant Caller as Caller
    participant GW as AIGateway
    participant RN as RequestNormalizer
    participant Router as BackendRouter
    participant Creds as CredentialProvider
    participant Cap as CapabilityGuard
    participant Budget as BudgetEnforcer
    participant Task as run_stream_task (spawned)
    participant Rel as ReliabilityLayer
    participant Adapter as BackendAdapter
    participant ResNorm as ResponseNormalizer
    participant Backend as AI Backend
    participant Tel as Telemetry

    Caller->>GW: chat_stream(ChatRequest)
    GW->>RN: normalize_chat(request, stream=true)
    RN-->>GW: CanonicalRequest

    GW->>Router: select(&canonical_request)
    Router-->>GW: SelectedBackend
    GW->>Creds: resolve(credential_ref, profile)
    Creds-->>GW: ResolvedCredential
    GW->>Cap: assert_supported(&canonical_request, &capabilities)
    GW->>Budget: pre_dispatch(&canonical_request, backend_id)
    Budget-->>GW: BudgetLease
    GW->>Tel: emit RequestStarted

    GW->>Task: spawn run_stream_task(tx, ...)
    GW-->>Caller: ChatEventStream (rx)

    Task->>Caller: ChatEvent::Started

    loop retry loop
        Task->>Tel: emit AttemptStarted
        Task->>Rel: ensure_backend_allowed(backend_id)

        Task->>Adapter: invoke_stream(AdapterContext, CanonicalRequest)
        Adapter->>Backend: HTTP POST (stream=true, SSE/NDJSON)
        Adapter-->>Task: AdapterInvocation {stream, cancel}

        loop consume adapter stream
            Backend-->>Adapter: SSE chunk / NDJSON line
            Adapter-->>Task: BackendRawEvent
            Task->>ResNorm: map_raw(request_id, raw_event)
            ResNorm-->>Task: ChatEvent

            alt TextDelta / ToolCallDelta / ToolCallReady
                Task->>Caller: ChatEvent (via mpsc)
            else Usage
                Task->>Budget: observe_event(backend_id, event)
                Task->>Caller: ChatEvent::Usage
            else Completed
                Task->>Rel: record_success(backend_id)
                Task->>Caller: ChatEvent::Completed
                Task->>Budget: release(lease)
                Task->>Tel: emit RequestCompleted
            else Failed (retryable)
                Task->>Rel: record_failure(backend_id)
                Task->>Tel: emit AttemptFailed
                Note over Task: sleep(backoff), continue outer loop
            else Failed (terminal)
                Task->>Rel: record_failure(backend_id)
                Task->>Caller: ChatEvent::Failed
                Task->>Budget: release(lease)
                Task->>Tel: emit RequestFailed
            end
        end
    end
```

## 3. Chat Session Layer — `turn_once()`

```mermaid
sequenceDiagram
    participant Caller as Caller
    participant TH as ChatThreadHandle
    participant Store as InMemoryChatSessionStore
    participant GW as AIGateway

    Caller->>TH: turn_once(ChatTurnRequest)

    TH->>Store: prepare_turn(session_id, thread_id, input_messages)
    Store-->>TH: PreparedTurn {turn_id, route_ref, messages (history + input)}

    TH->>TH: build ChatRequest from PreparedTurn + TurnRequest fields

    TH->>GW: chat_once(ChatRequest)
    Note over GW: (see chat_once sequence above)
    GW-->>TH: ChatResponse

    alt Success
        TH->>TH: assistant_message_from_response(response)
        TH->>Store: commit_turn_success(input_messages, assistant_message, usage, ...)
        Store-->>TH: TurnCommitOutcome
        TH->>TH: emit_turn_summary + record_turn_metrics
        TH-->>Caller: ChatTurnResponse {session_id, thread_id, turn_id, response}
    else Error
        TH->>Store: commit_turn_failure(session_id, thread_id, latency_ms)
        TH-->>Caller: GatewayError
    end
```

## Key Data Flow Summary

```text
ChatRequest (Beluna types)
  ↓  RequestNormalizer.normalize_chat()
CanonicalRequest (internal canonical types)
  ↓  Router.select() → SelectedBackend
  ↓  CapabilityGuard.assert_supported()
  ↓  BudgetEnforcer.pre_dispatch() → BudgetLease
  ↓  Adapter.invoke_once/invoke_stream(AdapterContext, CanonicalRequest)
  ↓      ↓ http_common: CanonicalMessage → wire JSON
  ↓      ↓ HTTP/RPC to backend
  ↓  BackendRawEvent stream
  ↓  ResponseNormalizer.map_raw()
ChatEvent stream / ChatResponse (Beluna types)
```
