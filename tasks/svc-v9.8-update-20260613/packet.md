# SVC v9.8 Update Task Packet

## MVT Core

- Objective & Hypothesis: Update Beluna's local SVC projection to v9.8 using `../svc` as the reference, while preserving Beluna-specific repository guidance.
- Guardrails Touched: Durable docs remain selective memory; root and meta guidance must stay concise and not create optional alignment docs without current pressure.
- Verification: Key v9.8 terms and entry points are searchable; `tasks/_template.md` matches the upstream v9.8 task packet template except for intentional Beluna-local path context.

## Current State

- Current Understanding: `../svc` is at v9.8, and Beluna already has some v9.8 MVT language but lacks implementation taste, task workspace semantics, search isolation, and updated meta routing hooks.
- User-Confirmed Constraints: User explicitly said "start" after a proposed scope that does not create `docs/15-alignment/`.
- Active Mode or Transition Note: Execute after a read-only Explore and Solidify restatement.
- Next Step: Patch root, meta, and task docs; then verify with targeted search and diff checks.

## Execution Notes

- key findings: Beluna already carried MVT language, but was missing v9.8's task workspace semantics, implementation taste entry point, search isolation, and updated meta hooks.
- decisions made: Do not create `docs/15-alignment/`; reference alignment substrate only as an optional escalation path because Beluna has no current alignment directory.
- final outcome: Root, meta, and task workspace docs were updated; `tasks/_template.md` now matches the upstream v9.8 task packet template.
