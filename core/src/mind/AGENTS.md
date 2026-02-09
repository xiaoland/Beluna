# AGENTS.md for core/src/mind

This directory implements the Mind layer MVP.

## Design Sources (Authoritative)

- PRD: `../../../docs/features/mind/PRD.md`
- HLD: `../../../docs/features/mind/HLD.md`
- LLD: `../../../docs/features/mind/LLD.md`

## Boundary and Quality Rules

- Keep behavior aligned with contracts in `../../../docs/contracts/mind/*`.
- Keep tests aligned under `../../tests/mind/*`.
- Do not couple Mind core to Unix socket/protocol/runtime modules.
- Keep preemption dispositions explicit: `pause|cancel|continue|merge`.
- Keep evolution proposal-only in MVP.

## Source Surfaces

```text
core/src/mind/
├── mod.rs
├── error.rs
├── types.rs
├── state.rs
├── goal_manager.rs
├── preemption.rs
├── evaluator.rs
├── conflict.rs
├── evolution.rs
├── ports.rs
├── facade.rs
└── noop.rs
```

## Last Updated

> 2026-02-09
