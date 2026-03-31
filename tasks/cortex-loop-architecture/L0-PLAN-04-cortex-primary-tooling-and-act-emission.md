# L0 Plan 04 - Cortex Primary Tooling and Act Emission
- Task: `cortex-loop-architecture`
- Micro-task: `04-cortex-primary-tooling-and-act-emission`
- Stage: `L0`
- Date: `2026-03-01`
- Status: `DRAFT_FOR_APPROVAL`

## Objective
Replace prompt-parsed act emission with structured Primary tool-calling and bounded wait semantics.

## Scope
1. Remove prompt-based act dispatch path (`<somatic-acts>` + `acts_helper` pipeline).
2. Remove act descriptor catalog from Primary system prompt.
3. Define act emission tool schema with:
- `payload`
- `wait_for_sense: integer` with constraints `>=0` and `<= max_waiting_seconds`.
  - `0` means no wait.
4. Support optional emitted-sense ids in act descriptor.
5. Merge expansion tools into one:
- `expand-senses(mode: raw | sub-agent, ...)`.
6. Align runtime handling of wait semantics with bounded timeout only.

## Current State
1. Acts are parsed from text sections and materialized by helper.
2. `wait_for_sense` is bool output-ir flag, not per-act bounded integer.
3. Two separate expand-sense tools exist.

## Target State
1. All act emission is structured via tool calls.
2. Wait behavior is per-act, bounded, and integer-driven.
3. Single expand-senses tool handles both modes.

## Key Gaps
1. Primary tool schemas and execution routing changes.
2. Removal of legacy acts helper and related prompt sections.
3. Timeout-driven wait handling integration with hybrid loop.

## Risks
1. Tool-call misuse can stall loop without strict schema validation.
2. Migration period can duplicate act paths if not atomically switched.

## L0 Exit Criteria
1. Structured act tool contract is complete and unambiguous.
2. Legacy prompt-based act path is scheduled for full removal.
3. Bounded integer wait policy is encoded in runtime contract.
