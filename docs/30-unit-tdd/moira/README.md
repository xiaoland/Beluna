# Moira Unit

Moira is Beluna's local control-plane and observability unit.

It prepares local Core artifacts and profiles, supervises local Core lifecycle, ingests OTLP logs, and provides the human-facing control plane through Loom.

Current code realizes the first explicit multi-owner Moira shell:

1. `Lachesis` for receiver status, wake browse, tick browse, and selected-tick inspection.
2. `Atropos` for supervised runtime status, wake, graceful stop, and force-kill.
3. `Clotho` for launch-target preparation and profile curation, including known local build registration, explicit local forge, published release intake, and app-local JSONC profile documents.

Schema-validation interactions with Core authority remain deferred to a later Clotho slice rather than current realization.

- [Design](./design.md)
- [Interfaces](./interfaces.md)
- [Data And State](./data-and-state.md)
- [Operations](./operations.md)
- [Verification](./verification.md)
