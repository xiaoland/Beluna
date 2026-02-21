# AGENTS.md for core/src/spine

Spine defines contracts for endpoint-level routing of serial `Act` dispatches.

## Invariants
- Spine accepts `Act` dispatches only.
- Routing is a mechanical endpoint lookup by `act.endpoint_id`.
- Capability routing is delegated to the target Body Endpoint.
- Spine executor is process-wide singleton initialized once at runtime boot.
- Registry owns remote endpoint session channels and lifecycle ownership.
- Inline adapter owns inline endpoint mailboxes and lifecycle ownership.
- Middleware entrypoint is `on_act` and returns `Continue`/`Break`.
- Dispatch failures are emitted back into afferent pathway as domain senses.
