# L2 Plan 01 - Module And Boundary Map
- Task Name: `refactor-mind-cortex-non-cortex`
- Stage: `L2` / Part 01
- Date: `2026-02-10`

## 1) Source File Map

### 1.1 Replace `mind` with canonical modules

```text
core/src/
├── cortex/
│   ├── mod.rs
│   ├── error.rs
│   ├── types.rs
│   ├── state.rs
│   ├── commitment_manager.rs
│   ├── planner.rs
│   ├── facade.rs
│   ├── ports.rs
│   └── noop.rs
├── non_cortex/
│   ├── mod.rs
│   ├── error.rs
│   ├── types.rs
│   ├── continuity.rs
│   ├── affordance.rs
│   ├── ledger.rs
│   ├── resolver.rs
│   ├── facade.rs
│   ├── ports.rs
│   ├── debit_sources.rs
│   └── noop.rs
├── spine/
│   ├── mod.rs
│   ├── error.rs
│   ├── types.rs
│   ├── ports.rs
│   └── noop.rs
└── lib.rs
```

### 1.2 Remove legacy module

```text
core/src/mind/*   (removed)
```

## 2) Responsibility Map

1. `core/src/cortex/*`
- deliberative goal lifecycle and decomposition,
- commitment lifecycle management (separate from semantic goal identity),
- non-binding `IntentAttempt` generation,
- no direct execution/economic enforcement.

2. `core/src/non_cortex/*`
- continuity kernel state,
- affordance enforcement (hard/economic/soft),
- survival ledger debit/credit accounting,
- admission resolver and admitted-action materialization.

3. `core/src/spine/*`
- contract-only execution substrate boundary,
- accepts admitted actions, returns execution/feedback events.

## 3) Mechanical Boundary Enforcement Design

1. Non-binding attempts
- `IntentAttempt` remains proposal-like by design.
- there is no spine API accepting attempts.

2. Spine admission gate
- `SpinePort` signature accepts only `AdmittedActionBatch`.
- `AdmittedAction` constructor is restricted to non-cortex resolver module.

3. Type-level no-bypass rule
- no public conversion function from `IntentAttempt` to `AdmittedAction`.
- conversion happens only inside non-cortex admission pipeline.

## 4) Public API Exposure (Core)

`core/src/lib.rs` exports:
1. `pub mod cortex;`
2. `pub mod non_cortex;`
3. `pub mod spine;`
4. keeps unrelated modules unchanged (`ai_gateway`, `protocol`, `server`, etc.).

## 5) Test File Map

```text
core/tests/
├── cortex/
│   ├── mod.rs
│   ├── commitment_manager.rs
│   ├── planner.rs
│   └── facade_loop.rs
├── non_cortex/
│   ├── mod.rs
│   ├── affordance.rs
│   ├── resolver.rs
│   ├── ledger.rs
│   └── debit_sources.rs
├── spine/
│   ├── mod.rs
│   └── contracts.rs
└── cortex_non_cortex_flow.rs
```

Legacy `core/tests/mind/*` is replaced by the new suites.

## 6) Dependency Direction Rules

1. `cortex` may depend on:
- local cortex modules,
- shared primitive crates,
- ports/types from `non_cortex` and `spine`.

2. `non_cortex` may depend on:
- local non-cortex modules,
- `spine` ports/types (contracts only),
- `ai_gateway` telemetry types for debit source adapter only.

3. `spine` may depend on:
- shared primitives and local spine modules.

4. Prohibited for all three:
- direct dependency on concrete body endpoint/runtime implementations in this phase,
- direct dependency on Unix socket server/protocol for internal logic.

## 7) Coupling-Control Matrix

| Concern | Owner | Forbidden Owner |
|---|---|---|
| Goal semantics and decomposition | `cortex` | `non_cortex`, `spine` |
| Continuity/survival authority | `non_cortex` | `cortex` |
| Admission/denial/degradation decision | `non_cortex::resolver` | `spine`, `cortex` |
| Execution dispatch contract | `spine::ports` | `cortex` direct runtime call |
| Global survival budget ledger | `non_cortex::ledger` | `cortex` |
| AI Gateway debit approximation ingestion | `non_cortex::debit_sources` | `cortex`, `spine` |

## 8) Out Of Scope In This Phase

1. Concrete body endpoint execution engine.
2. Socket protocol exposure for cortex/non-cortex/spine.
3. Full-fidelity cost model (v1 is approximate where needed).
