# Spine Module

Spine is the execution substrate between Admission/Continuity and Body Endpoints.

Code:
1. `core/src/spine/*` (core routing/registry/contracts)
2. `core/src/spine/adapters/*` (transport shells and wire parsing)

Current scope:
1. async routing executor with in-memory endpoint registry
2. capability catalog snapshot owned by Spine
3. UnixSocket adapter shell (`sense` ingress)
