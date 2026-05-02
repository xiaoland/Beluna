# Case: core.intent_to_act_ack.v1

> Last updated: 2026-05-02
> Status: implemented first-case prototype
> Scope: first Agent Task Case for Core

## MVT Core

- Objective & Hypothesis: Verify the minimal Core agent-task path by proving that a user intent entering afferent input can be converted by Cortex into one act that reaches an endpoint and receives an acknowledged outcome.
- Guardrails Touched: Core authority path must remain real enough to exercise afferent intake, Cortex decision, efferent dispatch, Spine endpoint routing, and terminal dispatch outcome.
- Verification: The case passes when the evidence journal contains the injected sense, exactly one matching received act, and one acknowledged dispatch outcome correlated to that act.

## Exploration Scaffold

- Input Type: Intent plus Artifact.
- Active Mode or Transition Note: Execute slice complete. This file defines the first case now implemented under `core/tests/agent-task`.
- Governing Anchors:
  - `/Users/lanzhijiang/Development/Beluna/tasks/agent-task-testing-20260425/PLAN.md`
  - `/Users/lanzhijiang/Development/Beluna/tasks/agent-task-testing-20260425/CASE-SCHEMA.md`
  - `/Users/lanzhijiang/Development/Beluna/core/AGENTS.md`
  - `/Users/lanzhijiang/Development/Beluna/core/src/cortex/AGENTS.md`
- Impact Hypothesis: A narrow ACK case can validate the runner, evidence journal, endpoint driver boundary, and oracle engine before adding shell/web side effects.
- Temporary Assumptions:
  - The first endpoint driver is `ack_recording_endpoint`.
  - The first case can run in an in-process harness.
  - The first AI boundary is AIMock strict replay.
- Negotiation Triggers:
  - The runner must bypass afferent, Cortex, efferent, or Spine to make the case pass.
  - The case becomes coupled to a specific internal helper rather than the public runtime path.
  - The endpoint driver logic leaks into generic kit primitives.

## Name

`core.intent_to_act_ack.v1`

Why this name:

- `core`: owning unit.
- `intent_to_act_ack`: the behavior under test.
- `v1`: case version.

## Runtime Path Under Test

```text
user intent
  -> afferent
  -> cortex
  -> efferent
  -> spine
  -> ack_recording_endpoint
  -> acknowledged dispatch outcome
```

## Draft Case YAML

```yaml
schema_version: 0
id: core.intent_to_act_ack.v1
title: User intent is converted into an acknowledged act

task:
  user_intent: "Record an audit note with the exact message: hello beluna"
  success_claim: "Core turns the user intent into one acknowledged audit act."
  injected_sense:
    endpoint_id: test.user
    neural_signal_descriptor_id: user.intent
    payload: "Record an audit note with the exact message: hello beluna"
    weight: 1.0

world:
  root: "$CASE_TMP/world"
  files: []
  env: {}
  endpoints:
    - id: audit
      kind: ack_recording_endpoint
      descriptors:
        - type: act
          neural_signal_descriptor_id: task.audit.record
          payload_schema:
            type: object
            required:
              - message
            properties:
              message:
                type: string
      response:
        outcome: acknowledged
        reference_id_template: "ack:{act_instance_id}"
  continuity:
    initial_state: default
  proprioception:
    entries: {}

ai:
  mode: replay
  provider: aimock
  base_url: "http://127.0.0.1:4010/v1"
  fixtures:
    root: "fixtures/agent_tasks/core.intent_to_act_ack.v1"
    routes:
      primary: "primary.json"
      attention: "attention.json"
      cleanup: "cleanup.json"
  normalization:
    strip_timestamps: true
    strip_uuids: true
    strip_run_ids: true
    strip_tmp_paths: true

runtime:
  harness: in_process
  tick_source: manual
  max_ticks: 2
  max_primary_turns: 4
  max_model_calls: 8
  max_acts: 1
  timeout_ms: 10000
  exercised_path:
    - afferent
    - cortex
    - efferent
    - spine
    - endpoint

oracle:
  pass:
    evidence:
      - stream: sense.injected
        match:
          endpoint_id: test.user
          neural_signal_descriptor_id: user.intent
      - stream: act.received
        exact_count: 1
        match:
          endpoint_id: audit
          neural_signal_descriptor_id: task.audit.record
          payload:
            message: "hello beluna"
      - stream: dispatch.outcome
        match:
          endpoint_id: audit
          neural_signal_descriptor_id: task.audit.record
          outcome: acknowledged
          reference_id_prefix: "ack:"
    correlation:
      require_act_instance_id: true
      require_tick: true
  fail:
    forbidden:
      endpoints:
        - shell
        - web
      max_unacknowledged_acts: 0
      dispatch_outcomes:
        - rejected
        - lost

metrics:
  collect:
    - wall_time_ms
    - tick_count
    - model_call_count
    - act_count
    - acknowledged_act_count

artifacts:
  write:
    - result.json
    - evidence.jsonl
    - ai-mock-journal.jsonl
    - diagnostics.log
```

## Implemented Case Location

```text
core/tests/agent-task/cases/core.intent_to_act_ack.v1/
├── case.yaml
└── fixtures/
    └── llm.json
```

The implemented `case.yaml` currently uses a JSON-compatible YAML subset so the first kit can avoid adding a YAML parser dependency. The file remains named `case.yaml` to preserve the case vocabulary.

The AIMock fixture returns one Primary tool-call response:

- `act_audit-1_task-audit-record` with payload `{ "message": "hello beluna" }`
- `break-primary-phase` with empty arguments

Attention and Cleanup receive simple text responses with empty tool-call sets.

## Expected Evidence

Minimum evidence streams:

```json
{"stream":"sense.injected","endpoint_id":"test.user","neural_signal_descriptor_id":"user.intent"}
{"stream":"act.received","endpoint_id":"audit","neural_signal_descriptor_id":"task.audit.record","payload":{"message":"hello beluna"}}
{"stream":"dispatch.outcome","endpoint_id":"audit","neural_signal_descriptor_id":"task.audit.record","outcome":"acknowledged","reference_id":"ack:<act_instance_id>"}
```

## Why This Case Comes First

- It validates the test runner before sandboxed side effects.
- It keeps the oracle simple and mechanical.
- It proves that Core can translate intent into an acknowledged action.
- It creates the first reusable endpoint driver and evidence streams.

## Non-Goals

- No filesystem mutation.
- No shell execution.
- No web fetch.
- No production endpoint added to `core/src/body`.
- No broad DSL.

## Resolved Questions

1. The first implementation uses AIMock immediately.
2. The first in-process harness can be built from existing public Core library APIs.
3. The endpoint driver emits `act.received` and `dispatch.outcome`; the harness emits `sense.injected`, `sense.admitted`, `endpoint.attached`, and `ai.boundary.started`.
4. `endpoint_id: audit` is the logical case endpoint id. The runtime-generated endpoint id is recorded as `runtime_endpoint_id`.

## Verification

Commands:

```bash
cargo test --manifest-path core/Cargo.toml --test agent_task -- --nocapture
cargo test --manifest-path core/Cargo.toml
```

Observed result on 2026-05-02:

- `agent_task`: 1 passed.
- full Core test surface: 1 integration test passed, lib/bin/doc tests have 0 tests.
