# Agent Task Testing Exploration

> Last updated: 2026-05-02
> Status: exploratory packet with first runnable prototype
> Scope: core task-level verification strategy

## MVT Core

- Objective & Hypothesis: Define a small, repeatable way to verify Beluna Core by Agent Task completion, so the development loop measures useful agent behavior with auditable runtime evidence.
- Guardrails Touched: Core remains the runtime authority for cognition, continuity, dispatch, body endpoint interaction, and observability; AI mocking must sit at the AI Gateway boundary while preserving real Core orchestration.
- Verification: This packet is complete when it captures the testing frame, candidate harness shape, open questions, decision gates, and the first runnable Core Agent Task Case.

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
  - AIMock should be the first AI boundary backend for replay fixtures.
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
  - Agent task kit primitive model and endpoint driver boundary.

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

This packet now contains:

- [`RESEARCH-NOTES.md`](./RESEARCH-NOTES.md): external benchmark, eval, AIMock, and sandboxing lessons.
- [`CASE-SCHEMA.md`](./CASE-SCHEMA.md): current Agent Task Case schema and runner/oracle draft.
- `/Users/lanzhijiang/Development/Beluna/core/tests/agent-task`: first runnable Core Agent Task test kit and case.

## Design Direction Snapshot

Agent Task Based tests should verify externally observable task completion in a constrained world.

The test case should answer five questions:

1. What task did the user ask Beluna to complete?
2. What world was Beluna allowed to touch?
3. Which Core authorities were exercised?
4. What evidence proves completion or safe failure?
5. What did the task cost in ticks, model calls, tokens, time, and actions?

The primary oracle should be a world-state or runtime-state fact, such as file content, endpoint state, dispatch outcome, continuity state, or structured observability. LLM-as-judge may be useful for auxiliary scoring, but it should not be the only pass/fail criterion for early Core gates.

The design should borrow from benchmark systems without adopting their full machinery:

- From task-specific LLM benchmarking: evaluate on Beluna's real prompts/tasks and track quality, cost, and latency together.
- From tool-agent benchmarks: model task state, policy, tool APIs, and multi-turn interaction as part of the case.
- From software-agent benchmarks: make environments reproducible and require native objective checks.
- From computer-use benchmarks: keep trajectory evidence for diagnosis because task success alone hides brittle behavior.
- From sandbox engineering: make the world boundary explicit, small, and enforceable before running agent-produced actions.

## Current Implementation Posture

After `/Users/lanzhijiang/Development/Beluna/tasks/core-test-cleanup-20260426`, Core has no remaining tests. The next test surface should start from Agent Task Cases rather than restoring historical component tests.

## Key Design Decisions

1. Core library exports
- Core is not binary-only in practice: `src/main.rs` bootstraps through the `beluna` library crate.
- Agent Task tests should first use the existing public Core library API.
- Do not add an `agent-task-testkit` feature by default.
- Add public exports only when a missing runtime-composition primitive is also a coherent Core library API.

2. Agent Task test location
- Agent Task tests belong under `core/tests/agent-task/`.
- They are black-box integration/task tests, not inline unit tests.
- Cargo exposes them through a single explicit test target named `agent_task`.

3. Test kit role
- The test kit should provide primitive evaluation machinery rather than case-specific helpers.
- Endpoint behavior should be pluggable drivers.
- The first recording/ACK endpoint is only the first endpoint driver, not a special concept baked into the kit.

4. Case-driven execution
- New task cases should be YAML-driven.
- Adding a normal case should not require new Rust test code.
- Rust code changes should be reserved for new endpoint drivers, oracle primitives, world primitives, AI-boundary adapters, or runner lifecycle changes.

5. AI boundary
- Agent Task runner should support both `replay` and `live` AI modes.
- `replay` mode uses AIMock strict replay through the published `@copilotkit/aimock` package and points Core's OpenAI-compatible AI Gateway backend at the local AIMock `/v1` URL.
- `live` mode uses the real configured AI Gateway route and a real LLM. It is the mode for evaluating whether endpoint act descriptors, sense payloads, Cortex prompts, IR shape, and phase orchestration actually help a model solve the task.
- AIMock fixtures are case-local and loaded from the case directory.
- CI should start with `replay`; `live` should begin as manual or scheduled evaluation with cost, latency, model-call count, act count, success rate, and failure classification artifacts.

6. Agent Task scope boundary
- Agent Task Case verifies Core's ability to solve a task using available capabilities.
- Endpoint implementation correctness belongs to endpoint-level contract tests and case setup preflight.
- If a trusted endpoint cannot perform its basic declared capability during preflight, the run result should be `invalid_environment`.
- Endpoint execution details, dispatch outcomes, shell result senses, and act payloads should be kept as diagnostics unless the case explicitly targets that Core path.

7. Orthogonal oracle principle
- Each Agent Task Case should keep its pass criteria focused on the behavior it uniquely contributes.
- `core.intent_to_act_ack.v1` verifies the minimal intent-to-ACK path.
- `core.shell_write_file_world_diff.v1` should focus on workspace file state as the primary proof and use dispatch/shell evidence for diagnosis.

## Proposed Test Tree

