# CLI Data And State

## Owned State

1. Local connection lifecycle state for endpoint session status and retry intent.
2. Endpoint identity and capability registration payload values for this client instance.
3. Transient stdin-to-sense translation state and stdout rendering state.

## Consumed State

1. Socket path and process input parameters supplied at CLI startup.
2. Core protocol responses, dispatch outcomes, and correlated message payloads.
3. Operator stdin text stream used as outbound sense input.

## Local Invariants

1. CLI does not own or persist product/domain authority state.
2. User input is translated into protocol-conformant senses; incoming outcomes are rendered explicitly.
3. Disconnect/failure states are surfaced explicitly to operators rather than silently masked.

## Authority Boundaries

1. CLI owns terminal interaction and local process lifecycle behavior.
2. Core owns dispatch outcome authority, runtime observability authority, and domain runtime truth.
3. Any change to cross-unit contract or authority semantics must escalate to Product TDD.

## Failure-Sensitive Assumptions

1. Core may be unavailable or disconnect during operation; client behavior must fail fast and remain recoverable.
2. Protocol incompatibility may occur; invalid frames/payloads must produce explicit failure paths.
3. Endpoint identity mismatch can break routing/correlation; identity contract assumptions are treated as critical.
