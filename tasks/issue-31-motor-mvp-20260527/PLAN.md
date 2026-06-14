# Issue #31 Motor MVP - Plan

> Last Updated: 2026-06-14
> Status: core topology correction implemented; next prerequisite pending
> Scope: entrypoint only; detailed exploration lives in sibling files
> Related issue: `#31`

## MVT Core

- Objective & Hypothesis: Define and validate a Motor MVP that improves Beluna's success rate on repeated artifact-editing work by adding a DSL-backed Motor component that participates in Afferent/Efferent pathway buses, calling active routines from Afferent Senses and emitting returned Acts to the Efferent bus.
- Guardrails Touched:
  - Restore act dispatch as a pipeline model, but make each stage middleware-like rather than a hard-coded single-purpose step.
  - Do not collapse Motor into Cortex. Cortex manages routine lifecycle by producing Neural Signal Acts addressed to Motor's endpoint namespace; Motor remains a core component outside Cortex.
  - Treat active routines as Motor-internal callbacks invoked from Afferent Senses, not as Cortex thought or one-shot Act expansion.
  - Model routine state explicitly: pure routine source plus Motor-owned activation state.
- Verification:
  - A Motor-aware Agent Task Test demonstrates measurable improvement over a Cortex-only baseline on iterative Markdown / HTML slide artifact work.
  - Observability can attribute the improvement to Motor participation rather than only to better Cortex prompting.
  - Acceptance evidence reports success rate, accuracy / diff quality, Motor interceptions, routine continuations, emitted Acts, afferent feedback, and failure modes across repeated runs.

## Packet Files

- [README.md](./README.md): task framing and current assumptions.
- [core-topology-correction/](./core-topology-correction/): Core topology correction sub-task.
- [MOTOR-AGENT-RATIONALE.md](./MOTOR-AGENT-RATIONALE.md): Motor's rationale from Beluna-as-agent and LLM Cortex constraints.
- [PREREQUISITE-SUBTASKS.md](./PREREQUISITE-SUBTASKS.md): Issue 31 prerequisite subtasks before Motor implementation.
- [CONTINUITY-STORE-ABSTRACTION.md](./CONTINUITY-STORE-ABSTRACTION.md): Continuity generic store direction with routine source as forcing case.
- [MOTOR-INTERNAL.md](./MOTOR-INTERNAL.md): current routine / DSL / registry assumptions.
- [ROUTINE-REFLEX-MODEL.md](./ROUTINE-REFLEX-MODEL.md): corrected routine lifecycle and Sense-to-Act reflex model.
- [ROUTINE-STATE.md](./ROUTINE-STATE.md): routine stateless/stateful decision and explicit activation state model.
- [ROUTINE-DSL.md](./ROUTINE-DSL.md): routine DSL semantic requirements and candidate shapes.
- [DSL-COMPARISON-SPIKE.md](./DSL-COMPARISON-SPIKE.md): Nanolang vs Rhai examples and Nanolang embedding spike evidence.
- [DSL-BOUNDARIES.md](./DSL-BOUNDARIES.md): Rhai semantic limits and Nanolang C wrapper integration impact.
- [NANOLANG-SUSPEND-SPIKE.md](./NANOLANG-SUSPEND-SPIKE.md): second Nanolang spike for `await_sense`-like suspension and event-step fallback.
- [VERIFICATION.md](./VERIFICATION.md): Agent Task Test and observability direction.
- [DISCUSSION.md](./DISCUSSION.md): discussion log and open questions.
- [NEXT-DISCUSSION.md](./NEXT-DISCUSSION.md): recommended next discussion order.

## Current Posture

The Core topology correction sub-task is implemented in code and verified with
`cargo check` plus `cargo test --lib --bins`.

Durable docs are still deferred until implementation verifies the exact shape.
