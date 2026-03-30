# Observability Contract

This file defines the authoritative cross-unit contract between Core OpenTelemetry log emission and Moira local observability consumption.

Core-internal family naming and emit-point choices belong in Core Unit TDD.
Loom view composition and operator interaction design belong in Moira Unit TDD.
This file defines the reconstructable operator domains and correlations that both units may rely on.

## Scope

1. Beluna's native first-party telemetry carrier is OpenTelemetry semantic context plus structured log body.
2. `core` owns emission semantics; `moira` owns local ingestion, storage, query, projection, and control-plane behavior built on those semantics.
3. This file defines reconstructable operator domains, not a fixed Loom panel layout or widget decomposition.
4. Metrics and traces remain limited to exporter status and handoff destinations in the current contract.
5. Early Beluna observability optimizes for lossless local inspection rather than aggressive summarization.

## Cross-Unit Model

1. Moira must be able to reconstruct one local wake from raw Core OTLP log events plus Moira-owned supervision state.
2. `run_id` and `tick` are the strongest globally meaningful observability anchors. They scope one wake and one analysis frame within that wake, and `tick` is the operator-facing trace anchor inside one wake.
3. Every Core event consumed by Moira is attributed to exactly one `tick`.
4. Events emitted before the first live tick grant, or otherwise outside admitted cognition work, remain attributed to the bootstrap anchor `tick = 0`.
5. Tick attribution is not, by itself, a universal causal guarantee among all events that share one `tick`. Stronger causality requires explicit telemetry parentage or structured event-body semantics.
6. Telemetry context and structured event body are separate concerns. Telemetry context carries wake attribution, tick attribution, and within-tick parentage. The structured body describes what happened.
7. `span_id` and `parent_span_id` remain the generic within-tick causality anchors when one event is nested under another. Literal span ids are operation-instance identifiers rather than operator-facing domain labels. Additional identifiers may remain local structured-body fields unless cross-module correlation justifies lifting them into broader telemetry context.
8. Event-family naming and Loom lane grouping are separate concerns. When they diverge, Core owns family semantics and Moira owns operator-facing grouping.
9. Structured fields carry the canonical semantics for conversation, organ execution, pathway state, topology, routing, and goal-forest reconstruction. Free-form text may supplement debugging, but it must not be the only source of truth for those domains.
10. Raw OTLP log records must preserve full request, response, signal, topology, and other early-stage diagnostic payloads by default. Summaries are convenience fields, not replacements.
11. Goal-forest comparison is derived from selected tick snapshots rather than emitted as canonical diff state.
12. Moira must be able to descend from any reconstructed operator context to the underlying raw OTLP records that justify it. This source-grounded inspection is the contract-level meaning of drilldown.
13. The target contract has one canonical tick-anchor surface. Duplicative Cortex-owned tick-summary surfaces are not part of the target contract.

## Required Structured Reconstruction Domains

1. Wake domain
- Core logs must expose stable wake correlation fields and timestamps sufficient to scope emitted events to one runtime execution.

2. Tick chronology domain
- Core logs must expose enough structure to inspect one `tick` as an analysis frame and chronology anchor.
- The minimum global correlation set is `run_id`, `tick`, and ordering timestamp.
- Within-tick nested or interval work additionally requires stable operation identity and parentage when relevant.
- Core logs must expose enough structure to reconstruct bounded-duration activity when that activity matters to operator reasoning.
- At minimum, Cortex organ execution must be reconstructable as one interval from paired boundary records or equivalent semantics.
- The contract does not require a dedicated Cortex-owned tick event as long as one canonical tick-anchor surface exists and the remaining required domains are reconstructable.

3. Cortex domain
- Core logs must expose structured organ-boundary records with full inputs and outputs, organ identity, and correlation to related AI-capability records.
- Related AI-capability request and response lifecycle records, committed conversation state, and authoritative conversation snapshots must remain reconstructable in the context of tick chronology and organ investigation when chat capability is in use.
- These records must carry provider, model, tool activity, token consumption, retry or failure detail when present, thinking payload when present, and full request and response payloads.
- The contract intentionally does not yet freeze the final split between generic gateway transport events and capability-specific chat events.
- Core logs must expose goal-forest snapshot records and runtime-grounded mutation records sufficient to explain what changed within a tick and to compare two ticks later.
- The current contract does not require pre-classified botanical diff verbs. It requires the structured mutation records that Core naturally owns at the current runtime boundary.
- When Cortex invokes an AI capability, the originating organ identity must remain available on the related records.

4. Stem domain
- Core logs must expose structured records for the canonical tick anchor or tick grant, afferent pathway activity, efferent pathway activity, proprioception changes, neural-signal catalog changes, and afferent rule lifecycle when that lifecycle is explicit in Core.
- These records must carry sense, act, descriptor, and endpoint identities when relevant, queue or deferral state when relevant, terminal outcome when relevant, and payloads needed for source-grounded inspection.
- The target contract requires afferent and efferent to remain distinguishable as separate operator domains. One coarse combined signal family is not a product-level requirement.

5. Spine domain
- Core logs must expose structured records for adapter lifecycle, endpoint lifecycle, inbound sense ingress from body endpoints, outbound act routing or binding, and terminal delivery outcomes.
- These records must carry the information required to understand which body topology was active, how senses entered Core, and how acts were delivered, rejected, lost, or otherwise completed.

## Consumer Guarantees

1. Moira may rely on the required domains above to implement:
- wake-scoped runtime inspection
- tick chronology and interval views
- Cortex / Stem / Spine investigation from one selected tick, including nested AI-capability activity where present
- goal-forest comparison between selected ticks
- source-grounded inspection down to the supporting raw OTLP records without leaving Loom

2. Product TDD defines reconstructable domains and required correlations, not exact Core family names, SQL projections, or Loom screen decomposition.

## Non-Guarantees In Current Contract

1. First-party local metrics dashboards or trace explorers beyond exporter-status and handoff surfaces.
2. Universal causality among all events that share one `tick`.
3. Canonical precomputed goal-forest diff storage.
4. The final split between generic AI-gateway transport events and capability-specific AI events.
5. Cross-wake analytics or fleet-wide aggregation.
6. One fixed UI decomposition for Loom.

## Compatibility Rule

1. Removing one of the required structured reconstruction domains, or dropping structured semantics that Moira depends on for reconstruction, is a breaking cross-unit change.
2. Replacing full raw request, response, signal, or topology payload preservation with summary-only emission is a breaking cross-unit change while the current observability contract remains in force.
3. Collapsing required interval-boundary data so Moira can no longer reconstruct required interval work is a breaking cross-unit change.
4. Core may evolve its internal family catalog and Moira may evolve Loom composition as long as the cross-unit reconstruction guarantees remain intact.
5. Breaking changes require synchronized updates to Product TDD, the affected Core and Moira Unit TDD docs, and corresponding verification guardrails.
