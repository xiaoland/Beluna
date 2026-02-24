# Spine HLD

## Architecture

Spine is split into:
1. Spine Core (`core/src/spine/*`): registry + router + contracts + singleton lifecycle.
2. Body Endpoint Dialect Adapters (`core/src/spine/adapters/*`): concrete transport+dialect pairs.

## Core Flow

1. Stem dispatches one `Act`.
2. Router resolves endpoint by `act.endpoint_id`.
3. Endpoint routes capability internally and returns `ActDispatchResult`.
4. `Spine::on_act_final` emits dispatch-failure domain senses on `Rejected|Lost`.
5. Stem maps final status to dispatch proprioception updates.

## Capability Catalog

1. Endpoint registrations are stored in Spine registry.
2. Registry snapshot is exposed as `SpineCapabilityCatalog`.
3. Runtime bridges Spine catalog into Cortex `CapabilityCatalog` during physical state composition.
4. Registry also owns remote endpoint session channels and endpoint/route ownership state.
5. Proprioception is not exposed as Spine snapshot API; adapters emit proprioception control senses directly.

## MVP Adapter

1. UnixSocket+NDJSON adapter receives NDJSON envelopes over AF_UNIX.
2. `auth`, `sense`, `act_ack`, `new_proprioceptions`, `drop_proprioceptions`, and `unplug` are ingress methods.
3. `act` is the adapter egress method to body endpoints.
4. Body endpoint lifecycle updates are translated into capability patch/drop and proprioception patch/drop senses.
5. Adapter handles transport+dialect only; ownership and dispatch maps live in registry.
