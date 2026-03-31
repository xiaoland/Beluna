# Hard Constraints

## Product-Shaping Constraints

1. No hidden fallback behavior for routing or configuration interpretation.
2. Deterministic behavior is preferred at system boundaries where ambiguity creates operational risk.
3. Product truth must stay separated from mechanism truth:
- PRD governs product behavior.
- TDD/deployment governs implementation and runtime mechanisms.
4. Body endpoints must not duplicate core runtime authority responsibilities.
5. Task workspace (`tasks/`) remains non-authoritative and separate from authoritative docs.

## Delivery Constraints

1. Documentation must stay maintainable for a small team.
2. Stable truths should have one authoritative home to avoid drift.
