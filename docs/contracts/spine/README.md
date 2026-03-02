# Spine Contracts

Boundary:
1. Input: `Act` (dispatch path) and adapter ingress messages (`auth`, `sense`, `act_ack`, proprioception updates).
2. Output: `ActDispatchResult` (`Acknowledged | Rejected | Lost`) and routed senses to afferent pathway.

Must hold:
1. Dispatch contract is act-based and admission-free.
2. Routing is deterministic endpoint lookup by `act.endpoint_id`.
3. Missing endpoint maps to deterministic `Rejected`; transport/runtime failures map to deterministic `Lost`.
4. Dispatch failures emit correlated `dispatch.failed` sense to afferent pathway.
5. Body endpoint descriptor/proprioception updates are applied via direct calls to `StemControlPort`.
6. External NDJSON wire contract:
- `auth.body = { endpoint_name, ns_descriptors, proprioceptions? }`
- `sense.body = { sense_instance_id, neural_signal_descriptor_id, payload, weight, act_instance_id? }`
- `act_ack.body = { act_instance_id }`.
7. Adapter-authenticated endpoint id is canonicalized by Spine to generated `body_endpoint_id`.

Neural signal design guidelines:
1. Keep parameters orthogonal and composable.
2. Preserve low-level explicit control points.
3. Default behavior must be safe and predictable.
