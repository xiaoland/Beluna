# Spine Module

Spine is the execution substrate between Stem and Body Endpoints.

Code:
1. `core/src/spine/*` (core routing/registry/contracts)
2. `core/src/spine/adapters/*` (body endpoint dialect adapters)

Current scope:
1. single-act dispatch middleware API (`on_act`) plus low-level dispatch API (`dispatch_act`)
2. endpoint-level routing by `act.endpoint_id`; descriptor routing is delegated to endpoint internals
3. Stem is neural-signal descriptor catalog SSoT; Spine keeps only minimal route index for dispatch lookup
4. Spine executor is process-wide singleton
5. UnixSocket+NDJSON body endpoint dialect adapter for:
   - `auth` ingress (endpoint registration + `ns_descriptors` publish)
   - `sense` ingress and `act_ack` ingress
   - `act` egress and disconnect/unplug lifecycle handling
6. In-process inline body endpoints (`core/src/body/*`) attach through Spine Inline Adapter during runtime boot
7. body endpoint descriptor/proprioception mutations are applied through Spine runtime control calls into Stem (`StemControlPort`)
8. descriptor registration/drop follows Stem-first commit:
   - Spine sends patch/drop to Stem
   - Stem validates and returns accepted/rejected mutation result
   - Spine updates route index only from accepted mutation set

Runtime notes:
1. Registry owns remote endpoint session channels and ownership maps.
2. Spine Runtime starts adapters from `spine.adapters`; `main` then starts inline endpoints and passes them the inline adapter instance.
3. `on_act` maps dispatch outcomes to middleware decisions (`Continue|Break`) and emits dispatch-failure senses into afferent pathway.
4. Spine returns deterministic rejection outcomes for missing endpoint and endpoint failures.
