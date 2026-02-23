# Cortex Contracts

Boundary:
1. input: `Sense[]`, `PhysicalState`, `CognitionState`
2. output: `CortexOutput { acts, new_cognition_state, wait_for_sense }`

Must hold:
1. Cortex is stateless and has no direct side effects on external components.
2. Same input + same model outputs + same helper config => deterministic `Act[]`.
3. `Act` is non-binding; execution decisions remain in Stem pipeline.
4. Cortex is the cognition engine; Primary is the cognition core inside Cortex, and helpers are cognition organs (not LLM-wrapper abstractions).
5. Input IR root is `<input-ir>` and Output IR root is `<output-ir>`.
6. Primary failure/timeout returns noop with unchanged cognition state.
7. Helper failure degrades to fallback section/empty helper output.
8. `sleep` sense is never processed by Cortex.
9. Capability catalog consumed by Cortex is the merged physical capability view from Stem.
10. Primary IR uses fully-qualified neural signal ids only (no endpoint/sense/act split id fields, no instance ids).
11. `acts_helper` conversion owns act structuring end-to-end (`cognition-friendly <acts> -> Act[]`), including deterministic `fq_act_id` validation and `act_instance_id` generation in Rust.
