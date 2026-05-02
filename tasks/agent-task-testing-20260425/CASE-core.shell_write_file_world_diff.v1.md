# Case: core.shell_write_file_world_diff.v1

> Last updated: 2026-05-02
> Status: replay implemented; live entry added
> Scope: second Agent Task Case for Core

## MVT Core

- Objective & Hypothesis: Verify that Core can solve a user-visible file creation task by using a trusted shell capability and leaving a mechanical workspace-state proof.
- Guardrails Touched: Core should exercise real afferent intake, Cortex reasoning, AI Gateway, efferent dispatch, Spine routing, and the production shell capability, while the pass oracle stays focused on task result.
- Verification: The case passes when `notes/hello.txt` exists under the case workspace and its content is exactly `hello beluna`.

## Scope Boundary

This case is about Core completing a task with a trusted endpoint capability.

Endpoint correctness is handled by endpoint contract tests and setup preflight. If shell preflight cannot create a probe file inside the case workspace, the run result should be `invalid_environment`.

Dispatch outcome, shell result sense, act payload, and model-call transcript are diagnostic evidence for failure classification. They should not be required pass criteria for this case.

## AI Modes

The same case should support two run modes:

```yaml
ai:
  mode: replay
  provider: aimock
```

and:

```yaml
ai:
  mode: live
  provider: configured_gateway
```

`replay` is the deterministic regression mode. `live` is the capability evaluation mode for real model reasoning, descriptor quality, sense design, and Cortex prompt/IR effects.

The implemented live entry is ignored by default and requires:

```bash
BELUNA_AGENT_TASK_CONFIG=/path/to/beluna.jsonc
BELUNA_AGENT_TASK_CASE=core.shell_write_file_world_diff.v1
cargo test --manifest-path core/Cargo.toml --test agent_task -- --ignored --nocapture
```

## Runtime Path Under Test

```text
user intent
  -> afferent
  -> cortex
  -> AI Gateway
  -> efferent
  -> spine
  -> std-shell endpoint
  -> workspace file mutation
```

## Draft Case Shape

```yaml
schema_version: 0
id: core.shell_write_file_world_diff.v1
title: Shell capability creates the requested workspace file

task:
  user_intent: "Create notes/hello.txt with the exact text: hello beluna"
  success_claim: "The requested file exists in the case workspace with exact content."
  injected_sense:
    endpoint_id: test.user
    neural_signal_descriptor_id: user.intent
    payload: "Create notes/hello.txt with the exact text: hello beluna"
    weight: 1.0

world:
  root: "$CASE_TMP/world"
  files:
    - path: notes
      kind: directory
  endpoints:
    - id: shell
      kind: std_shell
      preflight:
        create_probe_file: true
  continuity:
    initial_state: default
  proprioception:
    entries: {}

ai:
  mode: replay
  provider: aimock
  fixtures:
    root: fixtures

runtime:
  harness: in_process
  tick_source: manual
  max_ticks: 4
  max_primary_turns: 4
  max_model_calls: 8
  max_acts: 2
  timeout_ms: 10000

oracle:
  pass:
    files:
      - path: notes/hello.txt
        content_trimmed_exact: "hello beluna"
  diagnostics:
    evidence_streams:
      - act.received
      - dispatch.outcome
      - shell.exec.result
      - model.call
```

## Implementation Notes

- Use the production shell execution path.
- Keep the case workspace in a fresh temp directory.
- Keep file oracle primitives small: `CaseWorkspace`, `FileTreeSnapshot`, `FileTreeDiff`, `FileExpectation`, and `WorkspaceBoundary`.
- Treat workspace file state as the primary proof.
- Use shell and dispatch evidence for debugging and failure classification.

## Implemented Case Location

```text
core/tests/agent-task/cases/core.shell_write_file_world_diff.v1/
├── case.yaml
└── fixtures/
    └── llm.json
```

Replay fixture behavior:

- Calls `act_shell-1_shell-exec`.
- Uses `$CASE_WORKSPACE` as the rendered `cwd`.
- Executes `mkdir -p notes && printf 'hello beluna' > notes/hello.txt`.
- Calls `break-primary-phase` in the same Primary turn.

## Verification

Commands:

```bash
cargo test --manifest-path core/Cargo.toml --test agent_task -- --nocapture
cargo test --manifest-path core/Cargo.toml
```

Observed result on 2026-05-02:

- `agent_task`: replay passed, live entry ignored.
- full Core test surface: replay passed, live entry ignored, lib/bin/doc tests have 0 tests.
- latest shell replay artifact recorded `world-diff.json` with `created: ["notes/hello.txt"]`.
- live runs with workspace `beluna.jsonc` and `.env` wrote `o11y-contract-events.jsonl` plus `ai-gateway-summary.json`.
- live observation: one run corrected argv redirection by using `sh -c`, then wrote outside the case workspace because the runner had not bound the shell default cwd to `world.root`; the runner now binds cwd to `world.root`.
- live observation after cwd binding: one run spent tick 1 on an invalid `expand-senses` call, tick 2 on `mkdir`, and tick 3 on argv redirection; the case tick budget is now 4 so the model has one correction tick after receiving the shell result.
- live observation after 4 ticks: the model created `notes/hello.txt` in the case workspace with `hello beluna\n`; the file oracle now uses `content_trimmed_exact` because this case targets task-level file content, not final newline formatting.
- latest live run passed on 2026-05-02 with `tick_count: 2`, `world-diff.json` showing `created: ["notes/hello.txt"]`, and 32 captured `observability.contract` events.
