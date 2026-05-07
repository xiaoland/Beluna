# AGENTS.md for moira/src-tauri/src

This file applies to the Rust backend currently housed under `moira/src-tauri/src`.

The target direction is a library-first Moira runtime extracted from this transitional Tauri container.

## Backend Ownership

1. `app` owns transitional Tauri bootstrap, command wiring, app state composition, and app-wide event wiring.
2. `clotho` owns wake-input preparation concerns such as artifact/profile preparation homes.
3. `lachesis` owns OTLP ingest, DuckDB persistence, projections, and observability query behavior.
4. `atropos` owns supervised lifecycle behavior such as `wake`, graceful stop, force-kill, and terminal process state.
5. Future host API code owns typed runtime surfaces consumed by Apple Universal and later hosts.

## Local Rules

1. Keep Tauri command handlers thin. They are transport facades over runtime owners.
2. Lachesis owns DuckDB observability tables and projections.
3. Preserve current command names and `lachesis-updated` behavior unless the docs move first.
4. Prefer moving behavior behind the owning module boundary over adding more shared helpers in `app`.
5. Use biological lifecycle verbs for Moira-facing lifecycle actions where reasonable, while stable cross-unit correlation fields move only with docs.
6. Host-facing runtime APIs should use typed DTOs and explicit resource status.
7. Apple first-slice runtime behavior is process-local; cross-client Owner/Attach coordination belongs to later work.

## Refactor Guardrails

1. Keep new behavior behind `clotho`, `lachesis`, `atropos`, platform adapter, or host API owners.
2. Keep future supervision and preparation work under Atropos or Clotho ownership.
3. If a change needs new persistence, decide the owner first, then decide module state, filesystem state, or database schema.
