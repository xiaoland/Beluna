# Agent Task Testing Research Notes

> Last updated: 2026-04-27
> Status: exploratory notes
> Scope: external lessons for Beluna Agent Task Based tests

## Source Map

- Karl Lorey, "Without Benchmarking LLMs, You're Likely Overpaying 5-10x": https://karllorey.com/posts/without-benchmarking-llms-youre-overpaying
- Fil-C, "Linux Sandboxes And Fil-C": https://fil-c.org/seccomp
- CopilotKit AIMock docs: https://aimock.copilotkit.dev/docs/
- CopilotKit AIMock Record & Replay: https://aimock.copilotkit.dev/record-replay/
- CopilotKit AIMock Multi-Turn Conversations: https://aimock.copilotkit.dev/multi-turn/
- Sierra tau2-bench repository: https://github.com/sierra-research/tau2-bench
- OSWorld benchmark: https://os-world.github.io/
- WorkArena paper: https://arxiv.org/abs/2403.07718
- SWE-bench repository: https://github.com/SWE-bench/SWE-bench
- Inspect evaluation framework: https://inspect.aisi.org.uk/

## Lessons For Beluna

### 1. Evaluate Beluna On Its Own Tasks

Karl Lorey's benchmark article argues that generic model benchmarks are weak predictors for a specific production task, and that model choice should consider quality, cost, and latency together.

Beluna implication:

- Agent Task Cases should be built from Beluna-relevant operator workflows.
- Every case result should include pass/fail, latency, model-call count, token/cost estimate, tick count, and act count.
- Model/provider selection should eventually use a Beluna-specific Pareto frontier instead of generic leaderboard scores.

### 2. Completion Requires External Evidence

The user concern about coding-agent success being easier than office-agent success maps directly to evaluation design. Coding tasks often have strict feedback loops: compilation, tests, diffs, static checks. General work tasks have fuzzier state, hidden edge cases, and human workflow seams.

Beluna implication:

- A task passes only when the world, endpoint, continuity, or observability evidence proves the success claim.
- Reflection, self-checking, or "reasoning in a loop" is runtime behavior, not verification.
- Cases need explicit forbidden outcomes and budget exhaustion rules so a loop can fail cleanly.

### 3. Model Fixtures Need Deterministic Replay

AIMock is useful because it can mock common LLM APIs, record/replay fixtures, run in strict CI mode, normalize dynamic request parts through transforms, and support multi-turn tool-call flows.

Beluna implication:

- Start with AIMock at the AI Gateway HTTP boundary.
- Use strict replay for CI.
- Use record mode only during fixture authoring.
- Normalize volatile fields such as tick ids, timestamps, UUIDs, run ids, and temporary paths.
- For tool rounds, key fixtures by `toolCallId`; for repeated prompts, use `sequenceIndex` or an explicit predicate.

### 4. Tool-Agent Benchmarks Need State, Policy, And Conversation

tau-bench style systems model realistic data stores, tool APIs, domain policy, and user-agent interaction. This is closer to Beluna's future than single-prompt completion.

Beluna implication:

- A mature Agent Task Case should include task policy, initial world state, allowed tools/endpoints, and expected state transitions.
- A user simulator may become useful later, but first cases can inject a single user intent sense.
- Failures should be classified by where the trajectory broke: interpretation, tool choice, dispatch, world mutation, policy, memory, or observation.

### 5. Reproducible Environments Matter

SWE-bench's harness direction and OSWorld/WorkArena's environment design show the same lesson: agent evaluation quality depends on reproducible, inspectable environments.

Beluna implication:

- Each case should run in a fresh task world directory.
- External services should be local, fixed, and fixture-backed.
- The runner should save a run artifact containing config, fixture ids, world diff, action log, and structured observability summary.
- Long-lived shared state should be opt-in and named in the case.

### 6. Trajectory Evidence Is Needed For Diagnosis

OSWorld reports verified trajectories and analyzes factors like UI observations and history. WorkArena/BrowserGym emphasize rich observations and actions. Inspect records evaluation logs and exposes tasks, datasets, solvers, scorers, tools, agents, limits, and sandboxing as first-class concepts.

Beluna implication:

- Passing cases should still preserve a compact trajectory summary.
- The oracle should separate task success from trajectory quality.
- Runner output should make it easy to compare two runs of the same case.
- Beluna can borrow Inspect's vocabulary: Case/Dataset, Runner/Solver, Oracle/Scorer, Run Log.

### 7. Sandboxing Is Part Of Agent Testing

The Fil-C seccomp article distinguishes memory safety from sandboxing and emphasizes capability reduction, syscall allowlists, resource limits, and applying protections across all runtime threads.

Beluna implication:

- The case world should act as a sandbox boundary.
- Shell and web capabilities should be allowlisted per case.
- File writes should be constrained to the case workspace.
- Violations should terminate the task or produce explicit rejected outcomes.
- Resource limits should be part of the oracle: max ticks, max acts, max wall time, max bytes, max subprocesses.

## Design Consequences

1. Agent Task tests should be few and high-signal.
2. A case is an evaluation sample plus a reproducible world.
3. Oracles should prefer mechanical evidence over prose judgment.
4. The AI boundary should be replayable and inspectable.
5. Safety constraints are test inputs and pass/fail conditions.
6. Cost and latency are outcome metrics.
7. Trace artifacts are required for debugging and benchmark hygiene.

## Open Research Questions

1. Can AIMock exactly serve Beluna's current OpenAI-compatible adapter payloads, including tool calls and streamed responses?
2. Should first cases use AIMock Docker CLI, a programmatic Node harness, or a small Rust-owned fixture server with AIMock reserved for later?
3. Which sandbox mechanism is practical on macOS for shell endpoint task worlds?
4. Which observability events are mandatory in the first compact trajectory artifact?
5. Should Beluna adopt an external eval runner such as Inspect later, or keep the first runner native Rust?
