# Spine HLD

## Architecture

Spine is split into:
1. Spine Core (`core/src/spine/*`): registry + router + contracts + singleton lifecycle.
2. Body Endpoint Dialect Adapters (`core/src/spine/adapters/*`): concrete transport+dialect pairs.

## Core Flow

1. Stem dispatches one `Act`.
2. Router resolves endpoint by `act.endpoint_id`.
3. Endpoint routes capability internally and returns `EndpointExecutionOutcome`.
4. Stem maps outcome to deterministic `SpineEvent` and reconciles settlement.

## Capability Catalog

1. Endpoint registrations are stored in Spine registry.
2. Registry snapshot is exposed as `SpineCapabilityCatalog`.
3. Runtime bridges Spine catalog into Cortex `CapabilityCatalog` during physical state composition.
4. Registry also owns remote endpoint session channels and endpoint/route ownership state.

## MVP Adapter

1. UnixSocket+NDJSON adapter receives NDJSON envelopes over AF_UNIX.
2. `sense`, `new_capabilities`, and `drop_capabilities` are first-class ingress envelopes.
3. Body endpoint lifecycle updates are translated into capability patch/drop senses.
4. Adapter handles transport+dialect only; ownership and dispatch maps live in registry.
