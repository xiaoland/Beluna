# Discussion Log

## 2026-05-27

### Superseded Exploration Notes

- User clarified that `Continuity -> Spine` is not a correct design rule; the original pipeline shape was convenience-driven.
- User clarified that Continuity and Spine act handling should be understood as parallel from the act-processing perspective.
- User clarified that Motor becomes peer-level sender with Cortex from the act dispatch perspective.
- User clarified that Cortex should still control Motor, creating a non-trivial topology.

These notes motivated the first topology exploration, but the later middleware-pipeline correction supersedes them.

### Current Accepted Direction

- User clarified that Motor MVP does not encapsulate "receive conversation edit intent" as the routine level.
- User expects the slide acceptance criterion to depend more than 60% on Cortex, but Motor should measurably improve success rate and accuracy.
- User suggested Agent Test Case plus observability as a way to make iteration more accurate.
- User stated safety boundaries are not a priority for this issue.
- User clarified that Motor is outside Cortex, like `continuity`, `spine`, and `stem`.
- User requested avoiding monofile task packets because task packets are for exploration and thinking, not only plans.
- User corrected the topology again: restore act dispatch as a pipeline and make it middleware-like.
- User proposed `Motor -> Continuity -> Spine` as the dispatch pipeline.
- User clarified Cortex should still produce Acts; later correction: Act and Sense are Neural Signals, so avoid "ordinary Act" as a model term.
- User clarified Motor should intercept Acts it owns, expand them into a new set of Acts, and pass through unrelated Acts normally.
- User clarified Motor sense writeback should attach to the afferent pathway, like Continuity.
- User asked whether an `EfferentPathwayMiddlewareResponse` should exist. Current task answer: yes, but it can be very small: effectively `Vec<Act>`.
- User proposed that an empty response means downstream components have nothing to do, so the Act is terminated / intercepted. Current task answer: agree; response kind / operation variants are unnecessary if error and observability are kept outside the response.

## 2026-05-28

- User clarified that `routine` is acceptable vocabulary but must be grounded in Motor's actual needs rather than metaphysical description.
- User prefers DSL-authored routines.
- User clarified a routine is function-shaped.
- User clarified the routine registry is owned by Motor.
- User currently sees no required routine execution context.
- User expects routines to be stateless for MVP, with no persisted state.
- User clarified Act and Sense are both Neural Signals; there is no "ordinary Act" category.
- User clarified Motor and Continuity should be understood as inner body endpoint-like components via endpoint id.
- User said Motor itself currently has no required Senses, but routines may produce Senses.
- User clarified accepted/rejected payload authority belongs to the efferent pathway and applies across Motor, Continuity, Spine, Stem, and other efferent pathway components.
- User clarified Motor-expanded Acts continue after Motor because they are Motor middleware output.
- User said the discussion order should be updated.
- User clarified routine is not simply expanding an Act into more Acts; routine lets Motor continuously take over procedural actions.
- User clarified Motor is therefore connected to both Afferent Pathway and Efferent Pathway.
- User fixed Motor endpoint id as `motor`.
- User clarified Stem notifies Motor to register Neural Signals.
- User clarified routine maps 1:1 to Neural Signal at the Act layer.
- User rejected the question "how does Cortex discover Motor Acts through `PhysicalState.ns_descriptor`"; fq descriptor ids like `motor.xxx` are sufficient LLM information.
- User noted routine DSL is still undecided.
- User clarified routines are written by Cortex.
- User clarified Motor must have a routine-registration Act and routine-registration success/failure Senses.
- User clarified learned routines, meaning Cortex-created routines, are persisted through Continuity.
- User clarified communication for routine persistence is Act-based.
- User clarified the initial DSL decision should compare Nanolang, Mojo, other existing options, and a custom DSL.

## 2026-06-13

