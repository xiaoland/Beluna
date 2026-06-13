# Task Packet Template

Use this template as the compact control surface for a task packet.

A packet may live as a single file for small tasks. When it grows, move it into `tasks/<task-id>/packet.md` and split supporting material into adjacent files only where that improves human-agent collaboration.

## MVT Core

- Objective & Hypothesis: <one sentence for the goal and expected result>
- Guardrails Touched: <1-2 rules or boundaries that must not be broken>
- Verification: <objective proof such as tests, logs, structured output, or diff criteria>

## Current State

- Current Understanding: <what the agent currently believes is true>
- User-Confirmed Constraints: <constraints or decisions explicitly confirmed by the human>
- Active Mode or Transition Note: <current mode or why the task changed posture>
- Next Step: <the next concrete action>

## Exploration Scaffold (Optional)

- Perturbation: <what signal started this task>
- Input Type: <Intent | Constraint | Reality | Artifact>
- Governing Anchors: <existing PRD / TDD / AGENTS / deployment docs>
- Impact Hypothesis: <likely downstream effects>
- Temporary Assumptions: <assumptions to validate later>
- Negotiation Triggers: <when to stop and ask for human input>
- Promotion Candidates: <knowledge worth persisting after the task>
- Supporting Files: <index to notes, findings, decisions, verification, or work files>

## Progressive Split Guide

Start single-file when the packet is compact.

Upgrade to directory mode when current state, historical reasoning, evidence, decisions, temporary work, or verification begin to interfere with each other.

Recommended directory-mode starting point:

```text
tasks/<task-id>/
|-- packet.md
|-- notes.md
`-- work/
```

Split `notes.md` by pressure, not ceremony. Common split directions include current versus history, state versus evidence, decision versus exploration, control surface versus work, and summary versus raw output.

## Execution Notes

- key findings:
- decisions made:
- final outcome:
