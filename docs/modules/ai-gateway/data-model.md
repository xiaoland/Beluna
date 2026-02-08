# Data Model

## Key Types

- `BelunaInferenceRequest`: caller-facing input
- `CanonicalRequest`: backend-neutral normalized input
- `GatewayEvent`: canonical streaming event union
- `CanonicalFinalResponse`: non-stream aggregate output
- `GatewayError`: canonical error taxonomy

## Event Lifecycle

Canonical stream contract:

- first event: `Started`
- zero or more non-terminal events (`OutputTextDelta`, `ToolCallDelta`, `ToolCallReady`, optional `Usage`)
- exactly one terminal event: `Completed` or `Failed`

## Tool Status Scope

- Gateway emits tool-call statuses for inference-time assembly (`Partial`, `Ready`).
- `Executed` and `Rejected` are runtime/tool-execution states and are not gateway stream emissions.
