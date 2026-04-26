# Agent Task Testing Exploration

> Last updated: 2026-04-25
> Status: exploratory discussion packet
> Scope: core task-level verification strategy

## MVT Core

- Objective & Hypothesis: Define a small, repeatable way to verify Beluna Core by Agent Task completion, so the development loop measures useful agent behavior with auditable runtime evidence.
- Guardrails Touched: Core remains the runtime authority for cognition, continuity, dispatch, body endpoint interaction, and observability; AI mocking must sit at the AI Gateway boundary while preserving real Core orchestration.
- Verification: This packet is complete when it captures the testing frame, candidate harness shape, open questions, and decision gates clearly enough to continue discussion without jumping into implementation.

## Exploration Scaffold

- Perturbation: Existing core tests are drifting from current APIs, and many component-level tests risk becoming maintenance load or vanity coverage while the product is still moving quickly.
- Input Type: Intent plus Artifact.
- Active Mode or Transition Note: Explore. The current work is to frame the problem and standardize discussion before implementation planning.
- Governing Anchors:
  - `/Users/lanzhijiang/Development/Beluna/AGENTS.md`
  - `/Users/lanzhijiang/Development/Beluna/core/AGENTS.md`
  - `/Users/lanzhijiang/Development/Beluna/core/src/cortex/AGENTS.md`
  - `/Users/lanzhijiang/Development/Beluna/tasks/README.md`
  - `/Users/lanzhijiang/Development/Beluna/docs/00-meta/mode-a-explore.md`
  - `/Users/lanzhijiang/Development/Beluna/docs/00-meta/input-artifact.md`
  - `/Users/lanzhijiang/Development/Beluna/docs/30-unit-tdd/core/verification.md`
- Impact Hypothesis: A task-level harness can replace broad low-signal component testing as the primary business feedback loop, while a smaller set of contract tests remains for hard protocol and state invariants.
- Temporary Assumptions:
  - Beluna should be evaluated as an agent through standardized task completion, not through broad component pass counts.
  - The useful test unit is an Agent Task Case with a controlled world, scripted model behavior, executable Core runtime path, and explicit oracle.
  - CopilotKit AIMock is a strong candidate for model-boundary mocking because it supports OpenAI-compatible style record/replay and CI-friendly fixtures.
  - First useful Core task tests should drive the real AI Gateway, Cortex, Continuity, Efferent, Spine, and body endpoint surfaces as much as practical.
- Negotiation Triggers:
  - The harness starts testing only mocks and bypasses Core authority paths.
  - The case oracle becomes subjective or depends on free-form model prose.
  - Test setup cost grows faster than task signal.
  - The proposed task case requires durable product or cross-unit contract claims before those claims are accepted.
- Promotion Candidates:
  - Agent Task Case schema.
  - Core task runner boundary.
  - AI Gateway mock fixture policy.
  - Minimal task oracle rules.

## Current Read

Core currently has a standard Cargo test surface and several BDT-style integration entrypoints under `/Users/lanzhijiang/Development/Beluna/core/tests`.

Observed baseline from 2026-04-25:

- `cargo check --manifest-path core/Cargo.toml` passes.
- `cargo test --manifest-path core/Cargo.toml --no-run` fails because existing tests reference older APIs and data models.
- `cargo test --manifest-path core/Cargo.toml --lib --no-run` fails in inline test code for body and Unix socket paths.
- `cargo test --manifest-path core/Cargo.toml --test observability_contract` compiles, then fails because fixtures still reference older observability family names such as `cortex.tick`.

Interpretation:

- The runtime code can compile.
- The existing test suite is a mixed asset: it contains useful historical intent, stale API assumptions, and likely cleanup candidates.
- Restoring every old test as a gate would pull attention toward test-suite preservation instead of agent behavior quality.

## Core Testing Principle

Beluna Core should be tested primarily as an agent runtime.

The primary question:

> Given a task and a controlled world, can Beluna complete the task through its real runtime authorities and leave evidence that an operator can audit?

This shifts the main gate from component interaction to task completion evidence.

## Agent Task Case Shape

Candidate schema:

```text
AgentTaskCase
├── task
│   ├── id
│   ├── user_intent
│   └── success_claim
├── world
│   ├── initial_files
│   ├── endpoint_capabilities
│   ├── initial_continuity_state
│   └── initial_proprioception
├── ai
│   ├── mock_provider
│   ├── fixtures
│   └── allowed_routes
├── runtime
│   ├── tick_budget
│   ├── turn_budget
│   ├── act_budget
│   └── timeout_budget
├── oracle
│   ├── world_diff
│   ├── dispatch_records
│   ├── continuity_state
│   └── observability_events
└── hazards
    ├── forbidden_acts
    ├── forbidden_endpoints
    └── budget_exhaustion_rules
```

Notes:

