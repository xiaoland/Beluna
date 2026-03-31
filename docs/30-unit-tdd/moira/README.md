# Moira Unit

Moira is Beluna's local control-plane and observability unit.

It prepares local Core artifacts and profiles, supervises local Core lifecycle, ingests OTLP logs, and provides the human-facing control plane through Loom.

Current code primarily realizes the Lachesis observability slice plus the Loom inspection surface.
Clotho preparation and Atropos supervised wake control remain authoritative Moira responsibilities, but they still require the cleanup stage defined in the unit docs below before broader feature growth lands.

- [Design](./design.md)
- [Interfaces](./interfaces.md)
- [Data And State](./data-and-state.md)
- [Operations](./operations.md)
- [Verification](./verification.md)
