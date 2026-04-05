# Route: Constraint

## Trigger

Use when product behavior stays the same, but technical, dependency, performance, interface, or environment boundaries change.

## Primary Owner

- Usually `docs/20-product-tdd/` or `docs/30-unit-tdd/`
- Use `docs/40-deployment/` when the constraint is purely runtime or operational

## Common Mode Overlays

- `Solidify`
- `Execute`
- `Diagnose` when runtime reality diverges from the intended constraint

## Forbidden

- Do not rewrite PRD just to justify an implementation choice.
- Do not hide cross-unit contract changes inside unit-local docs.

## Read-Do Steps

1. Restate which boundary changed and which user-visible behavior must remain stable.
2. Read the smallest relevant Product TDD, Unit TDD, Deployment, and local `AGENTS.md` slices.
3. Update the technical contract in the correct owner layer.
4. If the constraint changes runtime procedure rather than design ownership, route the durable truth into `40-deployment`.

## Exit Criteria

- The changed technical or operational boundary is explicit.
- PRD commitments remain unchanged unless renegotiated.
- Cross-unit and unit-local owners remain consistent.
