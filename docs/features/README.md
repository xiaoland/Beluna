# README of docs/features

Feature docs answer "What to do".

Each feature package should include:

- PRD:
  - product intent and user-facing behavior
  - user stories, flow, acceptance criteria
  - (optional) domain glossary for feature-local terms
- HLD:
  - architecture, boundaries, and integration surfaces
- LLD:
  - contracts, invariants, and implementation-level constraints

Notes:

- Should reference `docs/modules/*` to avoid duplications.

Index:

- [AI Gateway](./ai-gateway/PRD.md)
- [Cortex](./cortex/PRD.md)
- [Non-cortex](./non-cortex/PRD.md)
- [Spine](./spine/PRD.md)
- [Mind (deprecated)](./mind/PRD.md)
