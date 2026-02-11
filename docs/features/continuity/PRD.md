# Continuity PRD

Continuity owns operational state, sensing ingestion, and non-semantic situation construction.

Invairants:

- `SituationView` is non-semantic; cortex is where meaning happens.

Requirements:

- Ingest sense/feedback signals deterministically.
- Build `SituationView` without semantic interpretation.
- Maintain attribution journals and replay-safe state.
