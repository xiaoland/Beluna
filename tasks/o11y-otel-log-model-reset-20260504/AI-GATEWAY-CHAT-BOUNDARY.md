# AI Gateway Chat Boundary

This note records the current implementation evidence for `request_id` and the target owner boundary for Slice 2.

It is tactical and non-authoritative.

## Current Implementation Evidence

### Cortex organ operation id

`core/src/cortex/runtime/primary.rs` creates a Cortex operation id before invoking AI Gateway Chat:

1. primary organ call id: `cortex-{stage}-{cycle_id}-turn-{step}`.
2. helper organ call id: `cortex-{stage}-{cycle_id}`.
3. `build_turn_input()` stores this value in chat metadata under the key `request_id`.
4. AI Gateway Chat reads metadata `request_id` as a fallback parent span id.

### AI Gateway backend dispatch id

`core/src/ai_gateway/chat/thread.rs` creates a backend dispatch id with:

```rust
let request_id = next_request_id(&guard.backend.backend_id, &guard.backend.model);
```

`core/src/ai_gateway/chat/runtime.rs` defines:

```rust
format!("{}-{}-{}", backend_id, model, seq)
```

This id is passed to `dispatch_complete()`, emitted on AI Gateway request events, stored in `AdapterContext`, and inserted into `TurnResponse.backend_metadata["request_id"]`.

### Transport header

`core/src/ai_gateway/adapters/http_stream.rs` forwards the backend dispatch id as HTTP header `x-request-id`.

### Bridge back to Cortex

`core/src/cortex/runtime/primary.rs` reads `TurnResponse.backend_metadata["request_id"]` and passes it into Cortex organ finish events as `ai_request_id`.

## Target Boundary

AI Gateway should split into two owner scopes:

1. `beluna.core.ai-gateway`: backend dispatch and transport request lifecycle.
2. `beluna.core.ai-gateway.chat`: thread/turn lifecycle and chat rich payloads.

Attributes stay event-schema local:

1. wake/tick grouping comes from trace id plus bootstrap/tick anchor events.
2. backend/model/capability can be AI Gateway transport attributes when the transport event schema needs them.
3. generic `operation.status` is withdrawn from the target fixture; eventName, severity, and event-specific body fields carry outcome.

Body carries owner rich payload:

1. chat ids: `chat_id`, `thread_id`, `turn_id`.
2. transport id: `transport_request_id`.
3. committed messages, finish reason, usage, backend metadata.
4. dispatch request payload on dispatch/start events when full request preservation is useful.

## Naming Risk

The current name `request_id` covers two roles:

1. Cortex operation id passed through chat metadata.
2. AI Gateway backend dispatch/transport request id.

Target schema should resolve these roles before Slice 2 implementation:

1. retire the Cortex-created organ operation id from the target schema when the Cortex span key is `primary`.
2. `transport_request_id` for the AI Gateway backend dispatch and HTTP request id.
3. `thread_id` and `turn_id` under `beluna.core.ai-gateway.chat`.

## `turn.committed` Payload Boundary

Current `AiGatewayChatTurnArgs` carries both `dispatch_payload` and `messages_when_committed`.

On committed turn events, the two fields overlap because both include the user/input messages that led to the turn. Their meanings differ:

1. `dispatch_payload` is the backend request shape before dispatch.
2. `messages_when_committed` is the committed thread turn after response materialization.

Target model:

1. `turn.dispatched` carries the full dispatch payload.
2. `turn.committed` carries committed state, finish reason, usage, backend metadata, and references to the transport request.
3. `turn.committed` can omit full `dispatch_payload` once the dispatch event exists.
