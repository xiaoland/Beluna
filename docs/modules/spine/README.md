# Spine Module

Spine is the execution substrate between Admission/Continuity and Body Endpoints.

Code:
1. `core/src/spine/*` (core routing/registry/contracts)
2. `core/src/spine/adapters/*` (transport shells and wire parsing)

Current scope:
1. async routing executor with in-memory endpoint registry
2. capability catalog snapshot owned by Spine
3. UnixSocket adapter shell (`sense` ingress + body endpoint lifecycle)
4. runtime bootstrap is exposed as `Spine::start(config, ingress_tx, builtin_sense_tx)` and owns adapter startup
5. active adapter config path: `spine.adapters.unix_socket.{enabled,socket_path}`
6. Spine exposes capability catalog getters (`Spine::capability_catalog_snapshot()` and `Spine::external_capability_catalog_snapshot()`); runtime merge (internal + spine) is performed in `brainstem`, with internal currently empty

Body endpoint lifecycle envelopes supported by UnixSocket adapter:
1. `body_endpoint_register`
2. `body_endpoint_invoke`
3. `body_endpoint_result`
4. `body_endpoint_unregister`
