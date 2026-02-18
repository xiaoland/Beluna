# AGENTS.md for core/src/body

`core/src/body` contains Core-bundled Inline Body Endpoints.

## Invariants
- Endpoints in this module run in the same process as Core.
- Endpoints are started from `main` and attach through Spine Inline Adapter.
- Endpoint act/sense mailboxes are created and owned by Spine Inline Adapter.
- Endpoint senses are forwarded to `SenseAfferentPathway` by Spine Inline Adapter.
- Config gating and limits are defined by `Config.body.*`.
