# Spine Module

Spine is the execution substrate between Stem and Body Endpoints.

Code:
1. `core/src/spine/*` (core routing/registry/contracts)
2. `core/src/spine/adapters/*` (body endpoint dialect adapters)

Current scope:
1. single-act dispatch API (`dispatch_act`) that accepts `Act` and returns `EndpointExecutionOutcome`
2. endpoint-level routing by `act.endpoint_id`; capability routing is delegated to endpoint internals
3. capability catalog snapshot owned by Spine registry
4. Spine executor is process-wide singleton
5. UnixSocket+NDJSON body endpoint dialect adapter for:
   - `auth` ingress (endpoint registration + capability publish)
   - `sense` ingress and `act_ack` ingress
   - `act` egress and disconnect/unplug lifecycle handling
6. In-process inline body endpoints (`core/src/body/*`) attach through Spine Inline Adapter during runtime boot
7. body endpoint capabilities are reflected to runtime through `new_capabilities` / `drop_capabilities` senses

Runtime notes:
1. Registry owns remote endpoint session channels and ownership maps.
2. Spine Runtime starts adapters from `spine.adapters`; `main` then starts inline endpoints and passes them the inline adapter instance.
3. Spine returns deterministic rejection outcomes for missing endpoint and endpoint failures.
4. Stem maps outcomes to settlement-linked `SpineEvent`s.
