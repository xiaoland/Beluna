# Cleanup Landing Plan

This note turns the backend and frontend cleanup targets into one landable sequence.
It is procedural, not authoritative, but it explicitly names the authoritative TDD files that must move with each slice so decisions do not get stranded in `docs/task`.

## Why This Plan Exists

The current Moira code already proves useful operator behavior, but its ownership boundaries are still too soft:

- backend logic is still concentrated around `lib.rs`, `commands.rs`, `state.rs`, `ingest/*`, and `store/*`
- frontend logic is still concentrated around `App.vue`, `api.ts`, `normalize.ts`, and `presenters.ts`
- the current docs already define the cleanup target, but without a concrete landing order the cleanup can easily drift into ad hoc implementation

This plan keeps the cleanup behavior-preserving while ensuring every stable decision has an authoritative home.

## Landing Principles

1. Each slice must be behavior-preserving from the operator point of view unless the slice explicitly says otherwise.
2. Each slice must move task-buffer docs, authoritative TDD, local AGENTS guidance, and code in the same direction.
3. Do not start new Clotho or Atropos operator features before the current Lachesis-heavy structure stops acting as the default owner for everything.
4. Prefer one-directional data flow and one clear owner per durable concern over clever abstractions.
5. Add local `AGENTS.md` only where the complexity boundary is stable and recurring.

## Recommended Local AGENTS Layout

There is already a unit-level [moira/AGENTS.md](/Users/lanzhijiang/Development/Beluna/moira/AGENTS.md).
That file should remain the Moira-wide entrypoint.

The cleanup should add two narrower local AGENTS files once the refactor begins:

1. `moira/src-tauri/src/AGENTS.md`
- preferred first addition
- covers `app / clotho / lachesis / atropos` ownership, thin Tauri command rules, Lachesis-only DuckDB ownership, and backend composition constraints

2. `moira/src/AGENTS.md`
- add when the frontend cleanup starts materially moving files
- covers `bridge / query / projection / presentation`, mythic feature namespaces inside layers, and presentation not becoming a catch-all interpretation layer

Do not add deeper AGENTS files unless one subarea becomes independently hard to work in.
Two narrow files under `src-tauri/src` and `src` are enough for the current complexity.

## Slice 0. Lock The Cleanup Contract

### Objective

Make the cleanup target authoritative enough that implementation does not rely on memory across turns.

### Task-Buffer Changes

- keep [BACKEND-REFACTOR-TARGET.md](/Users/lanzhijiang/Development/Beluna/docs/task/moira-v1/BACKEND-REFACTOR-TARGET.md) and [FRONTEND-REFACTOR-TARGET.md](/Users/lanzhijiang/Development/Beluna/docs/task/moira-v1/FRONTEND-REFACTOR-TARGET.md) aligned
- keep this landing plan updated when slice ordering changes

### Authoritative TDD Touchpoints

- [docs/30-unit-tdd/moira/design.md](/Users/lanzhijiang/Development/Beluna/docs/30-unit-tdd/moira/design.md)
  - keep backend top-level owners authoritative: `app / clotho / lachesis / atropos`
  - keep frontend layer ownership authoritative: `bridge / query state / projection / presentation`
- [docs/30-unit-tdd/moira/operations.md](/Users/lanzhijiang/Development/Beluna/docs/30-unit-tdd/moira/operations.md)
  - keep the cleanup-stage constraint authoritative
- [docs/30-unit-tdd/moira/verification.md](/Users/lanzhijiang/Development/Beluna/docs/30-unit-tdd/moira/verification.md)
  - keep cleanup exit intent authoritative

### Local Guidance

- keep [moira/AGENTS.md](/Users/lanzhijiang/Development/Beluna/moira/AGENTS.md) as the unit-level default
- do not add narrower AGENTS files yet unless implementation is starting immediately

### Done When

- the target module/layer split is no longer only implied by task notes
- the cleanup sequence has one explicit source of truth

## Slice 1. Backend Ownership Extraction

### Objective

Move the current backend into explicit top-level owners without changing current Lachesis behavior.

### Code Focus

- `moira/src-tauri/src/lib.rs`
- `moira/src-tauri/src/commands.rs`
- `moira/src-tauri/src/state.rs`
- `moira/src-tauri/src/ingest/*`
- `moira/src-tauri/src/store/*`
- `moira/src-tauri/src/model.rs`

