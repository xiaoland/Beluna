# Cortex LLD

## Deterministic IDs

- `cost_attribution_id = hash(reaction_id, affordance_key, capability_handle, based_on, planner_slot)`
- `attempt_id = hash(reaction_id, based_on, affordance_key, capability_handle, normalized_payload, requested_resources, cost_attribution_id)`

Both use canonical JSON and SHA-256 prefixing.

## Data Model

- `ReactionInput`: bounded, delta-oriented cycle input.
- `ReactionResult`: business output (`attempts`, `based_on`, `attention_tags`).
- `ProseIr`: primary-stage text IR.
- `AttemptDraft`: sub-stage structured draft with `intent_span` and `based_on`.
- `CapabilityCatalog`: runtime-provided affordance routing/schemas.
- `ReactionLimits`: hard call/token/time/payload/attempt bounds.

## Reactor Cycle Rules

1. exactly one primary call per cycle.
2. at most `max_sub_calls` total subcalls.
3. at most one repair call.
4. clamp runs as final authority before emission.
5. if first clamp rejects all, one repair pass may run.
6. if repaired clamp still rejects all, emit noop.

## Clamp Rules

1. reject empty `intent_span`.
2. reject empty `based_on`.
3. reject unknown `sense_id` references in `based_on`.
4. reject unknown affordances.
5. reject unsupported capability handles.
6. reject oversize payloads.
7. reject payload-schema violations.
8. enforce deterministic sort + `max_attempts` truncation.
