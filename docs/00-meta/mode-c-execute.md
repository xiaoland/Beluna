# SOP: Execute

## Role

Use when the current slice of work is clear enough to implement or edit safely.

This mode can appear in any input type once ownership and verification are sufficiently clear. For non-trivial code work, use implementation taste as a projection onto concrete code surfaces, not as a new durable owner.

## Forbidden

- Do not keep exploring in code when scope is still unclear.
- Do not bypass `Diagnose` when the task is really a reality mismatch.
- Do not skip local AGENTS and relevant TDD checks before coding.

## Read-Do Steps

1. Restate the exact change, protected invariants, and verification plan.
2. Load the governing anchors and nearest local `AGENTS.md` files.
3. If the change is risky, reference-sensitive, logic-altering, or not obviously local, perform the Impact Handshake and await human confirmation when required by Beluna gates.
4. Edit the smallest necessary surface area.
5. Verify the change with objective evidence.
6. Return to `Explore` or `Diagnose` if new uncertainty or unexplained behavior appears.

## Exit Criteria

- The intended change is implemented.
- Verification passes.
- No known invariant is violated.
- Any new uncertainty has been routed into the correct next mode.
