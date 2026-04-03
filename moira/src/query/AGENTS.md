# AGENTS.md for moira/src/query

## Scope

- This file applies to the frontend query layer under `moira/src/query`.
- Follow [../AGENTS.md](/Users/lanzhijiang/Development/Beluna/moira/src/AGENTS.md), [../../AGENTS.md](/Users/lanzhijiang/Development/Beluna/moira/AGENTS.md), and the repository root AGENTS first.

## Query Ownership

1. `loom/navigation`
- Owns active top-level Loom station state and shell-local navigation only.

2. `lachesis/*`
- Owns receiver refresh orchestration, wake selection, tick selection, active Lachesis detail tab, and live-update reactions for observability browsing.

3. `atropos/runtime`
- Owns runtime polling, wake/stop/force-kill orchestration, and supervision-related confirmation state exposed to presentation.

4. `clotho/builds`
- Owns current selected build ref plus build-registration dialog state for the current session.

5. `clotho/profiles`
- Owns profile-library refresh/load/save orchestration, current selected profile ref, and profile-dialog state for the current session.

## Local Rules

1. Query may call `bridge` and immediately normalize through `projection`, but it must not become the owner of backend-shaped contract types or display formatting.
2. Keep session-local selection state here until docs explicitly promote it to persistence. Do not smuggle selected build or profile refs into backend storage as a convenience shortcut.
3. Do not recreate one shared `control` query module. New behavior belongs under the owning mythic namespace or `loom/navigation` if it is truly shell-wide.
4. If a query module starts shaping operator narratives, labels, or raw-event summaries, move that work back into `projection`.
5. Presentation should receive stable query intents such as `wakeSelectedBuild` or `openProfileEditor`; it should not manually coordinate multi-step refresh or wake flows.

## Guardrails

1. Do not let `lachesis/workspace` absorb Clotho or Atropos orchestration merely because it already owns live updates.
2. Do not let `atropos/runtime` start resolving artifact or profile paths, editing profile documents, or owning preparation persistence. That belongs to Clotho.
3. Do not let `clotho/*` start normalizing runtime or observability semantics. That belongs to Atropos or Lachesis plus projection.
4. Introduce shared query helpers only when two feature owners genuinely need the same orchestration semantics; otherwise keep the ownership boundary explicit.