```text
core/tests/agent-task/
├── main.rs
├── cases/
│   └── core.intent_to_act_ack.v1/
│       ├── case.yaml
│       └── fixtures/
│           └── llm.json
├── kit/
│   ├── ai.rs
│   ├── case.rs
│   ├── endpoints.rs
│   ├── evidence.rs
│   ├── oracle.rs
│   └── runner.rs
└── CASE-core.intent_to_act_ack.v1.md
```

The implemented Cargo target is:

```toml
[[test]]
name = "agent_task"
path = "tests/agent-task/main.rs"
```

## Kit Primitive Model

The kit should be built around these primitives:

1. `CaseLoader`
- Reads YAML cases.
- Validates schema version and required fields.

2. Workspace and file-state primitives
- `CaseWorkspace` creates a fresh case workspace and resolves case-local paths.
- `FileTreeSnapshot` captures workspace-local file state before and after the run.
- `FileTreeDiff` describes file mutations.
- `FileExpectation` checks path existence, absence, and exact content.
- `WorkspaceBoundary` rejects path expectations outside the case workspace.

3. `CoreHarness`
- Builds the in-process Core runtime slice from public Core library APIs where possible.
- Injects user intent into afferent input.
- Advances ticks under budgets.

4. `AiBoundary`
- Starts and controls AIMock replay.
- Supplies the OpenAI-compatible base URL to Core AI Gateway.
- Handles normalization for volatile request fields.

5. `EndpointDriver`
- Registers a test endpoint with Core.
- Emits evidence into the journal.
- Converts endpoint behavior into structured outcomes.

6. `EvidenceJournal`
- Records generic evidence streams:
  - `sense.injected`
  - `act.received`
  - `dispatch.outcome`
  - `model.call`
  - `world.diff`
  - `observability.event`
  - `budget.event`

7. `OracleEngine`
- Evaluates case rules against evidence and file-state primitives.
- Produces failure class and compact evidence summary.

8. `BudgetGuard`
- Enforces max ticks, acts, model calls, wall time, and optional token/cost estimates.

9. `ArtifactWriter`
- Writes `result.json`, evidence journal, world diff, AI journal, and diagnostics.

Endpoint drivers should look conceptually like:

```rust
#[async_trait]
trait EndpointDriver {
    fn kind(&self) -> &'static str;

    async fn attach(
        &self,
        harness: &mut CoreHarness,
        spec: EndpointSpec,
        journal: EvidenceJournal,
    ) -> anyhow::Result<AttachedEndpoint>;
}
```

The first endpoint driver should be named `ack_recording_endpoint`: it records incoming acts into `act.received` evidence and returns an acknowledged dispatch outcome.

## Next Case

The next dedicated case file is:

- [`CASE-core.shell_write_file_world_diff.v1.md`](./CASE-core.shell_write_file_world_diff.v1.md)

Its purpose is to validate that Core can use a trusted shell capability to complete an externally observable file-writing task.

Primary oracle:

```text
workspace file state:
  notes/hello.txt exists
  content equals "hello beluna"
```

Diagnostic evidence:

```text
act payload
dispatch outcome
shell.exec.result sense
model-call trajectory
```

## First Case

The first dedicated case file is:

- [`CASE-core.intent_to_act_ack.v1.md`](./CASE-core.intent_to_act_ack.v1.md)

Its purpose is to validate:

```text
user intent -> afferent -> cortex -> efferent -> act acknowledged
```

It deliberately avoids shell/web world mutation so the first runner can prove the task pipeline and evidence/oracle model before adding sandboxed side effects.

## Execution Notes

- Key findings:
  - Runtime compile succeeds while broad test compilation is stale.
  - Agent Task completion is the right verification center for Beluna's current product shape.
  - AIMock is a credible candidate for deterministic model-boundary behavior.
  - Task-specific benchmarks must track cost and latency as first-class outcomes, not only pass/fail.
  - Agent loops need external evidence and bounded action budgets; self-check loops are insufficient as verification.
  - Sandbox and capability boundaries are part of the test subject for action-taking agents.
- Decisions made:
  - This packet remains exploratory at the strategy level.
  - The first runnable prototype is implemented under `core/tests/agent-task`.
  - Core library exports stay public-runtime first; no testkit feature by default.
  - Agent Task kit primitives should stay generic, with endpoint behavior modeled as drivers.
  - The first AI boundary uses AIMock strict replay from `@copilotkit/aimock`.
- Implemented prototype:
  - `case::CaseLoader` loads case-local `case.yaml` files using the current JSON-compatible subset.
  - `ai::AimockBoundary` starts AIMock and provides Core with a local `/v1` base URL.
  - `endpoints::AckRecordingEndpoint` records `act.received` and `dispatch.outcome`.
  - `EvidenceJournal` stores generic evidence events.
  - `OracleEngine` evaluates evidence match rules and correlation requirements.
  - `AgentTaskRunner` composes Core in-process through public library APIs and writes `result.json` plus `evidence.jsonl`.
- Verification commands:
  - `cargo test --manifest-path core/Cargo.toml --test agent_task -- --nocapture`
  - `cargo test --manifest-path core/Cargo.toml`
- Final outcome:
  - First runnable Agent Task test kit passes with `core.intent_to_act_ack.v1`.
