# AGENTS.md for moira

## Design Sources

- `docs/20-product-tdd/observability-contract.md`
- `docs/30-unit-tdd/moira/*`
- `docs/30-unit-tdd/core/observability.md`

## Boundaries

1. Moira owns local ingestion, storage, query, supervision, and Loom UX.
2. Moira must not redefine Core runtime behavior or Core observability emission semantics.
3. Use biological lifecycle verbs for Moira-facing actions where reasonable, but keep existing cross-unit correlation names stable unless the docs move first.

## Stage 1 Constraint

1. Land `raw_events`, `runs`, and `ticks` first.
2. Reconstruct selected tick detail from raw events before adding narrower projections.
3. Prefer live end-to-end inspection over broad automated test growth while the read models are still moving.
