# Motor MVP Verification Notes

## Core Topology Correction Verification

2026-06-14:

Passed:

```bash
cargo check --manifest-path core/Cargo.toml
cargo test --manifest-path core/Cargo.toml --lib --bins
```

Full `cargo test --manifest-path core/Cargo.toml` reached Agent Task tests but
failed because AIMock did not become ready on its health endpoint. The failure
was:

```text
AIMock did not become ready at http://127.0.0.1:<port>/health
AIMock exited before readiness check: exit status: 190
```

Core topology implementation evidence is recorded in
[core-topology-correction/IMPLEMENTATION-RESULT.md](./core-topology-correction/IMPLEMENTATION-RESULT.md).

## Test Hypothesis

Motor MVP is successful only if Motor participation measurably improves an end-to-end task that remains largely Cortex-shaped.

The first useful comparison:

1. Cortex-only baseline.
2. Cortex plus Motor routines that are created / activated through Motor lifecycle Acts, then react to matched Senses, update explicit activation state, and emit procedural Acts.

## Candidate Agent Task Test

Task family:

- Given an existing Markdown or HTML slide artifact and a sequence of edit requests, produce a final artifact that remains renderable and satisfies objective checks.

The Motor routine boundary should not be "understand the user conversation".

Better candidate routine boundaries:

- inspect and summarize artifact structure
- apply a bounded structural edit plan
- preserve slide delimiters / frontmatter / code fences
- run renderability validation and report structured failures
- repair local structural defects from validator output

## Candidate Metrics

- success rate across repeated runs
- artifact renderability
- structural preservation
- edit accuracy against requested changes
- unnecessary diff size
- number and type of Motor routine invocations
- number and type of active routine triggers from Afferent Senses
- number and type of routine state transitions
- number and type of Motor lifecycle Act interceptions
- number and type of Motor-emitted Acts
- number and type of routine-produced Senses
- failure attribution:
  - Cortex planning failure
  - Motor claim/routing failure
  - Motor routine selection failure
  - Motor execution failure
  - dispatch failure
  - artifact validation failure

## Observability Requirements

Minimum lineage chain:

- Cortex cycle id
- Cortex lifecycle Act id claimed by Motor
- Motor activation id
- Motor routine id
- triggering Sense ids
- previous and next routine state summaries
- Motor-emitted Act ids
- terminal dispatch outcomes
- resulting afferent senses / artifact validation outputs

This attribution is required so a passing acceptance result cannot be incorrectly credited only to prompt changes.
