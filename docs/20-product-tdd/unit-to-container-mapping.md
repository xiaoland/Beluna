# Unit-To-Container Mapping

## Current Mapping

| Technical Unit | Code Container | Packaging Shape | Mapping Rationale |
|---|---|---|---|
| `core` | `core/` | Rust crate/binary workspace | Runtime authority and tight internal subsystem coordination require one cohesive container. |
| `cli` | `cli/` | Rust Human Interface application | Terminal UX and endpoint protocol workflows stay lightweight; Moira hosting is future-scope. |
| `apple-universal` | `apple-universal/` | Swift Human Interface app workspace | Apple-native UX and lifecycle concerns justify a platform app container; this container hosts the first minimum native Moira Loom. |
| `moira` | `moira/` | Rust backend/runtime package with transitional Tauri/Vue container | Local control-plane supervision, artifact management, and observability storage/query belong in a library-first runtime. The current Tauri/Vue app is extraction source and transitional evidence. |

## Mapping Rule

A code container is a storage/deployment boundary. Product TDD owns architecture mapping and rationale.