### Expected Moves

- create `app`, `clotho`, `lachesis`, and `atropos` module roots
- move current OTLP receiver, normalization, DuckDB store, ingest pulse emission, and query APIs under `lachesis`
- make Tauri commands thin façades owned by `app`
- replace the direct `store + receiver` app state with an app-owned container of module handles
- do not add real Clotho or Atropos operator behavior yet beyond module skeletons

### Authoritative TDD Touchpoints

- [docs/30-unit-tdd/moira/data-and-state.md](/Users/lanzhijiang/Development/Beluna/docs/30-unit-tdd/moira/data-and-state.md)
  - update if state homes or persistence wording become more concrete during extraction
- [docs/30-unit-tdd/moira/operations.md](/Users/lanzhijiang/Development/Beluna/docs/30-unit-tdd/moira/operations.md)
  - update if module initialization or readiness wording becomes more concrete
- [docs/30-unit-tdd/moira/verification.md](/Users/lanzhijiang/Development/Beluna/docs/30-unit-tdd/moira/verification.md)
  - update cleanup exit intent if the slice settles more explicit backend evidence

### Local Guidance

- add `moira/src-tauri/src/AGENTS.md`
- keep it narrow:
  - top-level backend ownership
  - Tauri command thinness
  - Lachesis-only storage ownership
  - naming and composition guardrails

### Done When

- backend files no longer imply that Lachesis is the accidental owner of all future behavior
- the current observability commands and `lachesis-updated` event remain behavior-equivalent

## Slice 2. Frontend Bridge And Query Extraction

### Objective

Stop `App.vue` and `api.ts` from owning bridge, live refresh, and selection logic together.

### Code Focus

- `moira/src/App.vue`
- `moira/src/api.ts`
- `moira/src/types.ts`
- `moira/src/main.ts`

### Expected Moves

- introduce `bridge/*` for Tauri `invoke`, event listening, and environment checks
- introduce one query-state owner for wake selection, tick selection, loading state, and refresh timing
- keep the existing Loom behavior and component tree stable while moving responsibilities out of the root file

### Authoritative TDD Touchpoints

- [docs/30-unit-tdd/moira/design.md](/Users/lanzhijiang/Development/Beluna/docs/30-unit-tdd/moira/design.md)
  - update if the query-state boundary becomes concrete enough to deserve stronger wording
- [docs/30-unit-tdd/moira/verification.md](/Users/lanzhijiang/Development/Beluna/docs/30-unit-tdd/moira/verification.md)
  - update cleanup exit intent if the slice settles specific frontend evidence beyond “root views no longer combine everything”

### Local Guidance

- add `moira/src/AGENTS.md` at the start of this slice
- keep it narrow:
  - layer ownership
  - mythic feature namespaces inside layers
  - bridge/query/projection/presentation dependency direction

### Done When

- `App.vue` is no longer the owner of live update wiring and selection logic
- `api.ts` is no longer the owner of normalization and sorting

## Slice 3. Projection And Presentation Split

### Objective

Split the current frontend interpretation layer into maintainable projection and presentation owners.

### Code Focus

- `moira/src/normalize.ts`
- `moira/src/presenters.ts`
- `moira/src/json-sections.ts`
- `moira/src/coerce.ts`
- `moira/src/components/*`
- `moira/src/style.css`

### Expected Moves

- split `normalize.ts` into `projection/lachesis/*`
- separate raw transport contracts from normalized Loom-facing models
- move domain interpretation helpers out of presentation helpers
- organize presentation around:
  - `loom`
  - `lachesis`
  - future `clotho`
  - future `atropos`
- group current Lachesis presentation by operator task:
  - workspace
  - chronology
  - narratives
  - inspectors

### Authoritative TDD Touchpoints

- [docs/30-unit-tdd/moira/design.md](/Users/lanzhijiang/Development/Beluna/docs/30-unit-tdd/moira/design.md)
  - update if the mythic feature-namespace rule or presentation internal split becomes stable enough to stop living only in task docs
