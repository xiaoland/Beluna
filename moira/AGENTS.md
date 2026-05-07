# AGENTS.md for moira

## Design Sources

- `docs/20-product-tdd/observability-contract.md`
- `docs/30-unit-tdd/moira/*`
- `docs/30-unit-tdd/core/observability.md`

## Boundaries

1. Moira owns local preparation, supervision, ingestion, storage, query, projection, and host-facing Loom runtime semantics.
2. Core retains runtime behavior, config schema, endpoint protocol, and observability emission authority.
3. Human Interface hosts own platform-native Loom presentation.
4. The current Tauri/Vue Loom is transitional during backend runtime extraction.
5. Use biological lifecycle verbs for Moira-facing actions where reasonable, while keeping existing cross-unit correlation names stable until the docs move first.

## Stage 1 Constraint

1. Land `raw_events`, `runs`, and `ticks` first.
2. Reconstruct selected tick detail from raw events before adding narrower projections.
3. Prefer live end-to-end inspection over broad automated test growth while the read models are still moving.

## Issue 30 Direction

1. Moira backend moves toward a library-first runtime package.
2. Apple Universal is the first Human Interface host for a minimum native Loom.
3. The first Apple slice uses process-local embedded Moira runtime.
4. Owner/Attach authority coordination belongs to a later task packet.
