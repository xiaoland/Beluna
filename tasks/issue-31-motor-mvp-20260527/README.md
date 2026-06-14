# Issue #31 Motor MVP

> Non-authoritative task packet.

## Framing

Issue `#31` asks for "小脑 / Motor": a core-internal component that reduces Cortex load by handling learned or scripted low-consciousness operations.

Before continuing Motor design, two grounding corrections are required:

- Current Core topology must be treated as the source of truth. See
  [core-topology-correction/CORE-TOPOLOGY-REALITY.md](./core-topology-correction/CORE-TOPOLOGY-REALITY.md).
- Motor must be justified from Beluna-as-agent and LLM Cortex constraints, not
  from a free-floating metaphor. See
  [MOTOR-AGENT-RATIONALE.md](./MOTOR-AGENT-RATIONALE.md).

Current prerequisite subtasks:

- Correct Core pathway topology into a source + middleware model. See
  [core-topology-correction/](./core-topology-correction/).
- More precisely, model Afferent/Efferent as buses with multiple tx/rx. See
  [core-topology-correction/PATHWAY-BUS-MODEL.md](./core-topology-correction/PATHWAY-BUS-MODEL.md).
- Evolve Continuity toward a generic store abstraction, with routine source
  definitions as the first forcing case. See
  [CONTINUITY-STORE-ABSTRACTION.md](./CONTINUITY-STORE-ABSTRACTION.md).

The current corrected read:

- Motor is outside Cortex, like `continuity`, `spine`, and `stem`.
- Motor is connected to both Efferent Pathway and Afferent Pathway.
- The whole Motor component is middleware on both Efferent and Afferent Pathways.
- On Efferent Pathway, Motor can handle Motor-owned lifecycle Acts and pass through unrelated Acts.
- On Afferent Pathway, Motor can call active routines with matched Senses.
- Act and Sense are both Neural Signals; Motor should be understood through endpoint identity and descriptors, not through an ordinary/internal Act split.
- Motor endpoint id is `motor`.
- Cortex manages Motor routine lifecycle by producing lifecycle Acts addressed to Motor's endpoint namespace.
- Motor needs built-in create, delete, activate, and terminate routine Acts.
- A routine's primary active shape is `state + Sense -> state + Vec<Act>`.
- Motor may consume lifecycle Acts and emit downstream Acts that continue after Motor through the dispatch pipeline.
- Motor owns routine lifecycle result Senses; routines may also produce Senses through the afferent pathway.
- Cortex-created routines are learned Motor knowledge and are persisted through Continuity.
- Motor communicates routine persistence requests to Continuity through Acts, not direct storage calls.
- Generic act accepted/rejected/failure payload authority belongs to the efferent pathway across Motor, Continuity, Spine, and Stem.
- The slide Markdown / HTML acceptance criterion is end-to-end; it is not the Motor routine boundary itself.
- Motor should prove measurable improvement in success rate, accuracy, or iteration stability for that end-to-end task.

## Current Assumptions

- Security boundary design is not first-priority for this issue.
- Routines are currently assumed to be DSL-authored functions owned by Motor's registry.
- Routine definitions are persisted by Continuity; explicit routine activation state is owned by Motor and remains non-persisted for MVP unless later evidence changes this.
- Cortex writes routines by sending Motor lifecycle Acts.
- Stem at least registers built-in Motor lifecycle descriptors; whether per-routine Neural Signals are still needed is open under the reflex model.
- The act dispatch path should be a fixed middleware sequence: `Motor -> Continuity -> Spine`.
- Active routine execution should be modeled as Motor middleware invoking internal callbacks from Afferent Senses, not as one-shot Act expansion.
- Hidden mutable DSL runtime state should be avoided for MVP; state should be explicit and Motor-owned.
- The task packet is deliberately split across files because the packet is also an exploration and thinking workspace, not just a plan.

## Governing Anchors

- root `AGENTS.md`
- `tasks/README.md`
- `core/src/cortex/AGENTS.md`
- `docs/20-product-tdd/unit-topology.md`
- `docs/20-product-tdd/system-state-and-authority.md`
- `docs/30-unit-tdd/core/design.md`