- [docs/30-unit-tdd/moira/interfaces.md](/Users/lanzhijiang/Development/Beluna/docs/30-unit-tdd/moira/interfaces.md)
  - update only if the cleanup settles more explicit Loom surface guarantees
- [docs/30-unit-tdd/moira/verification.md](/Users/lanzhijiang/Development/Beluna/docs/30-unit-tdd/moira/verification.md)
  - update cleanup exit evidence once the projection/presentation split has a concrete operator-equivalence check

### Local Guidance

- extend `moira/src/AGENTS.md` if needed
- do not add deeper component-level AGENTS files unless one presentation subtree becomes independently difficult

### Done When

- `normalize.ts` is no longer a single catch-all projection file
- `presenters.ts` no longer mixes domain interpretation and display formatting
- presentation components consume explicit projection outputs instead of reconstructing OTLP meaning ad hoc

## Slice 4. Cleanup Integration Pass

### Objective

Make the backend and frontend cleanup meet cleanly before new feature work resumes.

### Code Focus

- bridge contracts versus Tauri command payloads
- query-state outputs versus projection inputs
- projection outputs versus presentation needs
- backend app state versus frontend bridge expectations

### Expected Moves

- remove transitional duplication left by slices 1 to 3
- align naming and file ownership
- trim any abstractions that survived cleanup without earning their keep

### Authoritative TDD Touchpoints

- [docs/30-unit-tdd/moira/design.md](/Users/lanzhijiang/Development/Beluna/docs/30-unit-tdd/moira/design.md)
- [docs/30-unit-tdd/moira/data-and-state.md](/Users/lanzhijiang/Development/Beluna/docs/30-unit-tdd/moira/data-and-state.md)
- [docs/30-unit-tdd/moira/operations.md](/Users/lanzhijiang/Development/Beluna/docs/30-unit-tdd/moira/operations.md)
- [docs/30-unit-tdd/moira/verification.md](/Users/lanzhijiang/Development/Beluna/docs/30-unit-tdd/moira/verification.md)

Only touch the files above for decisions that proved stable during implementation.
Do not bulk-rewrite them just because the cleanup ended.

### Done When

- cleanup-specific transitional glue is gone
- the code layout now matches the authoritative cleanup target closely enough that new feature work can extend it directly

## Slice 5. First Validating Cross-Slice Feature

### Objective

Prove the cleanup with one real operator flow:

- local source-folder build
- `wake`
- graceful stop

### Code Focus

- `clotho`
- `atropos`
- backend app composition
- frontend bridge/query/presentation surfaces needed for the minimal operator flow

### Authoritative TDD Touchpoints

- [docs/30-unit-tdd/moira/operations.md](/Users/lanzhijiang/Development/Beluna/docs/30-unit-tdd/moira/operations.md)
  - update wake/shutdown wording if the landed flow settles concrete sequencing
- [docs/30-unit-tdd/moira/verification.md](/Users/lanzhijiang/Development/Beluna/docs/30-unit-tdd/moira/verification.md)
  - promote the operator walkthrough or live verification evidence for the validating slice
- [docs/30-unit-tdd/moira/interfaces.md](/Users/lanzhijiang/Development/Beluna/docs/30-unit-tdd/moira/interfaces.md)
  - update only if the minimal Clotho or Atropos command surfaces become concrete and stable

### Done When

- Moira can build a local Core source folder, `wake` it, observe it, and stop it gracefully
- the feature lands on the cleaned-up boundaries instead of punching through them

## What Should Not Happen

1. Do not mix cleanup with new release-management UX.
2. Do not mix cleanup with major visual redesign.
3. Do not introduce a global frontend store or backend service locator unless the current slices clearly prove they are needed.
4. Do not let local AGENTS duplicate the repository root guidance; they should only cover local recurring complexity.
5. Do not postpone TDD updates until the very end; each stable slice must promote its stable boundary decisions while the reasoning is still fresh.

## Immediate Next Slice Recommendation

Start with Slice 1, but prepare Slice 2 in parallel at the document level only.

Reason:

- the backend owner extraction removes the biggest structural blocker for future Clotho and Atropos work
- the current frontend bridge can continue functioning while backend internals move, as long as current commands and payloads remain stable
- once backend ownership is explicit, the frontend bridge/query extraction can land with less guesswork about future command homes
