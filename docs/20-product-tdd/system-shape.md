# System Shape

Beluna is a multi-component system:

1. `core/`: runtime binary and domain subsystems.
2. `cli/`: standalone body-endpoint client.
3. `apple-universal/`: Apple body-endpoint application.

Core process shape:

1. Load config and initialize observability.
2. Build Stem-owned afferent/efferent pathways and physical-state store.
3. Build Spine runtime and endpoint adapters.
4. Build Continuity engine.
5. Build Cortex runtime with AI Gateway dependencies.
6. Spawn Stem tick runtime, efferent runtime, and Cortex runtime.
7. Handle graceful shutdown with bounded drain.

Design Source: ADR-001, ADR-002.
