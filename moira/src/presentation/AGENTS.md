# AGENTS.md for moira/src/presentation

## Scope

- This file applies to the frontend presentation layer under `moira/src/presentation`.
- Follow [../AGENTS.md](/Users/lanzhijiang/Development/Beluna/moira/src/AGENTS.md), [../../AGENTS.md](/Users/lanzhijiang/Development/Beluna/moira/AGENTS.md), and the repository root AGENTS first.

## Presentation Ownership

1. `loom/chrome`
- Owns status chrome, feature tabs, shared dialog shell, and cross-feature shell scaffolding only.

2. `lachesis/workspace`, `lachesis/chronology`, `lachesis/inspectors`
- Own observability browsing views for wake selection, tick browsing, chronology inspection, and raw-event drilldown.

3. `atropos/runtime`
- Owns runtime supervision panels and force-kill confirmation UI.

4. `clotho/workshop` and `clotho/dialogs`
- Own build-registration and profile-document browsing or editing surfaces.

5. `loom/shared`
- Owns generic JSON display helpers only.

## Local Rules

1. Presentation consumes query and projection outputs only. Components must not call Tauri `invoke` directly.
2. Keep `loom/chrome` generic. Do not move feature-specific preparation, supervision, or observability semantics into shared chrome unless multiple stations genuinely reuse them.
3. Dialog open or close truth belongs to query state. Presentation emits intents and renders shells; it does not become the source of truth for workflow orchestration.
4. `profile_id` is the primary operator-facing label for profiles. Raw profile paths are secondary derived information, not the main editing contract.
5. If a component starts reconstructing OTLP meaning, runtime rules, or file-resolution logic, move that work back into `projection`, `query`, or the backend owner.

## Guardrails

1. Keep `app/LoomApp.vue` thin. Do not rebuild one stacked mega-panel that mixes Lachesis, Atropos, and Clotho into a single presentational owner.
2. Shared modal behavior belongs in `loom/chrome/LoomDialogShell.vue`; feature-specific copy and actions stay inside the feature namespace.
3. Preserve mythic feature namespaces in directory layout, but keep file names functional and grep-friendly.
4. Atropos and Clotho presentation may exchange state through root-shell props, but they should not import each other's feature components directly as a shortcut.
