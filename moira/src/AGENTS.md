# AGENTS.md for moira/src

## Scope

- This file applies to the frontend under `moira/src`.
- Follow [../AGENTS.md](/Users/lanzhijiang/Development/Beluna/moira/AGENTS.md) and the repository root AGENTS first.

## Frontend Ownership

1. `bridge` owns Tauri `invoke`, event subscription, environment detection, and backend-shaped payload contracts only.
2. `query` owns wake/tick selection, refresh orchestration, loading state, live-update reactions, and query-owned UI state such as the active Lachesis detail tab.
3. `projection` owns normalization, chronology reconstruction, interval pairing, narrative shaping, structured drilldown assembly, and Loom-facing normalized models.
4. `presentation` owns Vue components, layout, popups, formatting, and visual composition.

## Local Rules

1. Keep data flow one-way: `bridge -> query -> projection -> presentation`.
2. Mythic names such as `lachesis`, `clotho`, and `atropos` are feature namespaces inside the layers above, not replacements for them.
3. Components must not call Tauri `invoke` directly.
4. Components must not become ad hoc OTLP interpreters just because one more label or summary is needed.
5. Raw JSON is a drilldown surface, not the primary browsing contract.
6. Keep Lachesis projection split by durable transformation work such as `receiver`, `wakes`, `ticks`, `raw-events`, `chronology`, `narratives`, and `labels` instead of rebuilding one large helper file.
7. Keep presentation grouped by operator task when that grouping is stable, for example `loom/chrome`, `loom/shared`, `lachesis/workspace`, `lachesis/chronology`, and `lachesis/inspectors`.
8. Do not recreate a shared frontend type bucket. Raw bridge contracts belong in `bridge/contracts`, normalized Lachesis models belong in `projection/lachesis/models`, and query-owned UI state belongs with `query`.

## Refactor Guardrails

1. Do not let `App.vue` remain the owner of live update wiring and selection logic after a query-state owner exists.
   `App.vue` should stay a thin root shell once `app/LoomApp.vue` exists.
2. Do not let `api.ts` or future bridge files own normalization, sorting, or Loom narrative rules.
3. If presentation helpers begin encoding Lachesis meaning, move that logic back into `projection`.
4. If one presentation subtree starts owning cross-cutting data shaping, stop and move that work back into `projection` or `query` before continuing.
