# Cortex Contracts

Boundary:
1. input: `Sense[]`, `PhysicalState`, `CognitionState`
2. output: `CortexOutput { acts, new_cognition_state }`

Must hold:
1. Cortex is stateless and has no direct side effects on external components.
2. Same input + same model outputs + same helper config => deterministic `Act[]`.
3. `Act` is non-binding; execution decisions remain in Stem pipeline.
4. Input IR root is `<input-ir>` and Output IR root is `<output-ir>`.
5. Primary failure/timeout returns noop with unchanged cognition state.
6. Helper failure degrades to fallback section/empty helper output.
7. `sleep` sense is never processed by Cortex.
8. Capability catalog consumed by Cortex is the merged physical capability view from Stem.
