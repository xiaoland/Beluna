# Spine PRD

## Purpose

Spine is the sole execution channel between Stem and Body Endpoints.
It is transport-agnostic and performs mechanical route dispatch only.

## Core Invariants

1. Transport ignorance in Spine core.
2. Universal endpoint abstraction for local/native and remote endpoints.
3. Mechanical routing by endpoint key (`endpoint_id`) only.
4. Capability routing is delegated to Body Endpoint internals.
5. Spine executor is process-wide singleton.

## Requirements

1. Spine accepts `Act` only.
2. Spine dispatches one act per call and returns one `ActDispatchResult` (`Acknowledged|Rejected|Lost`).
3. Missing endpoint maps to deterministic rejection (`endpoint_not_found`).
4. Endpoint invocation failure maps to deterministic rejection (`endpoint_error`).
5. Spine registry owns capability catalog snapshot and remote endpoint session ownership state.
6. Spine and adapters publish topology/body proprioception through `new_proprioceptions` / `drop_proprioceptions` senses.
7. Stem maps outcome to dispatch terminal status (`ACK|REJECTED|LOST`) and proprioception updates.
8. Neural signal design must follow: good abstraction, orthogonal parameters, and reasonable defaults.
9. AI-facing neural-signal APIs must be composable enough for low-level operations, not only high-level macros.

## MVP Scope

1. Spine core routing kernel, singleton lifecycle hooks, and in-memory registry.
2. UnixSocket+NDJSON body endpoint dialect adapter for `auth`/`sense`/`act_ack`/`new_proprioceptions`/`drop_proprioceptions` ingress, `act` egress, and endpoint lifecycle.
3. No WebSocket+Protobuf adapter in this MVP iteration.
