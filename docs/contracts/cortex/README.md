# Cortex Contracts

Boundary:
1. input: `Sense[]`, `PhysicalState`, `CognitionState`
2. output: `CortexOutput { acts, new_cognition_state }`

Must hold:
1. Cortex is stateless and has no direct side effects on external components.
2. Same input + same model outputs + same clamp config => deterministic `Act[]`.
3. `Act` is non-binding; execution decisions remain in Stem pipeline.
4. Deterministic clamp is final authority before act emission.
5. `sleep` sense is never processed by Cortex.
6. Capability catalog consumed by Cortex is the merged physical capability view from Stem.
