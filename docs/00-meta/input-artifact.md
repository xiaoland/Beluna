# Route: Artifact

## Trigger

Use when the request is for a bounded intermediate deliverable such as a script, migration helper, report, one-off analysis output, or temporary support asset.

## Primary Owner

- `tasks/`
- The local work surface when the artifact clearly belongs next to the code it supports

## Common Mode Overlays

- `Execute`
- `Explore` when the artifact shape is still unclear

## Forbidden

- Do not turn disposable tactics into durable architecture without evidence.
- Do not promote an artifact into durable docs just because it was useful once.

## Read-Do Steps

1. Restate the artifact, its consumer, and the proof that it is complete.
2. Build the smallest artifact that satisfies the request.
3. Keep verification explicit and keep promotion optional.
4. Promote only the stable reusable truth learned from the artifact, not the artifact by default.

## Exit Criteria

- The requested artifact exists and is verifiable.
- Promotion, if any, is justified by reuse and stability rather than convenience.
