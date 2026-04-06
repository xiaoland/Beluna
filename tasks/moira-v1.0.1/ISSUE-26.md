## Summary

Moira v1.0.1 should make two coordinated changes:

1. Clotho profiles become Clotho-owned wrapper documents that package Core config together with environment sources.
2. Lachesis chronology is no longer treated as a standalone primary browse surface; it becomes one Cortex View mode, and Cortex View hides unhandled ticks.

## Rationale

The current profile document is still effectively just a Core config file, which makes environment-dependent wake behavior awkward to package and reproduce through Moira.

The current Lachesis selected-tick workspace also overstates chronology as the universal default interpretation for every tick, while the chronology lane view is semantically a Cortex-focused investigation mode.

## Specification

### Wrapper Profiles And Wake Env Injection

Moira profile documents are upgraded from plain Core config files to Clotho-owned wrapper documents.

A wrapper profile contains:

1. `core_config`
2. `environment`

The environment section must support:

1. `env_files`
2. inline environment variables

Clotho resolves a selected wrapper profile into prepared wake input.
That prepared wake input must include:

1. the prepared launch target
2. the wrapper profile reference for operator traceability
3. a Core-readable config path derived from the wrapper profile
4. a resolved environment map ready for process launch

Atropos consumes prepared wake input and injects the resolved environment when waking Core.
Atropos does not become the owner of wrapper-profile parsing or environment-file parsing.

This slice does not require wake-time environment preflight.

### Cortex View And Lachesis Chronology

Lachesis chronology is no longer presented as a standalone primary year-ring view.
It becomes one mode of Cortex-focused investigation.

Cortex View must:

1. focus on ticks with reconstructable Cortex handling evidence
2. hide unhandled ticks
3. use chronology/timeline as one Cortex browsing mode rather than the universal default view

Moira must still preserve broader source-grounded investigation outside Cortex View, including non-Cortex tick diagnosis when operators need raw, Stem, or Spine evidence.

### Acceptance Criteria

- [x] A Clotho profile document is a wrapper document rather than a plain Core config document.
- [x] A wrapper profile can package Core config together with `env_files` and inline environment variables.
- [x] Clotho resolves a selected wrapper profile into prepared wake input that is sufficient for Atropos to wake Core without re-owning profile parsing.
- [x] Atropos injects the resolved environment from prepared wake input when launching Core.
- [x] Cortex View no longer treats chronology as a standalone primary year-ring surface.
- [x] Cortex View hides ticks that have no reconstructable Cortex handling evidence.
- [x] Broader Lachesis inspection still supports source-grounded diagnosis outside Cortex View.

## Technical Constraints

Clotho remains the owner of profile preparation and wake-input derivation.
Atropos remains the owner of supervision and process launch only.
Core remains the authority for Core config schema semantics.
Lachesis remains the owner of observability storage and projection; Clotho and Atropos state must not be folded into Lachesis tables or projections as a shortcut.

## Backward Compatibility

Not required.

## Alternatives Considered

Keeping profiles as plain Core config files plus separate sidecar env management was rejected because it keeps wake preparation split across multiple operator concepts instead of one Clotho-owned profile boundary.

Keeping chronology as a standalone primary Lachesis view was rejected because the chronology lane model is semantically a Cortex investigation mode rather than the universal default reading of every tick.

## Implementation Notes

Implemented evidence:

1. Clotho now parses wrapper profile documents, materializes a Core-readable config beside the wrapper profile, and resolves env overlays from `env_files` plus inline environment variables.
2. Atropos now consumes the prepared runtime wake input and injects resolved environment overrides when spawning Core.
3. Loom Profile editing now exposes structured editing for wrapper-profile environment fields.
4. Lachesis tick summaries now carry a Moira-owned `cortexHandled` predicate, and Cortex View filters its tick timeline to handled ticks only.
5. The previous standalone chronology tab is now a `timeline` mode inside Cortex View.

Verification:

1. `cargo test --lib` in `moira/src-tauri`
2. `pnpm exec vue-tsc --noEmit` in `moira`
3. `pnpm test` in `moira`

## References

- [tasks/moira-v1.0.1/README.md](/Users/lanzhijiang/Development/Beluna/tasks/moira-v1.0.1/README.md)
- [tasks/moira-v1.0.1/PLAN.md](/Users/lanzhijiang/Development/Beluna/tasks/moira-v1.0.1/PLAN.md)
- [tasks/moira-v1/README.md](/Users/lanzhijiang/Development/Beluna/tasks/moira-v1/README.md)
- [tasks/issue-23-clotho-follow-on-20260403/PLAN.md](/Users/lanzhijiang/Development/Beluna/tasks/issue-23-clotho-follow-on-20260403/PLAN.md)
- [docs/30-unit-tdd/moira/design.md](/Users/lanzhijiang/Development/Beluna/docs/30-unit-tdd/moira/design.md)
- [docs/30-unit-tdd/moira/operations.md](/Users/lanzhijiang/Development/Beluna/docs/30-unit-tdd/moira/operations.md)
- [docs/30-unit-tdd/moira/data-and-state.md](/Users/lanzhijiang/Development/Beluna/docs/30-unit-tdd/moira/data-and-state.md)
- [docs/30-unit-tdd/moira/verification.md](/Users/lanzhijiang/Development/Beluna/docs/30-unit-tdd/moira/verification.md)
