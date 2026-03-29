# Observability Contract

This file defines the authoritative cross-unit contract between Core OTLP log emission and Moira local observability consumption.

Core-internal family naming and emit-point choices belong in Core Unit TDD.
Loom view composition and operator interaction design belong in Moira Unit TDD.

## Scope

1. Beluna's first-party local observability contract is log-first.
2. `core` owns OTLP log emission semantics; `moira` owns local ingestion, storage, query, and control-plane behavior built on those semantics.
3. Metrics and traces remain limited to exporter status and handoff destinations in the current contract.
4. Early Beluna observability optimizes for lossless local inspection rather than aggressive summarization.

## Cross-Unit Reconstruction Rules

1. Moira must be able to reconstruct one local wake from raw Core OTLP log events plus Moira-owned supervision state.
2. Every Core event consumed by Moira belongs to exactly one `tick`.
3. `tick` is the local observability anchor for Beluna's runtime life rhythm. This rule is specific to observability and control-plane reconstruction; it does not redefine the global glossary into “tick means any cycle-like number everywhere”.
4. Events emitted before the first live tick grant, or otherwise outside one admitted cognition cycle, still remain tick-scoped by using the bootstrap anchor `tick = 0`.
5. Events that participate in one within-tick causal chain must expose stable span and parentage keys, plus stable operator-recognizable lane identity when the operator-facing view needs lane-based rendering.
6. Event-family naming and lane identity are separate concerns. Family names are owner-centric observability boundaries; lane types must remain entity-centric so Loom renders humans recognizable actors such as organ, thread, sense, act, endpoint, or adapter rather than subsystem verbs such as `spine.dispatch`.
7. Structured fields carry the canonical semantics for cycle, conversation, signal, topology, and dispatch reconstruction. Free-form text and raw body fields may supplement debugging, but they must not be the only source of truth for those surfaces.
8. Raw OTLP events must preserve full request, response, signal, and topology payloads by default during Beluna's early development phase. Summaries are convenience fields, not replacements.
9. Goal-forest comparison is derived from selected tick snapshots rather than emitted as canonical diff state.
10. Raw OTLP log events must remain preservable for drilldown from every higher-level Loom surface.

## Required Structured Observability Surfaces

1. Wake-scoping surface
- Core logs must expose stable wake correlation fields and timestamps sufficient to scope emitted events to one runtime execution.

2. Tick-trace surface
- Core logs must expose enough structure to render and inspect one tick as a trace-like event sequence.
- This surface must distinguish skipped, gated, and admitted tick states, and it must preserve the full admitted senses plus physical snapshot when one cognition cycle actually runs.
- The minimum correlation set is `run_id`, `tick`, ordering timestamp, span identity, parent span identity when nested, and stable resource-lane identity when relevant.
- When operator-facing lane rendering is needed, Core logs must expose enough structured identity that Moira can derive an entity-centric `lane_type` and `lane_key` without relying on free-form message parsing.

3. Conversation and LLM surface
- Core logs must expose structured AI-gateway request lifecycle records, committed-turn records, and authoritative thread snapshots sufficient to reconstruct conversation state across one tick and across consecutive ticks.
- These records must carry provider, model, tool activity, token consumption, gateway retry or failure detail when present, thinking payload when present, and full request/response payloads.
- Core must also expose the canonical thread messages array, or an equivalent authoritative thread snapshot, after each persisted thread rewrite or completed turn so Moira can connect turns into one human-readable conversation history without replay heuristics.
- When Cortex invokes the AI gateway, the originating `organ_id` must remain available on the related AI-gateway records.

4. Cortex organ and goal-forest surface
- Core logs must expose structured organ-boundary records with full inputs and outputs, organ identity, and correlation to related AI-gateway spans.
- Core logs must also expose goal-forest snapshot records and runtime-grounded mutation records sufficient to explain what changed within a tick and to compare two ticks later.
- The current contract does not require pre-classified botanical diff verbs; it requires the structured mutation records that Core naturally owns at the current runtime boundary.

5. Stem state and pathway surface
- Core logs must expose structured records for Stem tick grants, signal transitions, dispatch transitions, proprioception changes, descriptor catalog changes, and afferent rule lifecycle.
- These records must carry descriptor identity, endpoint identity when relevant, sense or act identity when relevant, queue or deferral state when relevant, terminal outcome when relevant, and payloads needed for drilldown.

6. Spine topology and dispatch surface
- Core logs must expose structured records for adapter lifecycle, endpoint lifecycle, dispatch binding, and terminal dispatch outcome.
- These records must carry the information required to understand which body topology was active and how acts were routed and completed.

## Consumer Guarantees

1. Moira may rely on the required surfaces above to implement:
- wake-scoped runtime inspection
- tick timeline and per-tick Gantt-style trace views
- AI-gateway thread and turn browsing grounded in authoritative thread snapshots plus committed turns and request lifecycle
- Cortex / Stem / Spine drilldown from one selected tick
- goal-forest comparison between selected ticks
- raw-event drilldown without leaving Loom

2. Product TDD does not freeze exact Core family names or Loom screen decomposition as long as the required reconstruction surfaces and correlations remain intact.

## Non-Guarantees In Current Contract

1. First-party local metrics dashboards or trace explorers beyond exporter-status and handoff surfaces.
2. Canonical precomputed goal-forest diff storage.
3. Cross-wake analytics or fleet-wide aggregation.
4. One fixed UI decomposition for Loom.

## Compatibility Rule

1. Removing one of the required structured observability surfaces, or dropping structured semantics that Moira depends on for reconstruction, is a breaking cross-unit change.
2. Replacing full raw request/response/signal payload preservation with summary-only emission is a breaking cross-unit change while the current observability contract remains in force.
3. Core may evolve its internal family catalog and Moira may evolve Loom composition as long as the cross-unit reconstruction guarantees remain intact.
4. Breaking changes require synchronized updates to Product TDD, the affected Core and Moira Unit TDD docs, and corresponding verification guardrails.
