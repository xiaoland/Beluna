# L1 PLAN - Strategy and Decision Model

## Strategy
Use a two-axis strategy to prevent policy oscillation:
- Axis A: strictness toward SVCv9 defaults.
- Axis B: preservation of Beluna-specific constraints where they demonstrably reduce risk.

Target output is not forced full convergence; target output is explicit and justified ownership.

## Decision Table (to be filled before edits)
For each drift item, assign one:
- `align_now`
- `align_later`
- `keep_intentional_drift`

Required fields per item:
- reason,
- owner doc(s),
- verification signal.

## Decision Heuristics
1. If V9 and V8 agree and Beluna diverges, default to `align_now`.
2. If V9 and Beluna diverge but Beluna has proven local safety value, allow `keep_intentional_drift` with explicit rationale.
3. If change blast radius is large and not blocking safety/clarity, use `align_later` with dated follow-up.

## Migration Safety Rules
1. Edit ownership statements before editing detailed catalogs.
2. Keep one authoritative owner per truth statement.
3. Prefer wording normalization before structural deletion.
4. Do not land partial cross-layer policy states in one PR.

## Delivery Slices
1. Governance/read-path wording normalization.
2. Canonical semantics ownership reconciliation.
3. Unit TDD admission policy reconciliation.
4. AGENTS scope slimming + restatement protocol.
5. Demotion lifecycle and closure checklist.
