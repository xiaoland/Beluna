# Frontend Refactor Target

This note defines the cleanup-stage frontend landing target for `moira/src`.
It is a procedural buffer anchored on the current Moira Unit TDD, not an authoritative contract by itself.

## Purpose

- turn the Unit TDD Loom split into a concrete frontend target
- keep the current Lachesis-backed Loom surfaces stable during cleanup
- keep layer-oriented ownership while still allowing mythic names where they clarify product-facing surfaces
- prevent `App.vue`, `api.ts`, and `normalize.ts` from remaining catch-all owners as Clotho and Atropos surfaces arrive later

## Cleanup-Stage Boundary

The cleanup stage is behavior-preserving.
It should not change the current operator-facing Loom surfaces:

- keep the current wake list
- keep the current tick timeline
- keep the current selected-tick detail tabs:
  - chronology
  - cortex
  - stem
  - spine
  - raw
- keep the current live refresh path driven by `lachesis-updated`
- keep raw-event inspection as the source-grounded fallback surface
- do not use cleanup work as a vehicle for broad UX redesign, new operator flows, or visual restyling unrelated to architectural cleanup

## Frontend Layer Target

The frontend target remains the Unit TDD split:

1. `bridge`
- owns Tauri `invoke`, event subscription, and environment detection only
- does not normalize payloads, sort domain records, or hold selection state

2. `query state`
- owns wake selection, tick selection, refresh orchestration, loading state, and live-update reactions
- does not reinterpret OTLP families inline or render view structure directly

3. `projection`
- owns normalization, chronology reconstruction, interval pairing, narrative shaping, and drilldown summaries
- owns the frontend-facing models consumed by presentation
- does not talk to Tauri directly

4. `presentation`
- owns Vue components, tab composition, dialogs, JSON inspectors, formatting helpers, and visual tokens
- consumes query-state outputs and projection outputs rather than reconstructing observability meaning inline

The durable ownership axis is still the layer split above.
Mythic names are a feature/surface axis inside those layers, not a replacement for them.

## Target Frontend Shape

The target stays layer-oriented, but mythic names can still appear inside the layers as product-facing feature namespaces.
That gives us both maintainability and semantic clarity:

- layers answer "who owns this kind of work?"
- mythic names answer "which Moira surface is this for?"

```text
moira/src/
  main.ts
  app/
    LoomApp.vue
  bridge/
    lachesis.ts
    clotho.ts
    atropos.ts
    events.ts
    env.ts
    contracts/
      lachesis.ts
      clotho.ts
      atropos.ts
  query/
    lachesis/
      workspace.ts
      selection.ts
      refresh.ts
    clotho/
      workspace.ts
    atropos/
      workspace.ts
  projection/
    lachesis/
      models.ts
      receiver.ts
      wakes.ts
      ticks.ts
      raw-events.ts
      chronology.ts
      narratives/
        cortex.ts
        stem.ts
        spine.ts
        ai.ts
      labels.ts
      json-sections.ts
    clotho/
      models.ts
    atropos/
      models.ts
  presentation/
    loom/
      chrome/
      layout/
      shared/
    lachesis/
      workspace/
      chronology/
      narratives/
      inspectors/
    clotho/
      artifacts/
      profiles/
    atropos/
      wake/
      supervision/
    format.ts
    theme.css
```

Notes:

- `App.vue` may remain the root file name if preferred, but it should behave like `app/LoomApp.vue`, not like a combined app/query/projection owner.
- `bridge`, `query`, `projection`, and `presentation` are the durable frontend layers.
- `lachesis`, `clotho`, and `atropos` can appear inside those layers as feature namespaces.
- the main cleanup goal is not more files for their own sake; it is to make data flow one-directional and ownership obvious.
- feature namespaces should be introduced only where there is real ownership or navigation value, not copied mechanically into every directory.

## Current Code To Target Mapping

### Root And Query Orchestration

- `moira/src/App.vue`
  - target owner: `app` plus `query`
  - current issue: it mixes bridge subscription, async loading, selection persistence, refresh timing, and root presentation composition
  - target direction:
    - keep root layout composition in `app/LoomApp.vue`
    - move wake/tick selection and refresh orchestration into one query-state owner such as `query/lachesis/workspace.ts`

### Bridge

- `moira/src/api.ts`
  - target owner: `bridge`
  - current issue: it mixes Tauri transport with normalization and sorting
  - target direction:
    - `bridge/lachesis.ts` handles `invoke`
    - `bridge/events.ts` handles `lachesis-updated`
    - `bridge/env.ts` handles Tauri presence checks
    - `bridge/contracts/*` defines raw transport payload types where useful

### Projection

- `moira/src/normalize.ts`
  - target owner: `projection`
  - current issue: one 927-line file owns normalization, chronology construction, interval pairing, subsystem partitioning, narrative shaping, and sorting
  - target direction:
    - split by stable transformation responsibility rather than by arbitrary line count
    - keep the current Lachesis-heavy projection code under `projection/lachesis/*`
    - likely seams:
      - `receiver.ts`
      - `wakes.ts`
      - `ticks.ts`
      - `raw-events.ts`
      - `chronology.ts`
      - `narratives/*`
      - `labels.ts`

- `moira/src/types.ts`
  - split across `bridge/contracts/*` and `projection/*/models.ts`
  - current issue: raw transport concerns and normalized frontend models are fused into one shared type bucket

- `moira/src/json-sections.ts`
  - target owner: `projection`
  - current role already fits reasonably well
  - likely landing path: move beside other projection helpers without expanding responsibility

