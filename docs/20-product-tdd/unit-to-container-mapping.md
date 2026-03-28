# Unit-To-Container Mapping

## Current Mapping

| Technical Unit | Code Container | Packaging Shape | Mapping Rationale |
|---|---|---|---|
| `core` | `core/` | Rust crate/binary workspace | Runtime authority and tight internal subsystem coordination require one cohesive container. |
| `cli` | `cli/` | Rust crate/application | Independent endpoint surface with lightweight lifecycle and protocol-focused concerns. |
| `apple-universal` | `apple-universal/` | Swift app workspace | Platform-specific UX and lifecycle concerns justify separate app container. |
| `monitor` | `monitor/` | Static web app | Read-only local observability UX is independently evolvable while preserving core authority boundaries. |

## Mapping Rule

A code container is a storage/deployment boundary, not architecture truth by itself. Product TDD owns this mapping and its rationale.
