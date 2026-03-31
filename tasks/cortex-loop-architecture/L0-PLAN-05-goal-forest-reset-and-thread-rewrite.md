# L0 Plan 05 - Goal Forest Reset and Thread Rewrite
- Task: `cortex-loop-architecture`
- Micro-task: `05-goal-forest-reset-and-thread-rewrite`
- Stage: `L0`
- Date: `2026-03-01`
- Status: `DRAFT_FOR_APPROVAL`

## Objective
Extend goal-forest patching with `reset` semantics that mutate thread history and system prompt deterministically.

## Scope
1. Patch tool accepts `reset: bool`.
2. If `reset=true`:
- delete messages from first user message through current tool call.
- replace goal-forest section in system prompt.
- resend updated system message before continuing.
3. Tool parameters allow optional numbering.
4. Keep numbering auto-generation as future work (not implemented now).

## Current State
1. Goal-forest patch uses natural-language instruction + sub-agent op conversion.
2. No reset parameter.
3. No thread-history surgery API in chat thread surface.

## Target State
1. Reset path is first-class and deterministic.
2. Thread mutation and system prompt replacement are runtime-safe.

## Key Gaps
1. Chat thread/store API lacks message-range delete operations.
2. System prompt replacement lifecycle is not encoded.
3. Reset transaction boundaries are undefined.

## Risks
1. Inconsistent thread state if deletion/reseed is partial.
2. Prompt drift if goal-forest replacement is not canonicalized.

## L0 Exit Criteria
1. Reset transaction semantics are fully specified.
2. Chat-store mutation APIs required by reset are listed.
3. Optional numbering behavior is contractually clear.