- `moira/src/coerce.ts`
  - likely remains a low-level projection utility
  - current role already fits reasonably well as a shared data coercion helper

### Presentation

- `moira/src/presenters.ts`
  - split across `projection` and `presentation`
  - current issue: it mixes domain-derived labels and summaries with formatting and tone helpers
  - target direction:
    - keep domain interpretation helpers such as narrative summaries and raw-event headlines in `projection`
    - keep display formatting such as time/count formatting and tone mapping in `presentation/format.ts`

- `moira/src/components/*`
  - target owner: `presentation/*`
  - current components are generally close to the target boundary
  - current issue: some components still depend on mixed helper modules that carry projection responsibilities
  - likely landing path:
    - `StatusHeader.vue` -> `presentation/loom/chrome`
    - `WakeSessionList.vue`, `TickTimeline.vue`, `TickDetailPanel.vue` -> `presentation/lachesis/workspace`
    - `TickChronology.vue`, `ChronologyLaneTrack.vue`, `ChronologyEntryDialog.vue` -> `presentation/lachesis/chronology`
    - `RawEventInspector.vue`, `JsonSectionGroup.vue` -> `presentation/lachesis/inspectors`

- `moira/src/style.css`
  - target owner: `presentation`
  - may remain a single stylesheet during cleanup
  - rename or relocate only if that improves ownership clarity without churning the styling surface

## Presentation Internal Target

`presentation` is the most complex frontend layer and should be designed explicitly rather than treated as "the remaining components" bucket.

Its internal split should be:

1. `loom`
- owns application chrome, shell layout, panel framing, shared empty/loading/error presentation, and global visual vocabulary
- should not own Lachesis-specific chronology or inspection behavior

2. `lachesis`
- owns the current observability workspace because that is the only frontend surface substantially realized today
- should group by operator task rather than by raw backend DTO shape
- likely subareas:
  - `workspace`
  - `chronology`
  - `narratives`
  - `inspectors`

3. `clotho`
- reserved for future artifact/profile preparation surfaces
- should not be forced into existence beyond placeholders until those operator flows actually land

4. `atropos`
- reserved for future wake/stop/supervision surfaces
- should stay separate from Lachesis browsing even though operators will move between them

5. shared presentation utilities
- formatting, lightweight visual helpers, and generic UI primitives that are not domain owners
- should remain small; if a utility starts encoding Lachesis meaning, it belongs back in projection or in a Lachesis presentation namespace

## Ownership Rules

### `bridge`

- may know Tauri command names and event names
- may not know chronology rules or selection policy
- should return raw payloads or minimally adapted transport responses

### `query state`

- may depend on `bridge` and `projection`
- may hold selected wake, selected tick, loading flags, and refresh timers
- may not reconstruct domain narratives inline inside the state owner

### `projection`

- may consume raw bridge payloads and emit Loom-facing models
- may own chronology pairing, subsystem grouping, and narrative section assembly
- may not subscribe to Tauri events or own async refresh loops

### `presentation`

- may know tabs, popups, layout, formatting, and component interaction details
- may not call `invoke` directly
- may not become the fallback home for ad hoc event interpretation merely because a component needs one more label
- may use mythic feature namespaces freely, but those namespaces must sit inside clear presentation ownership boundaries

## Data-Flow Principles

1. Data flows one way: `bridge -> query state -> projection -> presentation`.
2. Live update reactions start in query state, not in presentation components.
3. Projection is the only frontend owner that turns OTLP-shaped detail into Loom-specific chronology and narratives.
4. Presentation renders normalized models and structured sections; it should not need to read raw payload internals except through explicit projection outputs.
5. Bridge remains thin enough that future Clotho and Atropos commands can join it without importing Lachesis projection code.
6. Presentation may compose by mythic surface, but data ownership still flows through the layer split first.

## Landing Sequence

1. Introduce `bridge`, `query`, `projection`, and `presentation` directories without changing operator behavior.
2. Extract Tauri `invoke`, event listening, and environment detection from `App.vue` and `api.ts` into `bridge`.
3. Create one query-state owner for wake selection, tick selection, refresh orchestration, and loading state.
4. Move normalization and chronology logic out of `api.ts` and `App.vue`, then split `normalize.ts` by stable transformation seams.
5. Split `presentation` into Loom chrome plus Lachesis feature areas before introducing any Clotho or Atropos screens.
6. Separate display formatting from domain interpretation in the current `presenters.ts`.
7. Leave components behavior-equivalent while changing their dependencies to query-state outputs and projection outputs.
8. Only then expand new Loom surfaces for Clotho and Atropos.

## First Validating Outcome After Cleanup

The cleanup should preserve the current Lachesis operator workflow while making one thing obvious in code:

- `App.vue` is no longer the owner of live update wiring and selection logic
- `api.ts` is no longer the owner of normalization
- `normalize.ts` is no longer a single catch-all projection file

If those three pressure points remain, the cleanup did not actually land.

## Explicit Non-Goals For This Refactor

- no forced symmetry with backend mythic naming
- no broad visual redesign
- no replacement of the current Loom interaction model
- no introduction of a global store abstraction unless the current query-state needs clearly justify it
- no component explosion where small presentational files become harder to navigate than the current code

## Immediate Follow-Through

After this target is accepted, the next useful discussion should be the concrete cleanup sequence across both sides:

- backend first moves that unblock the current frontend bridge
- frontend first moves that reduce `App.vue` and `normalize.ts`
- the minimum behavior-preserving slice that can be landed without mixing cleanup with new Clotho or Atropos features
