# L2 PLAN 03 - Unit TDD Admission Audit

## Purpose
Assess each current unit against hard-unit admission criteria and recommend Unit TDD profile assignment.

## Criteria (V8/V9-aligned)
A unit is "hard" when at least one materially applies:

1. Non-obvious authority or state ownership.
2. Concurrency, ordering, or timing sensitivity.
3. Subtle failure semantics.
4. Multiple interacting interfaces with meaningful risk.
5. Invariants easy to violate during normal iteration.
6. High blast radius or high change cost.

## Unit Assessment

| Unit | Key Evidence | Verdict | Recommended Profile |
|---|---|---|---|
| `core` | Runtime authority owner; deterministic dispatch ordering; persistence and shutdown semantics; cross-subsystem boundaries (`stem/cortex/continuity/spine`). | Hard | Full Unit TDD contract pack |
| `apple-universal` | Connection lifecycle reliability, protocol compatibility, off-main-thread constraints, persistence + pagination/reconnect failure paths. | Hard | Full Unit TDD contract pack |
| `cli` | Minimal endpoint bridge responsibilities; explicit non-responsibility for core authority; limited local state and operations. | Not hard (current shape) | Lightweight Unit TDD profile |
| `monitor` | Read-only local log viewer; bounded parse/error behavior; no runtime authority ownership in MVP. | Not hard (current shape) | Lightweight Unit TDD profile |

## Recommendation
1. Adopt hard-unit-first admission policy in `docs/30-unit-tdd/index.md`.
2. Require full six-doc pack only for hard units.
3. Allow lightweight profile for straightforward units.
4. Keep existing full docs for `cli` and `monitor` as transitional, but do not require full-pack policy for future straightforward units.

## Migration Safety Note
This policy change updates admission and maintenance burden only. It does not change existing cross-unit contracts or runtime authority ownership.
