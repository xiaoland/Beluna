# Cortex Contracts

Related:
- Goal forest detailed contract and manual checklist: `./goal-forest.md`

## Boundary

1. Input: `Sense[]`, `PhysicalState`.
2. Output: `CortexOutput { control, pending_primary_continuation }`.

## Contract Details

1. Cortex does not return emitted acts at runtime boundary; act emission is executed inside Primary tool handlers via efferent pathway.
2. `CortexOutput.control` supports:
   - `ignore_all_trigger_for_ticks`
   - `wait_for_sense` (act correlation + expected fq-sense ids + wait ticks)
3. `CortexOutput.pending_primary_continuation=true` means the last assistant turn ended with `tool_calls`; runtime must inject those tool-result messages into the next admitted tick turn, together with that tick's user input.
4. Primary executes one AI Gateway thread turn per cortex cycle (no internal micro-loop).
5. Primary tool-call surface includes:
   - dedicated per-act tools (transport-safe alias mapped to fq act id),
   - `expand-senses`,
   - `patch-goal-forest`,
   - `add-sense-deferral-rule`,
   - `remove-sense-deferral-rule`,
   - `sleep`.
6. Sense expansion tool contract:
   - `mode: raw | sub-agent`
   - `senses_to_expand[].sense_id` uses `"<monotonic-id>. <fq-sense-id>"`.
7. Goal-forest mutations are applied through `patch-goal-forest` tool calls, and cognition persistence is done by direct Continuity calls from Cortex.
8. Proprioception must be refreshed from physical state before every Primary turn dispatch.
