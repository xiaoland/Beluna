# Spine Module

Spine is the execution substrate between Stem and Body Endpoints.

Code:
1. `core/src/spine/*` (core routing/registry/contracts)
2. `core/src/spine/adapters/*` (transport shells and wire parsing)

Current scope:
1. single-act dispatch API (`dispatch_act`) with deterministic sequence context from Stem
2. capability catalog snapshot owned by Spine registry
3. UnixSocket adapter shell for:
   - body endpoint register/unregister/disconnect lifecycle
   - external sense ingress
   - capability patch/drop ingress
4. body endpoint capabilities are reflected to runtime through `new_capabilities` / `drop_capabilities` senses

Runtime notes:
1. Spine execution reports settlement linkage fields (`reserve_entry_id`, `cost_attribution_id`) on every event.
2. Missing route and endpoint failures are mapped to deterministic rejection events.
