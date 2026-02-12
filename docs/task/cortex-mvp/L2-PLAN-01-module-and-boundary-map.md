# L2-01 - Module And Boundary Map
- Task Name: `cortex-mvp`
- Stage: `L2` detailed file
- Date: `2026-02-11`
- Status: `DRAFT_FOR_APPROVAL`

## 1) Implementation Boundary
This task changes Cortex from command-step API to reactor API.

Canonical boundary after this task:
1. Input to Cortex: `ReactionInput` only.
2. Output from Cortex: `ReactionResult` only.
3. Progression: inbox event arrival only.
4. Persistence: outside Cortex.

## 2) Source File Map (Planned)
### Cortex module
1. `core/src/cortex/mod.rs`
- export new reactor surface and ports.
- retire `CortexFacade::step` exports.

2. `core/src/cortex/types.rs`
- replace command/commitment-centric primary contracts with reactor contracts.
- keep deterministic id derivation helpers in cortex domain.

3. `core/src/cortex/reactor.rs` (new)
- `CortexReactor` run loop and per-cycle orchestration.

4. `core/src/cortex/pipeline.rs` (new)
- primary -> extractor/filler -> clamp -> repair orchestration helpers.

5. `core/src/cortex/clamp.rs` (new)
- deterministic validation/clamping authority.

6. `core/src/cortex/ports.rs`
- define async cognition ports (primary/extractor/filler).
- define optional out-of-band telemetry port (not in business result).

7. `core/src/cortex/adapters/ai_gateway.rs` (new)
- real runtime adapters backed by `ai_gateway::AIGateway`.

8. `core/src/cortex/error.rs`
- add reactor/pipeline-specific error kinds while preserving deterministic error typing.

### Admission/Continuity boundary updates
9. `core/src/admission/types.rs`
- extend `IntentAttempt` with `based_on: Vec<SenseId>`.
- preserve `attempt_id` as required correlation key.

10. `core/src/continuity/types.rs`
- ensure feedback event type exposed to Cortex ingress includes `attempt_id` + non-semantic code.

### Runtime/protocol wiring
11. `core/src/protocol.rs`
- introduce event message schema for Cortex ingress (`sense`, `env_snapshot`, `admission_feedback`, config updates).

12. `core/src/server.rs`
- add `CortexIngressAssembler` + bounded channels.
- run `CortexReactor` as always-on task.
- enforce mechanical backpressure at ingress boundaries.

### Config/schema
13. `core/src/config.rs`
- add `cortex` runtime config block.

14. `core/beluna.schema.json`
- add schema for new `cortex` config block.

## 3) File Retirement/Cutover
Direct cutover requirement means old command-step API is retired from canonical use.

Target retirements:
1. `CortexCommand` as runtime-facing contract.
2. `CortexFacade::step` as primary API.
3. commitment-manager-centric planning entrypoint for reactor path.

Note:
- Existing deterministic id derivation logic remains reusable.
- Legacy types can be removed or retained internally only if not exported as canonical interfaces.

## 4) Dependency Direction Rules
1. `core/src/cortex/*` may depend on:
- `core/src/ai_gateway/*` via adapters/ports,
- `core/src/admission/types.rs` for `IntentAttempt`.

2. `core/src/cortex/*` must not depend on:
- `core/src/server.rs`,
- socket protocol parsing,
- continuity persistence internals.

3. `core/src/server.rs` may depend on:
- cortex reactor API,
- continuity/admission feedback APIs,
- protocol message types.

4. `core/src/admission/*`, `core/src/continuity/*`, `core/src/spine/*` remain semantic-free execution/feedback layers.

## 5) Statelessness Enforcement Location
1. Enforced inside Cortex by design:
- no durable goals/commitments store in Cortex state.
- each cycle consumes complete bounded decision context from input.

2. Maintained outside Cortex:
- latest env snapshots,
- feedback windows,
- distributed goal source material.

3. Owner of pre-react assembly:
- runtime ingress assembler (`server` boundary), not Cortex.

## 6) Ingress And Backpressure Boundary
1. Upstream publishes events only.
2. Ingress assembler normalizes events into bounded `ReactionInput`.
3. Assembler sends to bounded Cortex inbox channel.
4. If inbox is full:
- upstream observes backpressure via blocked send or deterministic rejection code.
- no semantic override path is allowed.

## 7) L2-01 Exit Criteria
This file is complete when:
1. changed files and ownership boundaries are explicit,
2. cutover vs retained components are unambiguous,
3. statelessness and ingress responsibilities are mechanically assigned.
