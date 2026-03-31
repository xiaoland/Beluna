# L1 Plan - Body Endpoints MVP (High-Level Strategy)
- Task Name: `body-endpoints-mvp`
- Stage: `L1` (high-level strategy)
- Date: `2026-02-12`
- Status: `DRAFT_FOR_APPROVAL`

## 0) Locked Inputs And Amendments
User-confirmed direction:

1. Apple Universal App is a Body Endpoint (not part of `std-body`).
2. Apple endpoint behavior remains simple chatbot logic.
3. Apple wire direction aligns with OpenAI Responses API semantics.
4. Shell policy is open command execution with hard caps.
5. Web policy is open outbound HTTP(S) with hard caps.

Boundary amendment from follow-up review:
1. `core` implementation is not modified in this task.
2. `core` is treated as process-boundary (start/stop lifecycle), not in-process runtime composition.
3. Integration glue lives in `runtime/src`.

## 1) Strategy Summary
Implement Body endpoints as external endpoint clients over Spine UnixSocket:

1. `std-body` hosts shell/web endpoints and self-registers with Spine UnixSocket.
2. Apple Universal App self-registers as `chat.reply.emit` endpoint via UnixSocket.
3. `runtime/src` manages process lifecycle (`start`/`stop`) for core and std-body host.
4. Spine core behavior remains unchanged.

## 2) Target Architecture

```text
runtime (process lifecycle only)
  -> starts/stops core process
  -> starts/stops std-body host process

core process
  -> Spine UnixSocket Adapter
  -> Cortex/Continuity/Admission/Ledger/Spine internal loop

std-body host (external endpoint client)
  -> registers shell/web endpoints
  -> handles endpoint_invoke and sends endpoint_result

Apple Universal App (external endpoint client)
  -> registers chat endpoint
  -> handles endpoint_invoke and sends endpoint_result
```

## 3) Endpoint Capability Model
1. Shell:
- `tool.shell.exec` / `cap.std.shell`
2. Web:
- `tool.web.fetch` / `cap.std.web.fetch`
3. Apple chat output:
- `chat.reply.emit` / `cap.apple.universal.chat`

## 4) OpenAI Responses Alignment Scope
Apple payload subset aligns with Responses semantics:

1. user input shape carried in `sense.payload` using `message + input_text`.
2. assistant output carried in invoke payload using `response.output[].message.content[].output_text`.
3. alignment is schema-level subset, not full protocol clone.

References:
1. https://developers.openai.com/api/reference/resources/responses
2. https://developers.openai.com/api/reference/resources/responses/methods/create

## 5) Responsibility Split
1. `core`: unchanged domain engine and socket adapter behavior.
2. `runtime`: process lifecycle orchestration only.
3. `std-body`: shell/web endpoint logic and endpoint-client host loop.
4. `apple-universal`: chat UI and Apple endpoint-client loop.

## 6) Key Technical Decisions
1. No `std_body_bridge` inside core.
2. No duplicated core-generic endpoint type system inside `std-body`.
3. Shell/web handlers define only endpoint-specific payload DTOs.
4. Timeout and output/response caps are enforced in endpoint handlers.
5. Deterministic failure reason mapping is preserved at endpoint-result level.

## 7) Risks And Mitigations
1. Risk: core socket protocol may lack endpoint register/invoke/result flow in current build.
- Mitigation: treat as explicit compatibility gate before L3 implementation; discuss exception if missing.
2. Risk: process lifecycle drift between runtime and child processes.
- Mitigation: PID/state supervision and bounded stop policy.
3. Risk: shell/web abuse.
- Mitigation: strict caps and deterministic rejection codes.

## 8) L2 Deliverables
L2 defines:
1. exact file map for `runtime` and `std-body`.
2. endpoint-client wire contracts for std-body and Apple.
3. start/stop lifecycle algorithms.
4. contract-to-test matrix and compatibility gates.

## 9) L1 Exit Criteria
1. process-boundary architecture is explicit and accepted.
2. Apple self-registration flow is explicit and accepted.
3. no-core-modification constraint is reflected in all downstream plans.

Status: `READY_FOR_L2_APPROVAL`