- `task.success_claim` should be concrete and operator-facing.
- `oracle.world_diff` should carry the strongest proof when the task is meant to change the world.
- `dispatch_records` should prove terminal outcomes, including ACK / reject / lost paths where relevant.
- `continuity_state` should prove memory or goal-state effects only when the task claims persistence.
- `observability_events` should prove reconstructability, not become the only proof.

## AI Mock Candidate

Candidate: CopilotKit AIMock.

Useful source anchors:

- https://aimock.copilotkit.dev/docs
- https://aimock.copilotkit.dev/multi-turn

Why it fits the current discussion:

- It can act as a model-boundary fixture layer.
- It supports multi-turn tool-call scenarios.
- It can support record/replay workflows for tests that begin as exploratory runs.
- It keeps the model behavior deterministic enough for CI-style task verification.

Questions to settle:

- Whether AIMock can serve Core's exact OpenAI-compatible wire expectations with minimal glue.
- How to route Primary, Attention, Cleanup, sense helper, and acts helper through separate fixtures.
- How strict fixture matching should be during early product iteration.
- Whether recorded fixtures belong under `core/tests/fixtures/agent_tasks` or task-local fixtures until stable.

## Harness Boundary Options

### Option A: Black-Box Core Process

Run `beluna` as a child process with temporary config, AIMock endpoint, temp working world, and local body endpoints.

Signal:

- Highest realism for runtime composition and config.
- Strongest rehearsal for future CI and release gates.

Cost:

- More process control, ports, temp directories, shutdown handling, and log capture.

### Option B: In-Process Core Runtime Harness

Construct Core runtime modules in a Rust integration test and wire AIMock plus local endpoints in-process.

Signal:

- Good runtime coverage with faster iteration.
- Easier state inspection and deterministic assertions.

Cost:

- Some startup/config behavior receives separate coverage.

### Option C: Hybrid First Step

Start with in-process Agent Task Case execution, then graduate selected cases to black-box process runs once the case schema is stable.

Signal:

- Good first discussion-to-code path.
- Allows task oracle design to mature before process orchestration expands.

Current leaning:

- Option C.

## First Task Candidates

### Candidate 1: Create A File Through Shell Endpoint

Task:

> Given a user intent sense asking Beluna to create a named file with known content, Core should dispatch a shell act that creates the file and records an acknowledged outcome.

Strong oracle:

- File exists with exact content.
- Shell endpoint received one expected act.
- Dispatch outcome is acknowledged.
- Related act id and emitted sense are correlated.
- Observability includes reconstructable cortex/stem/spine activity for the task.

Why it is useful:

- It validates agent task completion through a visible world change.
- It exercises act selection, dispatch, body endpoint, terminal outcome, and audit surfaces.

Risks:

- Shell can hide too much behavior inside one command.
- Fixture prompts may overfit to one exact act.

### Candidate 2: Fetch A Local Web Page And Persist A Summary

Task:

> Given a user intent sense asking Beluna to inspect a local web page, Core should call the web endpoint and persist a concise task result.

Strong oracle:

- Local server received expected request.
- Web endpoint result sense has expected status and body snippet.
- Continuity state records the intended task result if product semantics support that claim.

Why it is useful:

- It exercises perception through a body endpoint and follow-up state handling.

Risks:

- Product semantics for persistence may still be fluid.

### Candidate 3: Reject An Unsafe Or Unsupported Act

Task:

> Given a model proposal that targets an unavailable or forbidden endpoint, Core should produce a terminal rejection and preserve auditability.

Strong oracle:

- No world mutation.
- Dispatch outcome is rejected with explicit reason.
- Observability reconstructs the rejection path.

Why it is useful:

- It tests safety and failure semantics early.

Risks:

- It validates guardrail behavior more than task completion.

## Discussion Questions

1. What is the smallest operator-visible task that captures Beluna's product promise today?
2. Which evidence should count as the primary oracle: world diff, dispatch outcome, continuity state, observability trace, or a weighted combination?
3. Should first Agent Task Cases use real shell/web endpoints, inline spy endpoints, or both?
4. Should the first model fixture be hand-authored, recorded from a live run, or generated through AIMock record/replay?
5. Which existing tests become mandatory contracts after the Agent Task gate exists?
6. Which existing tests can move to archive or deletion once their intent is represented by task cases?
7. How strict should case replay be while Cortex phase contracts are still evolving?

## Possible Next Discussion Output

The next useful artifact is a short `CASE-SCHEMA.md` or a section in this file defining:

- required fields
- fixture location
- oracle classes
- task lifecycle
- allowed harness shortcuts
- promotion criteria from exploratory task case to CI gate

## Execution Notes

- Key findings:
  - Runtime compile succeeds while broad test compilation is stale.
  - Agent Task completion is the right verification center for Beluna's current product shape.
  - AIMock is a credible candidate for deterministic model-boundary behavior.
- Decisions made:
  - This packet remains exploratory.
  - Implementation planning waits for more discussion.
- Final outcome:
  - Pending.
