# Observability Contract

This file defines the authoritative cross-unit contract between Core OpenTelemetry emission and Moira local observability consumption.

Core-local event schemas belong in Core Unit TDD.
Loom view composition and operator interaction design belong in Moira Unit TDD.
This file defines reconstructable operator domains and correlations that both units may rely on.

## Scope

1. Beluna's native first-party telemetry carrier is OpenTelemetry semantic context plus structured log body.
2. Core owns emission semantics.
3. Moira owns local ingestion, storage, query, projection, and control-plane behavior built on those semantics.
4. Early Beluna observability optimizes for lossless local inspection and source-grounded drilldown.
5. Metrics and OTLP trace ingestion remain outside Moira's first-party local storage contract for this stage.

## Cross-Unit Model

1. Core first-party logs use native OTLP fields: resource, instrumentation scope, `eventName`, trace/span context, attributes, and body.
2. `resource` identifies the Core process.
3. `scope.name` identifies the emitting Core owner and is Moira's native owner-lane key. Most owner scopes are fixed; Spine endpoint and adapter owners use Core-canonical dynamic scope suffixes.
4. `eventName` identifies the event type under one owner. `scope.name + eventName` is the event schema key.
5. `body` carries the rich structured event payload.
6. Attributes carry small, stable metadata used for the event type's own lookup, grouping, or filtering.
7. One wake plus one tick maps to one trace. Pre-first-tick activity uses `tick = 0`.
8. Wake read models are anchored by `beluna.core.main.runtime / booted` body identity. Tick read models are anchored by `beluna.core.stem.tick / granted` body identity plus trace id.
9. Within-tick interval work is a Moira projection from shared span context across boundary records.
10. Parent/child topology can later come from OTLP traces, a span registry, or explicit bridge fields.
11. Raw OTLP records must preserve full request, response, signal, topology, chat, and diagnostic payloads by default.
12. Summaries are convenience fields inside body; source-grounded payload remains canonical.
13. Core event vocabulary uses `started` and `finished` for lifecycle boundaries. `completed` and `dispatched` are retired in the native first-party contract.
14. Dynamic owner scope suffixes are stable, OTLP-safe canonical segments. The event body preserves the original endpoint id or adapter name.

## Required Owner Domains

1. Main domain
- Core exposes runtime bootstrap and signal/exporter state.
- The bootstrap anchor event is `beluna.core.main.runtime / booted`.
- The bootstrap body carries `run_id`.

2. Tick chronology domain
- Core exposes one canonical tick grant anchor.
- The target anchor event is `beluna.core.stem.tick / granted`.
- The tick body carries `run_id`, `tick`, and `tick_seq`.
- Moira uses this anchor plus trace id to build admitted tick chronology.

3. Cortex domain
- Core exposes concrete organ interval records with full inputs and outputs.
- Cortex owner scopes include `beluna.core.cortex.primary`, `beluna.core.cortex.attention`, `beluna.core.cortex.cleanup`, `beluna.core.cortex.sense-helper`, `beluna.core.cortex.acts-helper`, and `beluna.core.cortex.goal-forest`.
- Stable organ boundary event names are local to each owner lane, such as `started` and `finished`.
- Primary phase records share the `primary` span for one tick's primary phase.
- Related AI records remain reconstructable as transport and chat owner records.
- Goal-forest inspection remains grounded in snapshots and runtime-owned mutation records.

4. AI Gateway transport domain
- Core exposes capability-neutral backend transport records under `beluna.core.ai-gateway.transport`.
- Transport event names include `request.started`, `attempt.failed`, `request.finished`, and `request.failed` when those phases exist.
- Transport records may carry backend id, model, capability, attempt/retry detail, provider request/response payloads, usage, and terminal error detail.
- Transport request identity is `transport_request_id` in body.

5. AI Gateway Chat domain
- Core exposes chat turn and thread lifecycle under `beluna.core.ai-gateway.chat`.
- Chat turn lifecycle event names include `turn.started`, `turn.finished`, and `turn.failed`.
- Chat thread event names include `thread.opened`, `thread.derived`, `thread.rewritten`, and `thread.snapshot`.
- Chat records own `thread_id`, `turn_id`, transport request identity, messages, turn start payloads, tool/message payloads, finish reason, usage, and backend metadata.

6. Stem domain
- Core exposes afferent pathway activity, efferent pathway activity, proprioception changes, neural-signal catalog changes, and afferent rule lifecycle when those surfaces are explicit in Core.
- Stem owner scopes include `beluna.core.stem.afferent-pathway`, `beluna.core.stem.efferent-pathway`, `beluna.core.stem.proprioception`, and `beluna.core.stem.descriptor-catalog`.
- Afferent rule lifecycle belongs to the afferent pathway owner through event names such as `rules.added`, `rules.removed`, and `rules.replaced`.
- These records carry sense, act, descriptor, and endpoint identities where the event schema needs them.

7. Spine domain
- Core exposes adapter lifecycle, endpoint lifecycle, inbound sense ingress, outbound act routing/binding, and terminal delivery outcomes.
- Spine endpoint owner scopes use `beluna.core.spine.endpoint.<endpoint-id-segment>`.
- Spine adapter owner scopes use `beluna.core.spine.adapter.<adapter-name-segment>`.
- Endpoint owner event names include lifecycle records, `sense.received`, `act.started`, `act.finished`, `act.rejected`, and `act.lost`.
- Adapter owner event names include `enabled`, `disabled`, and `faulted`.
- Spine event schemas decide whether act and descriptor ids live in attributes or body. Endpoint id and adapter name remain recoverable from body even when the owner scope carries their canonical segment.

## Consumer Guarantees

Moira may rely on this contract to implement:

1. wake-scoped runtime inspection.
2. tick list and selected-tick raw event browsing.
3. native event timeline from `scope.name`, `eventName`, `traceId`, `spanId`, severity, attributes, and body.
4. source-grounded inspection down to the supporting raw OTLP records without leaving Loom.
5. progressive interval, nested AI, Stem, Spine, and goal-forest projections when the needed owner records are available.

Product TDD defines reconstructable domains and required correlations. Core Unit TDD owns exact owner scopes, event names, span keys, and event schemas.

## Non-Guarantees In Current Contract

1. First-party local metrics dashboards.
2. Moira-owned OTLP trace storage.
3. Universal causality among all records that share one tick trace.
4. Canonical precomputed goal-forest diff storage.
5. Cross-wake analytics or fleet-wide aggregation.
6. One fixed Loom UI decomposition.

## Compatibility Rule

1. Dropping full raw request, response, signal, topology, or chat payload preservation is a breaking cross-unit change during early development.
2. Changing `beluna.core.main.runtime / booted` or `beluna.core.stem.tick / granted` anchor identity fields requires synchronized Core and Moira updates.
3. Collapsing interval-boundary data downgrades the corresponding Moira projection and requires updating the owning docs.
4. Core may evolve internal emit helpers and Moira may evolve Loom composition while the reconstruction guarantees remain intact.
5. Changing dynamic Spine owner scope canonicalization is a breaking observability contract change.
6. Breaking changes require synchronized updates to Product TDD, affected Unit TDD docs, and verification guardrails.
