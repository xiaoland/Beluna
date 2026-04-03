# Moira Unit

Moira is Beluna's local control-plane and observability unit.

It prepares local Core artifacts and profiles, supervises local Core lifecycle, ingests OTLP logs, and provides the human-facing control plane through Loom.

Current code realizes the first explicit multi-owner Moira shell:

1. `Lachesis` for receiver status, wake browse, tick browse, and selected-tick inspection.
2. `Atropos` for supervised runtime status, wake, graceful stop, and force-kill.
3. `Clotho` for known local build registration plus app-local JSONC profile documents.

Published artifact discovery, checksum trust, local source-folder compile, and schema-validation interactions with Core authority remain later Clotho slices rather than current realization.

- [Design](./design.md)
- [Interfaces](./interfaces.md)
- [Data And State](./data-and-state.md)
- [Operations](./operations.md)
- [Verification](./verification.md)
