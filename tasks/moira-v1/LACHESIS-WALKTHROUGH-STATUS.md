# Lachesis V1 Walkthrough Status

This note records what was actually verified for the current Lachesis v1 working set and what still remains pending.
It is procedural, not authoritative.

## Verified In This Turn

- `tasks/moira-v1/L2.md`, `OPEN-QUESTIONS.md`, `L3.md`, and `LACHESIS.md` now align to the current canonical family names used by:
  - `core/src/observability/contract/mod.rs`
  - `moira/src/projection/lachesis/*`
- The task buffer now explicitly distinguishes:
  - current canonical family names
  - historical draft vocabulary
  - historical fixture-id naming residue
- `cargo check` passed in `moira/src-tauri`.
- `pnpm -C moira build` passed.
- Operator-reported live desktop-shell walkthrough on `2026-04-02` confirmed:
  1. Moira started through the Tauri desktop shell
  2. a real local Core executable was registered through the Clotho control surface
  3. Atropos woke the supervised Core successfully
  4. Lachesis materialized a new wake and related OTLP logs
  5. Atropos stopped the supervised Core successfully

## Live Walkthrough Still Pending

The minimal live walkthrough is no longer missing.
What still remains is fuller browse-surface evidence for the already-landed Lachesis operator workspace.

Pending walkthrough path:

1. confirm wake list materialization explicitly from Loom
2. confirm tick timeline materialization explicitly from Loom
3. inspect one selected tick through:
   - chronology
   - cortex
   - stem
   - spine
   - raw

## Why It Is Still Pending

- The current gap is no longer “missing live walkthrough evidence for wake/stop”.
- The remaining gap is narrower: the operator-reported walkthrough established the supervision loop and OTLP ingest, but this note does not yet capture a full selected-tick inspection pass across every current Lachesis tab.

## Exit Condition For This Note

This note can be retired once the current Moira workspace has one written walkthrough that covers both:

1. the already-proven live wake/ingest/stop loop
2. one explicit selected-tick browse pass through the current Lachesis workspace
