# AGENTS.md for moira/src-tauri/src

This file applies to the Rust backend under `moira/src-tauri/src`.

## Backend Ownership

1. `app` owns Tauri bootstrap, command wiring, app state composition, and app-wide event wiring.
2. `clotho` owns wake-input preparation concerns such as artifact/profile preparation homes.
3. `lachesis` owns OTLP ingest, DuckDB persistence, projections, and observability query behavior.
4. `atropos` owns supervised lifecycle behavior such as `wake`, graceful stop, force-kill, and terminal process state.

## Local Rules

1. Keep Tauri command handlers thin. They are transport façades, not owners of query, storage, or process logic.
2. Lachesis is the only owner of DuckDB observability tables and projections. Do not fold Clotho or Atropos state into Lachesis tables as a shortcut.
3. Preserve current command names and `lachesis-updated` behavior unless the docs move first.
4. Prefer moving behavior behind the owning module boundary over adding more shared helpers in `app`.
5. Use biological lifecycle verbs for Moira-facing lifecycle actions where reasonable, but do not rename stable cross-unit correlation fields opportunistically.

## Refactor Guardrails

1. Do not let `lib.rs` or app bootstrap become the fallback home for new behavior after the module split exists.
2. Do not let Lachesis become a convenience owner for future supervision or preparation work just because it already has persistence.
3. If a change needs new persistence, decide the owner first, then decide whether it belongs in module state, filesystem state, or a database schema.
