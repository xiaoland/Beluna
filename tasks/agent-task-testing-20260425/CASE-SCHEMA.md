# Agent Task Case Schema Draft

> Last updated: 2026-05-02
> Status: exploratory schema draft with implemented subset
> Scope: Beluna Core Agent Task Based tests

## Design Claim

An Agent Task Case is the smallest durable test unit for Beluna Core.

It combines:

- a user-visible task,
- a controlled world,
- a deterministic AI boundary,
- a bounded Core runtime run,
- a mechanical oracle,
- and a compact trajectory artifact.

The case runner should be file-driven. Adding an ordinary case should add YAML and fixtures, not custom Rust assertions.

The kit should provide primitive machinery:

- case loading,
- world setup,
- Core harness lifecycle,
- AI boundary control,
- endpoint driver attachment,
- evidence journaling,
- oracle evaluation,
- budget enforcement,
- artifact writing.

Endpoint behaviors are drivers. The first driver is `ack_recording_endpoint`, which records incoming acts into generic evidence and returns an acknowledged outcome. It is not a special oracle type and it is not a production body endpoint.

Agent Task Cases verify Core's ability to solve tasks with available capabilities. Endpoint implementation correctness belongs to endpoint-level contract tests and setup preflight. Case pass criteria should stay as orthogonal as practical: endpoint and dispatch evidence should support diagnosis when another case already covers that Core path.

## Implemented Subset

The first runnable subset lives under:

```text
core/tests/agent-task/
```

Implemented primitives:

- `CaseLoader`: loads `cases/*/case.yaml` through the current JSON-compatible YAML subset.
- `AiBoundary`: starts AIMock strict replay through `@copilotkit/aimock` and exposes a local OpenAI-compatible `/v1` base URL.
- `EndpointDriver`: supports `ack_recording_endpoint`.
- `EvidenceJournal`: stores generic events in memory and writes `evidence.jsonl`.
- `OracleEngine`: evaluates evidence stream matchers, exact counts, reference id prefix checks, and basic correlation requirements.
- `ArtifactWriter`: writes `result.json` and `evidence.jsonl`.

Current case fixture shape:

```text
cases/<case-id>/
├── case.yaml
└── fixtures/
    └── llm.json
```

The first implementation intentionally keeps YAML parsing to a JSON-compatible subset to avoid adding a parser dependency before the case language stabilizes.

## AI Modes

Agent Task runner should support:

```yaml
ai:
  mode: replay
```

and:

```yaml
ai:
  mode: live
```

`replay` mode uses deterministic AIMock fixtures and is the starting point for CI regression.

`live` mode uses a real configured AI Gateway route and a real LLM. It exists to measure whether Beluna's act descriptors, sense payloads, Cortex prompts, IR shape, and phase orchestration enable a model to solve the task. Live runs should write evaluation artifacts with success/failure, cost, latency, model-call count, act count, and failure classification.

## Endpoint Boundary

Agent Task Case setup may run endpoint preflight checks. A failed preflight should classify the run as `invalid_environment`.

Examples:

- shell endpoint can create a probe file inside the case workspace
- web endpoint can fetch a local probe URL
- endpoint descriptor catalog contains the expected capability

The main case oracle should avoid re-testing endpoint internals. Endpoint act payloads, dispatch outcomes, endpoint result senses, and transcripts should still be captured as diagnostic evidence.

## Orthogonal Oracle Principle

Each case should contribute one primary proof.

Examples:

- `core.intent_to_act_ack.v1`: primary proof is the minimal intent-to-ACK evidence path.
- `core.shell_write_file_world_diff.v1`: primary proof should be workspace file state.

This keeps failures easier to classify and prevents every Agent Task Case from growing into a broad duplicate of the same dispatch-path checks.

## Minimal YAML Shape

```yaml
schema_version: 0
id: core.shell.write_file.v1
title: Create a file through the shell endpoint

task:
  user_intent: "Create notes/hello.txt with the exact text: hello beluna"
  success_claim: "The requested file exists with exact content."
  policy_refs: []

world:
  root: "$CASE_TMP/world"
  files:
    - path: notes
      kind: directory
  env:
    BELUNA_TASK_WORKSPACE: "$CASE_TMP/world"
  endpoints:
    - id: shell
      kind: inline_shell
      allow:
        commands:
          - /bin/sh
        writable_paths:
          - "$CASE_TMP/world"
    - id: audit
      kind: ack_recording_endpoint
      allow:
        descriptors:
          - task.audit.record
  continuity:
    initial_state: null
  proprioception:
    entries: {}

ai:
  mode: replay # replay | live
  provider: aimock
  fixtures:
    root: fixtures/agent_tasks/core.shell.write_file.v1
    routes:
      primary: primary.json
      attention: attention.json
      cleanup: cleanup.json
  normalization:
    strip_timestamps: true
    strip_uuids: true
    strip_run_ids: true
    strip_tmp_paths: true

runtime:
  harness: in_process
  max_ticks: 3
  max_primary_turns: 4
  max_model_calls: 8
  max_acts: 2
  timeout_ms: 10000

oracle:
  pass:
    world:
      files:
        - path: notes/hello.txt
          content_exact: "hello beluna"
    diagnostics:
      evidence_streams:
        - act.received
        - dispatch.outcome
        - shell.exec.result
  fail:
    forbidden:
      paths_outside_world: true
      endpoints:
        - web
      dispatch_outcomes:
        - lost

metrics:
  collect:
    - wall_time_ms
    - model_call_count
    - token_estimate
    - act_count
    - tick_count
```

## Required Fields

1. `id`
- Stable case id.
- Should include unit, domain, task, and version.

