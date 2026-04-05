# Route: Intent

## Trigger

Use when the business wants new behavior, scope, policy, or strategy.

## Primary Owner

- `docs/10-prd/`

## Common Mode Overlays

- `Explore`
- `Solidify`
- `Execute`

## Forbidden

- Do not encode mechanism, topology, or interface details in PRD.
- Do not skip impact review against existing claims, workflows, scope, and glossary terms.

## Read-Do Steps

1. Restate the intended behavior change and success signal.
2. Inspect the impacted `_drivers/`, `behavior/`, and `glossary.md` slices.
3. Update PRD first so the changed claim, workflow, rule, or scope boundary is explicit.
4. Promote downstream technical implications into `20-product-tdd` or `30-unit-tdd` only after product truth is stable.

## Exit Criteria

- PRD reflects the new or revised product truth.
- Impact on existing product claims is explicit.
- Business vocabulary remains consistent.
