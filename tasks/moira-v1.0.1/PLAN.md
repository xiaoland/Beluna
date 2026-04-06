# Moira v1.0.1 Plan

## MVT Core

- Objective & Hypothesis: define a coherent Moira v1.0.1 slice that upgrades Clotho profiles into wrapper documents with environment support and reshapes Lachesis chronology into a Cortex-focused view; the hypothesis is that this removes current wake fragility around missing runtime env while making Loom's Cortex investigation model more intentional.
- Guardrails Touched: `docs/30-unit-tdd/moira/design.md`; `docs/30-unit-tdd/moira/operations.md`
- Verification: the task packet, issue `#26`, and later code/docs changes all agree on the same external contract for wrapper profiles, prepared wake input, and Cortex View behavior.

## Context

Moira v1 landed three stable owners:

1. `clotho` owns launch-target preparation and profile-document management.
2. `atropos` owns supervision and wake execution.
3. `lachesis` owns OTLP ingest, storage, query, and Loom observability browsing.

Two follow-on pressures now meet in the same slice:

1. The current profile document is still effectively just a Core config file, which makes runtime env requirements awkward to package with wake preparation.
2. The current Lachesis selected-tick workspace still treats chronology as the default browse surface, even though the chronology lane view is semantically a Cortex-focused investigation mode rather than the universal default interpretation for every tick.

## Current Baseline

### Clotho / Atropos

Current code shape:

- `WakeInputRequest` selects a launch target and an optional `profile_id`.
- `ClothoService::prepare_wake_input(...)` resolves the launch target plus profile path.
- `PreparedWakeInput` currently contains:
  - prepared launch target
  - optional `profile_path`
- `AtroposService::wake(...)` launches Core with:
  - executable path
  - working directory
  - optional `--config <profile_path>`

This means the profile document itself must already be a Core-readable config file.

### Lachesis / Loom

Current UI shape:

- `Lachesis` owns wake list, tick timeline, and one selected-tick workspace.
- selected-tick detail tabs are:
  - `chronology`
  - `cortex`
  - `stem`
  - `spine`
  - `raw`
- chronology is the current default selected-tick view.

This shape works for broad tick inspection, but it overstates chronology as the universal primary interpretation instead of one Cortex-oriented reading mode.

## Locked Decisions

1. `Profile` becomes a Clotho-owned wrapper document.
2. Unhandled ticks are hidden from `Cortex View`.
3. No dedicated preflight gate is required for env presence in this slice.

## Proposed External Contract

### 1. Wrapper Profile Document

The durable profile document is no longer “the Core config file”.
It becomes a Clotho-owned wrapper document that packages:

1. `core_config`
2. `environment`

Minimum supported environment sources:

1. `env_files`
2. inline environment variables

Negative rules:

1. No general string-template interpolation contract.
2. No new secret-store requirement in this slice.
3. No promotion of Clotho-specific wrapper fields into Core config authority.

Implication:

- Core schema authority remains with Core.
- Clotho owns the wrapper-document contract and the translation from wrapper profile to wake input.

### 2. Prepared Wake Input

`PreparedWakeInput` should evolve from:

- launch target
- optional profile path

to a shape that can carry:

1. prepared launch target
2. wrapper profile path for operator traceability
3. resolved Core config path to pass into Core
4. resolved env map to inject at wake time

This implies one Clotho-owned preparation step:

- materialize or derive a Core-readable config file from the wrapper document

The important boundary rule is unchanged:

- Atropos consumes prepared wake input
- Atropos does not become the owner of profile parsing, env-file parsing, or config derivation

Atropos responsibility in this slice:

- inject the already resolved env map into the spawned Core process

### 3. Environment Resolution

The intended behavior is:

1. child process still inherits the host environment by default
2. wrapper-provided env files and inline env vars overlay that runtime environment
3. inline env vars win over env-file values when both define the same key

Recommended path semantics:

- relative `env_file` paths resolve relative to the wrapper profile document directory, not the current shell cwd

No preflight requirement means:

- missing env is allowed to fail at runtime
- Moira does not need to guarantee that every declared env source satisfies every Core need before wake

### 4. Cortex View Reshape

Lachesis chronology should no longer be positioned as a standalone primary view.
Instead:

1. chronology becomes one Cortex-focused investigation mode
2. `Cortex View` hides ticks with no reconstructable Cortex handling evidence
3. broader wake/tick/raw browsing remains available for diagnosis

This means “hide unhandled ticks from Cortex View” should not be interpreted as “erase those ticks from Lachesis”.
The intended split is:

1. `Cortex View`
- handled ticks only
- chronology/timeline as one Cortex investigation mode
- Cortex narratives and goal-forest context

2. broader Lachesis inspection
- may still expose all ticks
- keeps Stem / Spine / raw diagnosis available even when Cortex did not handle a tick