2. `task.user_intent`
- The exact user-facing instruction injected as a sense or input.

3. `task.success_claim`
- One sentence describing the observable result.

4. `world`
- Defines all mutable state the agent can affect.
- The runner should treat omitted capabilities as unavailable.

5. `ai`
- Defines replay/record/live mode and fixtures.
- CI should use replay mode.

6. `runtime`
- Defines budgets and harness type.

7. `oracle`
- Defines pass/fail criteria.
- Early cases should rely on mechanical checks.

## Oracle Classes

### World Oracle

Checks external state:

- file exists,
- file content exact match,
- local service state,
- endpoint-owned record,
- database row,
- absence of mutation.

This is the preferred primary oracle when the task changes the world.

### Dispatch Oracle

Checks Core act results:

- acknowledged,
- rejected with reason,
- lost with reason,
- act id correlation,
- endpoint id,
- descriptor id,
- payload shape.

This is a required supporting oracle for action-taking cases.

### Evidence Oracle

Checks generic evidence streams:

- `sense.injected`,
- `act.received`,
- `dispatch.outcome`,
- `model.call`,
- `world.diff`,
- `observability.event`,
- `budget.event`.

This is the preferred shape for endpoint-driver-backed cases because it keeps oracle logic independent from a specific driver implementation.

### Continuity Oracle

Checks persisted cognition state:

- revision changed or stayed stable,
- goal forest changed as expected,
- context reset happened,
- restart restore succeeds.

Use only when the task claims memory or continuity behavior.

### Observability Oracle

Checks structured trace reconstructability:

- expected event families exist,
- run id and tick correlation exist,
- act/sense correlation exists,
- terminal outcome is inspectable.

This supports diagnosis and Moira-facing trust. It should not be the sole proof for task completion.

### Policy Oracle

Checks constrained behavior:

- forbidden endpoint not used,
- path boundary respected,
- unsafe act rejected,
- budget respected,
- human approval requested if required.

This is required for tasks that touch shell, filesystem, network, or external endpoint behavior.

## Runner Lifecycle

1. Allocate a fresh case temp directory.
2. Materialize `world`.
3. Start AIMock or configured AI mock in replay/record mode.
4. Start local services and endpoint harnesses.
5. Build Core config from the case.
6. Start Core harness.
7. Inject the user intent sense.
8. Advance ticks until pass, fail, timeout, or budget exhaustion.
9. Stop Core and all harness services.
10. Evaluate oracle.
11. Write run artifact.

## Run Artifact Shape

```text
case-runs/<case-id>/<timestamp>/
├── case.yaml
├── result.json
├── world-before.json
├── world-after.json
├── world-diff.json
├── dispatch-log.jsonl
├── ai-mock-journal.jsonl
├── observability-summary.json
└── diagnostics.log
```

## Result JSON Shape

```json
{
  "case_id": "core.shell.write_file.v1",
  "status": "passed",
  "failure_class": null,
  "metrics": {
    "wall_time_ms": 821,
    "tick_count": 1,
    "model_call_count": 3,
    "act_count": 1,
    "token_estimate": 1842
  },
  "evidence": {
    "world": ["notes/hello.txt content_exact"],
    "dispatch": ["shell acknowledged"],
    "observability": ["cortex.primary", "stem.efferent", "spine.act"]
  }
}
```

## Failure Classes

- `task_not_completed`
- `wrong_world_state`
- `forbidden_world_mutation`
- `unexpected_endpoint`
- `dispatch_rejected`
- `dispatch_lost`
- `ai_fixture_miss`
- `budget_exhausted`
- `timeout`
- `observability_missing`
- `harness_error`

## Fixture Policy

1. Exploratory recordings live under the task packet until stable.
2. Stable fixtures move to `core/tests/fixtures/agent_tasks/<case-id>/`.
3. CI uses AIMock strict mode.
4. Record mode is local-only.
5. Fixture changes require a case result artifact explaining why the expected trajectory changed.

## Harness Decision

Initial leaning:

- Start with `in_process` for faster iteration and easier state inspection.
- Graduate selected cases to `process` harness once schema and first task oracle settle.

The process harness should eventually validate:

- config loading,
- binary startup,
- shutdown,
- filesystem paths,
- observability export wiring.

## First Candidate Case

`core.intent_to_act_ack.v1`

Reason:

- The success oracle is concrete and mechanical.
- The task validates `user intent -> afferent -> cortex -> efferent -> act acknowledged`.
- The endpoint is a test driver, so the first runner avoids shell/web sandbox complexity.
- It creates the generic evidence streams needed by later world-mutation cases.

Risks to resolve before implementation:

- The harness must exercise the real Core authority path rather than direct endpoint invocation.
- The endpoint driver must stay a plugin, not a kit primitive.
- Logical endpoint ids must be reconciled with runtime-generated body endpoint ids.

Case packet:

- [`CASE-core.intent_to_act_ack.v1.md`](./CASE-core.intent_to_act_ack.v1.md)

## Second Candidate Case

`core.shell.write_file.v1`

Reason:

- Adds a real world mutation oracle after the ACK runner is stable.
- Introduces shell sandbox and path confinement pressure.

## Third Candidate Case

`core.safety.reject_forbidden_path.v1`

Reason:

- Tests safe failure as a first-class product behavior.
- Validates sandbox and policy oracle.
- Complements the happy-path world mutation case.

## Decision Gates Before Coding

1. Accept the minimal schema fields.
2. Choose AIMock integration mode: Docker CLI, Node programmatic wrapper, or Rust-owned fixture server.
3. Choose first case: shell write-file or inline spy endpoint.
4. Define the minimum run artifact.
5. Decide where stable fixtures live.
