# Core Authority Drift Remediation Plan (2026-02-14)

## Goal

Align Core with the provided authority statements, with ruthless refactor allowed and no compatibility layer requirement.

## Design Decisions Locked by Authority

1. Endpoint-name suffixing (`<semantic_name>.<monotonic_int>`) happens when Spine registers a body endpoint.
2. Body endpoint registration is not a sense.
3. Auth is handled by adapter; auth payload carries registration info and may include capabilities declaration; adapter calls Spine `new_body_endpoint`.
4. Body endpoint may send unplug request; adapter calls Spine `remove_body_endpoint`.
5. Adapter owns endpoint session/connection lifecycle; Spine tracks endpoint-to-adapter ownership only.
6. Spine `start` boots all configured adapters from `config.spine.adapters[]`, each entry shaped as `{type, config}`.
7. Only capability changes flow through sense queue.
8. No capability-singleton enforcement in catalog; Act includes fully-qualified endpoint name and capability routing is completed within body endpoint.
9. No compatibility mode required.

## Workstreams

### WS1 — Contract realignment (P0)

- Introduce Cortex-owned contract module for Sense/Act/Capability declarations (or canonical re-home with strict ownership docs).
- Replace ambiguous endpoint identity fields with:
  - `body_endpoint_id: Uuid`
  - `body_endpoint_name: String` (fully-qualified name assigned by Spine)
- Ensure Act contains fully-qualified endpoint name field used for dispatch.

### WS2 — Spine endpoint lifecycle API (P0)

Add explicit Spine APIs:

- `new_body_endpoint(authenticated_registration) -> BodyEndpointHandle`
  - Input includes semantic base name declared by endpoint + adapter identity.
  - Auth payload may include initial capabilities declaration for immediate bootstrap.
  - Spine allocates UUID and monotonic suffix, then records fully-qualified endpoint name.
- `remove_body_endpoint(body_endpoint_id | fully_qualified_name)`
  - Removes endpoint and all attached capability routes.
- Spine maintains endpoint-to-adapter ownership mapping only (no socket/session state machine in Spine).
- Keep Spine process singleton semantics.

### WS3 — Adapter auth/registration/unplug flow (P0)

Refactor unix adapter protocol handling to:

1. Accept `auth` method frame.
2. Validate/authenticate at adapter boundary.
3. Transform auth-carried registration payload (plus optional capabilities declaration) into Spine `new_body_endpoint` request.
4. Keep session lifecycle in adapter (connect/disconnect/retry); Spine only receives ownership-level events.
5. Accept `sense` method for domain senses and capability change intents only.
6. Accept unplug request from endpoint, map to Spine `remove_body_endpoint`.
7. Outbound adapter->endpoint uses only `act` method frames.

### WS4 — Capability change flow through sense only (P0)

- Restrict sense control events to capability add/drop updates.
- Ensure endpoint lifecycle changes (new/remove body endpoint) do not use sense path.
- Keep single authoritative point that mutates capability catalog through control-sense handling.

### WS5 — Routing semantics adjustment (P1)

- Update dispatch to route by fully-qualified endpoint name from Act to endpoint session.
- Capability id is treated as endpoint-internal routing token; Spine does not enforce singleton semantics across catalog.
- Remove central assumptions that require unique global capability behavior.

### WS6 — NDJSON v2 protocol rewrite (P1)

Adopt strict envelope:

```json
{
  "method": "sense | act | auth | unplug",
  "id": "uuid-v4",
  "timestamp": 1739500000000,
  "body": { }
}
```

Rules:

- Endpoint -> adapter: `auth`, `sense`, `unplug`
- Adapter -> endpoint: `act`
- Reject any direction violation.
- Remove legacy `type` frame handling (no compatibility path).

### WS7 — Spine startup and adapter configuration model (P1/P2)

- Add `config.spine.adapters[]` and adapter factory wiring.
- `Spine.start` boots all configured adapters (`{type, config}`), e.g. `unix-socket-ndjson` with `{socket_path}`.
- Start in-core body endpoints via inline adapter after Spine starts.
- Start Stem after adapter and endpoint startup stage.

### WS8 — Tests and conformance (P0-P2)

Add/adjust tests for:

- Spine monotonic endpoint-name assignment at endpoint registration.
- Auth (with/without capabilities declaration) -> `new_body_endpoint` transform contract.
- Unplug -> `remove_body_endpoint` contract.
- Spine does not hold adapter session/socket state; only endpoint-to-adapter ownership mapping is asserted.
- Directional protocol enforcement for NDJSON v2 methods.
- Capability-change-only sense path.
- Dispatch by fully-qualified endpoint name.

## Suggested Execution Sequence

1. WS2 (Spine lifecycle API) + WS1 (contract fields)
2. WS3 (adapter auth/registration/unplug)
3. WS4 (sense boundary cleanup)
4. WS6 (NDJSON v2 rewrite)
5. WS5 (routing semantics)
6. WS7 (boot order)
7. WS8 (conformance closure)

## Completion Criteria

- Endpoint registration path is: endpoint `auth` (optionally with capabilities) -> adapter auth -> Spine `new_body_endpoint` -> fully-qualified name assigned.
- Endpoint unplug path is: endpoint `unplug` -> adapter -> Spine `remove_body_endpoint`.
- Registration/removal are not emitted as senses.
- Capability changes are the only endpoint-related events in sense flow.
- Spine startup is adapter-array-driven via `config.spine.adapters[]`.
- Wire protocol conforms to required NDJSON v2 envelope and direction rules.
