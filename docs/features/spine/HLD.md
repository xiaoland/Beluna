# Spine HLD

## Architecture

Spine is split into:
1. Spine Core (`core/src/spine/*`): registry + router + contracts.
2. Spine Adapters (`core/src/spine/adapters/*`): transport and wire shell.

## Core Flow

1. Admission emits `AdmittedActionBatch`.
2. Spine router resolves endpoint by route key (`affordance_key`, `capability_handle`).
3. Endpoint invocation result maps to `SpineEvent`.
4. Spine emits ordered report for Continuity settlement.

## Capability Catalog

1. Endpoint registrations are stored in Spine registry.
2. Registry snapshot becomes `SpineCapabilityCatalog`.
3. Runtime bridges Spine catalog to Cortex `CapabilityCatalog` view.

## MVP Adapter

1. UnixSocket adapter receives NDJSON envelopes and forwards normalized `ClientMessage`.
2. `Sense` is first-class ingress for reaction triggers.
