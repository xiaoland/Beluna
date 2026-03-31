# L2-02 - Interfaces And Data Contracts
- Task Name: `body-endpoints-mvp`
- Stage: `L2`
- Date: `2026-02-12`
- Status: `DRAFT_FOR_APPROVAL`

## 1) Core Boundary Rule
1. `core` is treated as a process boundary, not a library integration target for this task.
2. `std-body` must not redefine core-generic endpoint models (`route`, `cost`, `invocation`, `outcome`) as parallel Rust type systems.
3. Generic endpoint envelopes follow canonical Spine UnixSocket protocol fields (core is source of truth).

## 2) `std-body` Contracts (No Core-Type Duplication)

### 2.1 What `std-body` defines
`std-body` defines only endpoint-specific payload DTOs:

1. Shell payload DTO:
- `ShellExecRequest { argv, cwd, env, timeout_ms, stdout_max_bytes, stderr_max_bytes }`
2. Web payload DTO:
- `WebFetchRequest { url, method, headers, body_text, timeout_ms, response_max_bytes }`

### 2.2 What `std-body` does not define
1. no duplicate `RouteKey`-like structs.
2. no duplicate `CostVector`-like structs.
3. no duplicate endpoint invocation/outcome model structs.

### 2.3 Host loop contract
`std-body/src/host.rs` processes canonical socket envelopes:
1. send `endpoint_register` for shell and web routes.
2. receive `endpoint_invoke`.
3. execute endpoint handler.
4. send `endpoint_result`.

Envelope parsing is performed via schema-keyed `serde_json::Value` extraction so core envelope shape remains single-source.

## 3) Runtime Lifecycle Contracts (`runtime/src`)

### 3.1 Runtime command surface
`runtime/src/main.rs` exposes:
1. `start`
2. `stop`
3. `status` (optional but recommended)

### 3.2 Start contract
`start` must:
1. start core process.
2. wait until core UnixSocket is available.
3. start std-body endpoint host process.
4. not wire Apple app directly; Apple self-registers when launched.

### 3.3 Stop contract
`stop` must:
1. send `exit` command to core UnixSocket.
2. stop/cleanup std-body host child.
3. remove/refresh runtime PID state.

## 4) Apple Universal App Protocol Contract
Apple app is an independent endpoint client:

1. connects to Spine UnixSocket.
2. sends `endpoint_register` for:
- `affordance_key: "chat.reply.emit"`
- `capability_handle: "cap.apple.universal.chat"`
3. receives `endpoint_invoke`.
4. sends `endpoint_result`.

Runtime does not register this endpoint on Apple’s behalf.

## 5) OpenAI Responses-Aligned Payload Subset

### 5.1 Apple -> Core user input (inside `sense.payload`)
```json
{
  "conversation_id": "conv_001",
  "input": [
    {
      "type": "message",
      "role": "user",
      "content": [
        { "type": "input_text", "text": "..." }
      ]
    }
  ]
}
```

### 5.2 Core -> Apple chatbot invoke payload (`normalized_payload`)
```json
{
  "conversation_id": "conv_001",
  "response": {
    "object": "response",
    "id": "resp_001",
    "output": [
      {
        "type": "message",
        "role": "assistant",
        "content": [
          { "type": "output_text", "text": "..." }
        ]
      }
    ]
  }
}
```

References:
1. https://developers.openai.com/api/reference/resources/responses
2. https://developers.openai.com/api/reference/resources/responses/methods/create

## 6) Explicit Compatibility Gate
This design assumes Spine UnixSocket protocol supports endpoint client lifecycle envelopes:
1. `endpoint_register`
2. `endpoint_invoke`
3. `endpoint_result`
4. `endpoint_unregister` (optional but preferred)

If these are absent in current core build, implementation is blocked under current "no core changes" rule and must be discussed before L3 execution.

Status: `READY_FOR_REVIEW`
