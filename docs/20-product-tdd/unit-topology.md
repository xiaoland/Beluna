# Unit Topology

Beluna currently has four technical units.

## Units

1. `core`
- Runtime authority and composition root.
- Owns cognition execution, routing, persistence, resource control, endpoint protocol authority, and observability export.

2. `cli`
- Terminal-oriented Beluna Human Interface.
- Uses Core endpoint protocol for body interaction.
- Future Moira hosting belongs to a separate task slice.

3. `apple-universal`
- Apple ecosystem Beluna Human Interface.
- Uses Core endpoint protocol for body interaction.
- Hosts the first minimum native Moira Loom surface through an embedded Moira backend runtime.

4. `moira`
- Library-first local control-plane and observability runtime unit for Beluna operation.
- Prepares local Core artifacts/profiles, supervises local Core lifecycle, ingests Core OTLP logs, and exposes host-facing control/query APIs.
- Loom is the operator-facing experience implemented by Human Interface hosts. The current Tauri/Vue Loom is a transitional implementation while Apple Universal receives the first native minimum Loom surface.

## Authority Summary

- Core retains runtime behavior, config shape, endpoint protocol, and observability emission authority.
- Moira owns local preparation, supervision, observability ingestion/storage/query/projection, and future sandbox/ledger platform adapters.
- Human Interface units own platform-native UX, endpoint interaction presentation, and any host-native Loom surface.

## Core Internal Subsystems

- `cortex`
- `stem`
- `continuity`
- `spine`
- `ledger`
- `ai_gateway`
- `body`
- `observability`

These remain internal module boundaries inside the `core` technical unit.