### 5. Handled Tick Predicate

For this slice, the practical handled-tick predicate should be rooted in evidence, not heuristics:

- a tick is Cortex-handled when the selected wake/tick contains reconstructable `cortex.*` records needed for Cortex inspection

Minimum family set:

1. `cortex.primary`
2. `cortex.sense-helper`
3. `cortex.goal-forest-helper`
4. `cortex.acts-helper`
5. `cortex.goal-forest`

This should stay a Moira-owned projection rule, not a new Core contract.

## Workstreams

### W1. Clotho Wrapper Profile Contract

Owner focus:

- `moira/src-tauri/src/clotho/*`
- `docs/30-unit-tdd/moira/*`

Likely code touchpoints:

- `moira/src-tauri/src/clotho/model.rs`
- `moira/src-tauri/src/clotho/service.rs`
- `moira/src-tauri/src/clotho/profiles.rs`
- `moira/src-tauri/src/app/commands/clotho.rs`

Likely outcomes:

1. new wrapper profile shape
2. resolved config output
3. resolved env payload in `PreparedWakeInput`

### W2. Atropos Wake Injection

Owner focus:

- `moira/src-tauri/src/atropos/*`

Likely code touchpoints:

- `moira/src-tauri/src/atropos/service.rs`
- `moira/src-tauri/src/atropos/model.rs`

Likely outcomes:

1. `wake(...)` injects resolved env into `Command`
2. runtime status keeps showing the wrapper profile path rather than confusing it with the derived config path

### W3. Loom Cortex View Reshape

Owner focus:

- `moira/src/query/lachesis/*`
- `moira/src/projection/lachesis/*`
- `moira/src/presentation/lachesis/*`

Likely code touchpoints:

- `moira/src/query/lachesis/state.ts`
- `moira/src/query/lachesis/workspace.ts`
- `moira/src/projection/lachesis/ticks.ts`
- `moira/src/presentation/lachesis/workspace/TickDetailPanel.vue`
- `moira/src/presentation/lachesis/workspace/LachesisWorkspacePanel.vue`
- `moira/src/presentation/lachesis/chronology/*`

Likely outcomes:

1. chronology stops being the generic default selected-tick tab
2. Cortex-focused browse mode absorbs the existing chronology/timeline surface
3. unhandled ticks are hidden from Cortex-specific browse entry points

### W4. Verification And Promotion

Likely evidence:

1. wrapper profile round-trip tests
2. env-file and inline-env merge tests
3. wake path tests proving env injection reaches child process launch
4. frontend verification for Cortex View filtering and chronology relocation
5. issue/doc sync

## Implementation Result

Implemented in this slice:

1. Clotho now treats profile documents as wrapper documents and materializes a Core-readable config beside the wrapper profile.
2. Atropos injects Clotho-resolved environment overrides into the spawned Core process while still showing the wrapper profile path in runtime status.
3. Loom Profile editing now exposes structured `environment` editing rather than forcing raw wrapper JSONC edits for env changes.
4. Lachesis tick summaries carry a Moira-owned `cortexHandled` predicate rooted in reconstructable `cortex.*` evidence.
5. Cortex View now absorbs chronology as a `timeline` mode, and its tick timeline hides unhandled ticks while broader Stem / Spine / Raw browsing remains available.

Verification performed:

1. `cargo test --lib` in `moira/src-tauri`
2. `pnpm exec vue-tsc --noEmit` in `moira`
3. `pnpm test` in `moira`

## Design Points To Freeze Before Code

1. Derived config materialization home
- app-local temp/prepared directory or another Clotho-owned transient path

2. Env merge order
- recommended: host inherited env, then `env_files` in listed order, then inline env overrides

3. Wrapper profile editor behavior
- recommended v1.0.1 default: keep raw JSONC editing rather than inventing a structured form editor

4. Cortex View interaction shape
- recommended: do not delete broader Lachesis browse capability; reshape chronology under Cortex investigation rather than collapsing everything into one filtered workspace

## Out Of Scope For This Packet

1. Secret store integration
2. Schema-validation gate before wake
3. Large Lachesis storage redesign
4. Cross-unit Core config schema redesign
5. Removing raw or broader tick inspection from Lachesis entirely

## Backward Compatibility Stance

Not required.

If implementation cost stays low, one temporary compatibility path for existing plain config profile documents may be considered, but it is not a planning gate for this slice.

## Promotion Targets After Confirmation

1. `docs/30-unit-tdd/moira/interfaces.md`
2. `docs/30-unit-tdd/moira/data-and-state.md`
3. `docs/30-unit-tdd/moira/operations.md`
4. `docs/30-unit-tdd/moira/verification.md`
5. if cross-unit wake contract semantics change materially, the relevant `docs/20-product-tdd/*` anchors
