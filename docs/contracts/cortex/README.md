# Cortex Contracts

Related:
- Goal forest detailed contract and manual testing checklist: `./goal-forest.md`

Boundary:
1. input: `Sense[]`, `PhysicalState`, `CognitionState`
2. output: `CortexOutput { acts, new_cognition_state, wait_for_sense }`

Must hold:
1. Cortex is stateless and has no direct side effects on external components.
2. Same input + same model outputs + same helper config => deterministic `Act[]`.
3. `Act` is non-binding; execution decisions remain in Stem pipeline.
4. Cortex is the cognition engine; Primary is the cognition core inside Cortex, and helpers are cognition organs (not LLM-wrapper abstractions).
5. Input IR root is `<input-ir>` and Output IR root is `<output-ir>`.
6. Primary executes as a bounded Cognitive Micro-loop and uses AI Gateway tool calls for Internal Cognitive Actions.
7. Internal Cognitive Action tools are `expand-sense-raw`, `expand-sense-with-sub-agent`, `patch-goal-forest`; they are not Somatic Acts.
8. Primary failure/timeout or `max_internal_steps` exhaustion returns noop with unchanged cognition state.
9. Helper failure degrades to fallback section/empty helper output.
10. Sense entries expose tick-local monotonic integer `sense-instance-id`; internal sense expansion tools consume these IDs.
11. Sense helper contract: small payload passthrough, large payload Postman Envelope (`brief`, `original_size_in_bytes`, `confidence_score`, `omitted_features`).
12. `hibernate` sense is never processed by Cortex.
13. Capability catalog consumed by Cortex is the merged physical capability view from Stem.
14. Proprioception consumed by Cortex is a merged map from Stem and rendered in Input IR `<proprioception>`.
15. Sense and proprioception semantics are distinct: `sense` is point-in-time external event; `proprioception` is continuous internal state.
16. Primary IR uses fully-qualified neural signal ids; senses additionally carry `sense-instance-id`.
17. `acts_helper` conversion owns act structuring end-to-end (`cognition-friendly <somatic-acts> -> Act[]`), including deterministic `fq_act_id` validation and `act_instance_id` generation in Rust.
18. Primary output sections are optional; missing sections must not fail contract parsing and must degrade deterministically (`somatic-acts` -> no acts, `new-focal-awareness` -> keep current l1-memory, `is-wait-for-sense` -> false).
19. Goal-forest mutations are not carried by Output IR sections; they are applied through `patch-goal-forest` tool calls during the Primary micro-loop.