- User proposed that the previous Motor operation model may be wrong.
- User reframed Motor as a host whose composition elements are routines.
- User clarified routines are created and managed by Cortex.
- User proposed that Motor needs at least four Cortex-facing Acts: create, delete, activate, and terminate routine.
- User clarified that once a routine is active, it should attach to Sense and Act processing as a bypass path.
- User clarified the active routine's purpose is to replace Cortex's thinking process for mechanical reactions.
- User proposed the core active routine shape as `Senses -> routine -> Acts`.
- Superseded task response: the old `Act -> routine -> Vec<Act>` model might be retained for Motor control-plane middleware.
- Superseded task response: active routines might be modeled as afferent sidecars.
- Still-current task response: the earlier "routine 1:1 maps to Motor Act Neural Signal" assumption is weakened or superseded for primary execution.
- User further corrected that the whole Motor component should be modeled as middleware on both Efferent and Afferent Pathways.
- User rejected a separate "control plane" / `Act -> Vec<Act>` model.
- User clarified that active routines are called internally by Motor with Afferent Pathway Senses as arguments, and routine returns are Acts sent to the Efferent Pathway.
- User identified routine stateless/stateful semantics as the next core problem.
- Current task response: update model to `state + Sense -> routine -> state + Acts`; routine source should be pure, active routine state should be explicit and Motor-owned, and hidden mutable DSL runtime state should be avoided for MVP.
- User then identified two blockers before continuing Motor discussion:
  1. current understanding of Beluna Core topology is insufficient or incorrect.
  2. Motor was being understood too metaphysically; it must be grounded in Beluna being an agent and Cortex being LLM-based.
- Current task response: stop advancing Motor details and create grounding notes for Core topology reality and Motor's agent/LLM rationale.
- User accepted that core topology correction and Continuity routine source persistence are prerequisite subtasks for implementing Motor.
- User proposed the key Core topology correction: make Afferent and Efferent Pathways a `source + middleware` model.
- Current task response: record source + middleware as the pathway target model and split Issue 31 prerequisites into:
  1. Core pathway topology correction.
  2. Continuity routine source persistence.
- User asked whether Cortex and Spine should also be modeled as middleware for a cleaner topology.
- Superseded task response: Cortex and Spine could be modeled as terminal
  middleware roles if authority remains explicit.
- User clarified Afferent middleware is allowed to consume / transform Senses.
- User clarified Continuity should evolve into a generic store rather than coupling directly to routine storage.
- Current task response: routine source should be a forcing case for a generic Continuity store abstraction. Continuity should care about durable record properties, not routine DSL/runtime semantics.
- User rejected modeling `Terminal` as a fixed participant role because Continuity
  or Motor can terminally handle their own Efferent Acts, such as
  `continuity.l1_memory.update` or `motor.routine.activate`.
- User proposed focusing on the middleware contract instead:
  `accepted`, `rejected`, `continue(original/transformed)`.
- Current task response: accept this correction. Terminal behavior should be a
  middleware decision, not a fixed component role. Spine is only the middleware
  that usually accepts/rejects endpoint-directed Acts.
- User clarified that `continue` is same-pathway / same-signal-type only:
  `middleware(Sense) -> Continue(Sense)`, not
  `middleware(Sense) -> Continue(Act)`.
- User asked what "source API" means.
- Current task response: rename the idea to source port / emission port.
  Cross-pathway emissions such as Motor `Sense -> Acts` happen through an
  `EfferentSourcePort`, while the Afferent middleware decision still only
  accepts, rejects, or continues Senses.
- User clarified that Afferent/Efferent Pathways are closer to buses and should
  support multiple rx and multiple tx.
- Current task response: update the topology target to `PathwayBus<S>` with
  multiple source ports, multiple subscriptions, tap subscriptions, middleware
  subscriptions, routing policy, and lineage/observability.

## Open Questions

1. Is explicit Motor-owned activation state accepted as the routine state model?
2. Should activation state be raw JSON, typed DSL values, or a restricted Motor state object?
3. What exact built-in descriptor ids should Motor use for routine lifecycle Acts and lifecycle result Senses?
4. Should `terminate-routine` be named `deactivate-routine` if the routine definition remains persisted?
5. What minimum routine definition schema does Cortex submit on create?
6. What Sense selector syntax should active routines use?
7. What activation scope is safe for MVP: global, conversation, artifact, cycle, or explicit activation id?
8. Should active routines only observe matched Senses, or can they consume / transform Senses before Cortex sees them?
9. Should routine output be just `Vec<Act>` plus side-effect Sense emission, or an explicit `(Vec<Act>, Vec<Sense>)`?
10. What exact Continuity-owned Acts persist create, delete, activate, and terminate decisions?
11. Do routine-produced Acts re-enter the normal Efferent Pathway before Motor or use an after-Motor emitter?
12. Does Stem still need to register per-routine Neural Signals, or only built-in Motor lifecycle descriptors plus routine-produced Sense descriptors?
13. Under `state + Sense -> state + Vec<Act>`, is Rhai the preferred MVP DSL and is Nanolang still justified?
14. What is the smallest slide-MVP Motor routine set that can realistically improve success rate without swallowing Cortex's role?
