# Spine Contracts

Boundary:
1. input: `Act`
2. output: single `ActDispatchResult` per dispatch (`Acknowledged | Rejected | Lost`)

Must hold:
1. dispatch contract is act-based (admission-free).
2. routing is a mechanical endpoint lookup by `act.endpoint_id`.
3. capability routing is delegated to the endpoint implementation.
4. missing endpoint maps to deterministic rejection; transport/runtime loss maps to deterministic `Lost`.
5. Stem maps outcome to ordered settlement events and guarantees linkage fields:
   - `reserve_entry_id`
   - `cost_attribution_id`
6. Spine and adapters publish proprioception updates through `new_proprioceptions` / `drop_proprioceptions` control senses.

Neural Signal Design Guidelines:
1. Prefer excellent abstraction with orthogonal parameters and reasonable defaults.
2. Keep each parameter axis independent whenever possible; avoid coupled flags that force hidden behavior bundles.
3. Defaults must be safe and predictable, while still allowing explicit low-level override when needed.
4. AI-facing neural-signal APIs must be highly composable, so complex behaviors are formed by combining small primitives.
5. Neural signals must remain low-level operable: do not hide essential control points behind high-level one-shot wrappers.
