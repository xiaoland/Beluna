# Cortex HLD

## Module Boundary

Inputs:
- `ReactionInput`
  - ordered `sense_window` with stable `sense_id`,
  - latest endpoint snapshots,
  - admission feedback signals correlated by `attempt_id`,
  - capability catalog,
  - bounded cycle limits,
  - distributed intent context.

Outputs:
- `ReactionResult`
  - `attempts: IntentAttempt[]` (or empty noop),
  - `based_on: [sense_id...]`,
  - `attention_tags`.

## Key Components

- `CortexReactor`: always-on async inbox loop.
- `CortexPipeline`: single-cycle cognition orchestration.
- Cognition ports:
  - `PrimaryReasonerPort`,
  - `AttemptExtractorPort`,
  - `PayloadFillerPort`.
- `DeterministicAttemptClamp`: final schema/catalog/capability/cap authority.

## Invariants

- Reactor progression is inbox-event driven.
- Cortex is stateless for durable goal/commitment storage.
- `IntentAttempt` remains non-binding and world-relative.
- feedback to Cortex includes `attempt_id` correlation.
- business output remains clean (telemetry is out-of-band).
