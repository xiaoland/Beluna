# Spine LLD

## Data Model

1. `RouteKey { affordance_key, capability_handle }`
2. `EndpointRegistration { endpoint_id, descriptor }`
3. `SpineCapabilityCatalog { version, entries[] }`
4. `EndpointInvocation { action }`
5. `EndpointExecutionOutcome` -> `SpineEvent` mapping

## Registry Rules

1. duplicate route registration is rejected (`RouteConflict`).
2. same affordance must keep consistent descriptor shape and defaults (`RegistrationInvalid`).
3. catalog snapshots are sorted by route key.

## Router Rules

1. serialized mode preserves original batch order.
2. best-effort mode executes concurrently but report order remains by `seq_no`.
3. route miss -> `ActionRejected(reason_code = "route_not_found")`.
4. endpoint invoke error -> `ActionRejected(reason_code = "endpoint_error")`.

## Adapter Shell Rules

1. UnixSocket adapter owns bind/accept/read lifecycle.
2. wire parser (`spine/adapters/wire.rs`) validates envelope schema.
3. `Sense` ingress triggers reaction input assembly in runtime.
