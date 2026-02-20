# Spine LLD

## Data Model

1. `Act` dispatch input (from `runtime_types`).
2. `RouteKey { endpoint_id, capability_id }` for capability catalog/drop bookkeeping.
3. `EndpointCapabilityDescriptor` and `SpineCapabilityCatalog { version, entries[] }`.
4. `EndpointExecutionOutcome` returned by Spine dispatch.
5. `SpineEvent` settlement mapping performed by Stem.

## Registry Rules

1. duplicate route registration (`endpoint_id` + `capability_id`) is rejected (`RouteConflict`).
2. same endpoint must keep consistent descriptor shape/defaults (`RegistrationInvalid`).
3. endpoint resolution is keyed by `endpoint_id` only.
4. catalog snapshots are sorted by route key.
5. registry owns remote session channels, route ownership, and endpoint ownership maps.

## Router Rules

1. `dispatch_act(act)` validates required act fields.
2. missing endpoint -> `Rejected(reason_code = "endpoint_not_found")`.
3. endpoint invoke error -> `Rejected(reason_code = "endpoint_error")`.
4. returned outcome is forwarded unchanged to Stem.

## Adapter Dialect Rules

1. UnixSocket+NDJSON adapter owns bind/accept/read/write lifecycle.
2. NDJSON framing/parser is part of `spine/adapters/unix_socket.rs` dialect implementation.
3. `auth`, `sense`, `act_ack`, and `unplug` ingress are validated and forwarded.
4. `act` egress is wrapped as NDJSON envelope and retried until `act_ack` or timeout.
5. adapter does not own route/session dispatch policy state.
