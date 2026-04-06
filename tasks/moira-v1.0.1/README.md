# Moira v1.0.1 Working Set

This folder is the non-authoritative planning packet for the next Moira update anchored on issue `#26`.

## Working Intent

Moira v1.0.1 is currently defined as one combined follow-on slice:

1. Clotho profiles become Clotho-owned wrapper documents rather than plain Core config files.
2. Wrapper profiles can carry environment sources (`env_files` and inline env vars) together with Core config.
3. Clotho prepares resolved wake input from the wrapper profile; Atropos injects the resolved env when waking Core.
4. Lachesis chronology is no longer treated as a standalone primary view; it becomes one Cortex-focused investigation mode.
5. Cortex View hides unhandled ticks, while broader Lachesis/raw investigation remains available elsewhere.

## Locked Decisions

1. `Profile` is formally upgraded from a pure Core config document to a Clotho-owned wrapper document.
2. Unhandled ticks are hidden from `Cortex View`.
3. No dedicated wake-time preflight is required for environment variables in this slice.

## Current Status

Implemented:

1. Clotho wrapper profiles with `core_config`, `env_files`, and inline environment variables.
2. Atropos wake-time env injection from Clotho-prepared wake input.
3. Loom Profile editor support for wrapper-profile environment editing.
4. Lachesis Cortex View reshape, including timeline-as-mode and handled-tick filtering inside Cortex View.

## Packet Map

- [PLAN.md](/Users/lanzhijiang/Development/Beluna/tasks/moira-v1.0.1/PLAN.md): task plan, contract proposal, workstreams, and verification shape
- [ISSUE-26.md](/Users/lanzhijiang/Development/Beluna/tasks/moira-v1.0.1/ISSUE-26.md): enhancement-template draft used to update GitHub issue `#26`

## Relationship To Prior Work

- [tasks/moira-v1/README.md](/Users/lanzhijiang/Development/Beluna/tasks/moira-v1/README.md) defines the completed v1 floor.
- [tasks/issue-23-clotho-follow-on-20260403/PLAN.md](/Users/lanzhijiang/Development/Beluna/tasks/issue-23-clotho-follow-on-20260403/PLAN.md) is the closest Clotho follow-on packet for wake-input preparation.
- [tasks/diagnose-loom-cortex-events-20260406/PLAN.md](/Users/lanzhijiang/Development/Beluna/tasks/diagnose-loom-cortex-events-20260406/PLAN.md) explains the recent missing-Cortex-events diagnosis that motivates wrapper-profile env support.

## Use Rule

This folder is procedural only.
Durable product or technical truth should move into `docs/20-product-tdd` and `docs/30-unit-tdd/moira` only after the design is confirmed and implemented.
