# Spine HLD

## Architecture

Spine is split into:
1. Spine Core (`core/src/spine/*`): registry + router + contracts.
2. Spine Adapters (`core/src/spine/adapters/*`): transport and wire shell.

## Core Flow

1. Stem dispatches one `ActDispatchRequest`.
2. Router resolves endpoint by route key (`endpoint_id`, `capability_id`).
3. Endpoint invocation result maps to a deterministic `SpineEvent`.
4. Stem reconciles settlement in Ledger and notifies Continuity.

## Capability Catalog

1. Endpoint registrations are stored in Spine registry.
2. Registry snapshot is exposed as `SpineCapabilityCatalog`.
3. Runtime bridges Spine catalog into Cortex `CapabilityCatalog` during physical state composition.

## MVP Adapter

1. UnixSocket adapter receives NDJSON envelopes.
2. `sense`, `new_capabilities`, and `drop_capabilities` are first-class ingress envelopes.
3. Body endpoint lifecycle updates are translated into capability patch/drop senses.
