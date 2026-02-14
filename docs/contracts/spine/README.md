# Spine Contracts

Boundary:
1. input: `ActDispatchRequest`
2. output: single `SpineEvent` per dispatch

Must hold:
1. dispatch contract is act-based (admission-free).
2. events are ordered/replayable by Stem-provided `seq_no`.
3. settlement linkage fields are always present:
   - `reserve_entry_id`
   - `cost_attribution_id`
4. route-miss and endpoint-failure map to deterministic rejection events.
5. routing behavior is mechanical table lookup over route key (`endpoint_id`, `capability_id`).
