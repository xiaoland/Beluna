# Data Model

## Key Types

- `ChatRequest`: caller-facing chat capability input
- `CanonicalRequest`: backend-neutral normalized input
- `ChatEvent`: chat streaming event union
- `ChatResponse`: non-stream aggregate output
- `GatewayError`: canonical error taxonomy

Code split:

- non capability-specific types are in `core/src/ai_gateway/types.rs`
- chat capability types are in `core/src/ai_gateway/types_chat.rs`

## Event Lifecycle

Chat stream contract:

- first event: `Started`
- zero or more non-terminal events (`TextDelta`, `ToolCallDelta`, `ToolCallReady`, optional `Usage`)
- exactly one terminal event: `Completed` or `Failed`

## Tool Status Scope

- Gateway emits tool-call statuses for inference-time assembly (`Partial`, `Ready`).
- `Executed` and `Rejected` are runtime/tool-execution states and are not gateway stream emissions.
