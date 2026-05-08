# Moira Unit

Moira is Beluna's local control-plane and observability unit.

It prepares local Core artifacts and profiles, supervises local Core lifecycle, ingests OTLP logs, and exposes host-facing runtime APIs for Loom surfaces.

Current backend code realizes the first explicit multi-owner Moira runtime:

1. `Lachesis` for receiver status, wake browse, tick browse, and selected-tick inspection.
2. `Atropos` for supervised runtime status, wake, graceful stop, and force-kill.
3. `Clotho` for launch-target preparation and profile curation, including known local build registration, explicit local forge, published release intake, and app-local JSONC profile documents.

Apple Universal hosts the first minimum native Loom surface through an embedded process-local Moira backend runtime. The legacy Tauri/Vue Loom has been retired from the active Moira code surface.

Schema-validation interactions with Core authority remain deferred to a later Clotho slice.

- [Design](./design.md)
- [Interfaces](./interfaces.md)
- [Data And State](./data-and-state.md)
- [Operations](./operations.md)
- [Verification](./verification.md)
