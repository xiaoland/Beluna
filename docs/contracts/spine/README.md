# Spine Contracts

Boundary:
1. input: `Act`
2. output: single `EndpointExecutionOutcome` per dispatch

Must hold:
1. dispatch contract is act-based (admission-free).
2. routing is a mechanical endpoint lookup by `act.endpoint_id`.
3. capability routing is delegated to the endpoint implementation.
4. missing endpoint and endpoint failure map to deterministic rejection outcomes.
5. Stem maps outcome to ordered settlement events and guarantees linkage fields:
   - `reserve_entry_id`
   - `cost_attribution_id`
