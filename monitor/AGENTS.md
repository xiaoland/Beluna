# AGENTS.md for monitor

## Design Sources

- `docs/20-product-tdd/unit-topology.md`
- `docs/20-product-tdd/cross-unit-contracts.md`
- `docs/30-unit-tdd/monitor/*`
- `core/src/logging.rs`

## Boundaries

1. Monitor is a read-only consumer of local core logs.
2. Monitor must not redefine or mutate core runtime authority.
3. Monitor must not change core log format assumptions locally; any contract drift must be promoted via docs.

## Current MVP Constraints

1. Pure static browser app (no backend).
2. Directory access via `showDirectoryPicker`.
3. Auto refresh prefers `FileSystemObserver` and degrades to polling.

## Coding Notes

1. Keep source files small and focused.
2. Preserve graceful behavior for malformed NDJSON lines.
3. Keep mobile and desktop layouts usable.
