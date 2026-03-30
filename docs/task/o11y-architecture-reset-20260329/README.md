# Observability Architecture Reset

This task note tracks the current discomfort around Beluna Core observability after Moira Stage 2 landed.

It is intentionally issue-like and non-authoritative.
The purpose is to separate:

1. symptoms visible in Loom
2. underlying Core observability model problems
3. redesign decisions that should move into authoritative docs before the next refactor stage

## Context

Stage 2 proved that Moira can ingest and reconstruct a useful local observability surface, but it also surfaced architectural strain in the event model.

The current design successfully produces data, yet several event families and field shapes appear to be compensating for weak ownership boundaries:

1. a private `ContractEvent` algebra is treated as the true schema, while OTLP mostly acts as transport
2. generic infrastructure families are carrying capability-specific semantics
3. some family boundaries are owner-centric in a way that degrades operator readability
4. some event shapes appear optimized for current rendering shortcuts instead of honest runtime semantics

This note exists because fixing Moira alone is not enough. The actual task is a broader Beluna observability upgrade, and that may legitimately require destructive refactoring in Core.

## Why This Matters

If the observability model is structurally wrong, Moira will keep paying for that debt in every query, projection, and visualization layer.

In particular:

1. Lachesis query logic becomes a semantic repair layer instead of a reconstruction layer
2. Loom timelines and chronology views become harder to trust
3. Core runtime ownership becomes harder to reason about because the logs do not align with the real boundaries
4. future metrics, traces, and first-party endpoint supervision will inherit the same ambiguity

## Current Fault Lines

### 1. Private contract schema vs OTLP semantics

Current code path:

`runtime emit helper -> ContractEvent enum -> flatten_contract_event() -> tracing log -> OTLP exporter`

This means the canonical event schema currently lives in Core's private Rust algebra, not in a first-class OpenTelemetry semantic model.

Observed consequences:

1. many fields such as `family`, `thread_id`, `turn_id`, `request_id`, and similar correlation anchors are duplicated as payload fields instead of being treated primarily as telemetry context
2. `payload` becomes a large serialized envelope containing the real event body
3. `*_when_present` fields signal that the model is shaped around optional Rust struct serialization rather than around a clean telemetry taxonomy

This is not the same as "we are not using OTLP".
We are using OTLP logs as transport and export format.
The concern is that we are not yet using OpenTelemetry semantics as the native modeling surface.

## 2. `ai-gateway.request` currently mixes transport concerns with chat-capability concerns

The current `ai-gateway.request` family carries both:

1. generic backend-governed request lifecycle information
2. chat-specific or capability-specific detail such as `thread_id`, `turn_id`, `effective_tools`, and provider request/response payloads

That suggests the current boundary is too low-level for the semantics it is carrying.

Likely problem:

1. the general AI gateway layer should own transport, retry, backend routing, and backend error semantics
2. the chat capability layer should own thread, turn, tool-call, message, and committed conversation semantics

If true, then part of the current model is coupling Moira to chat capability details through a generic infrastructure family.

This does not automatically mean the `ai-gateway.*` namespace is wrong.
It may mean the current family split is wrong, for example:

1. gateway transport/retry events remain in a generic gateway family
2. chat-specific events move under a capability-owned family lattice

That decision still needs authoritative design work.

## 3. Tick ownership may be over-modeled

Current Stage 2 emits both `stem.tick` and `cortex.tick`.

Discomfort:

1. `stem.tick` already captures the rhythm/grant anchor
2. `cortex.tick` may be duplicating a boundary that should instead be reconstructable from `stem.tick`, `cortex.organ`, goal-forest events, and drained afferent activity

Possible direction:

1. keep `stem.tick` as the canonical life-rhythm event
2. either remove `cortex.tick`, or narrow it drastically to only Cortex-owned semantics that cannot be reconstructed elsewhere

This one remains open.
There is a real chance that some current `cortex.tick` fields are useful, but they may belong in more specific events.

## 4. `stem.signal` is overloaded

Current Stage 2 uses one `stem.signal` family for both afferent and efferent transitions, distinguished by fields such as `direction` and `transition_kind`.

Discomfort:

1. afferent and efferent are different operators' mental models
2. their state machines are not symmetric
3. the current event family suggests a single abstraction where the runtime behavior is not actually uniform

Current mismatch examples:

1. efferent does not naturally have the same `defer` or `release` lifecycle as afferent
2. afferent does not naturally have the same terminal result semantics as efferent dispatch

Likely direction:

1. split into `stem.afferent` and `stem.efferent`
2. let each family own only the transitions its runtime actually has
3. stop forcing one shared transition vocabulary across both directions

## 5. Verb-shaped family names currently leak into the model

Examples:

1. `stem.dispatch`
2. `spine.dispatch`

Discomfort:

1. these names are verbs or pipeline actions, while the operator-friendly lane model wants entity-centric nouns
2. `stem.dispatch` also overlaps conceptually with transition cases already represented inside `stem.signal`

This suggests one of two things is wrong:

1. the family naming is wrong
2. the family boundaries are wrong

Likely direction:

1. prefer entity- or domain-object-centered families when they represent durable operators' concepts
2. keep verbs only when the runtime truly owns a short-lived operation that is not better modeled as an entity lifecycle

Related naming discomfort:

1. `stem.descriptor.catalog` reads like a container path, not a runtime object
2. `stem.ns-descriptor.catalog` is likely closer to the actual domain noun

## 6. Current chronology projection is treating start and end logs as separate bars

`cortex.organ.start` and `cortex.organ.end` should represent one span-like interval in Loom.

Important distinction:

1. this may be partly a Core event-model issue
2. this may also be partly a Moira projection issue

The current Stage 2 data already carries shared identifiers such as `request_id` and `span_id`.
That means Moira should be able to pair the start and end records into one operator-visible task bar even before a deeper redesign.

So this specific issue is not sufficient evidence that Core must change.
It is evidence that Moira's current chronology projection is too literal.

## Working Hypothesis

The core architectural problem is not "missing data".
It is that the event model currently mixes three different concerns:

1. runtime ownership
2. operator-facing semantic objects
3. telemetry transport/correlation encoding

Those three concerns need to be separated more cleanly.

## Tentative Redesign Axes

These are not decisions yet.
They are the axes the next authoritative design pass should resolve.

### A. Separate telemetry context from event body

We likely need a cleaner model with three layers:

1. OpenTelemetry context
2. stable event family plus event body
3. Moira-owned projections for UI surfaces

Rough target:

1. trace and span context should not be redundantly encoded everywhere as ad hoc payload decoration
2. event bodies should describe what happened
3. Moira should derive view grouping from telemetry context plus stable domain identifiers, not from stringly repair logic

### B. Re-check event ownership against actual runtime owners

Guiding question:

"Which runtime module genuinely owns the transition being reported?"

Examples:

1. chat thread and turn semantics likely belong with the chat capability, not with generic gateway infrastructure
2. Stem afferent and efferent probably deserve distinct observability boundaries
3. tick rhythm likely belongs primarily to Stem, with Cortex contributing only genuinely Cortex-owned work

### C. Model interval work as spans, not as unrelated point events

Moira's humane chronology wants interval semantics.

So the model likely needs:

1. one canonical tick trace anchor
2. consistent span identity for interval work such as organ execution, gateway request execution, and dispatch execution
3. start/end or open/close records that pair naturally into one operator-visible duration bar

The current "two logs with the same id" approach may be good enough if the projection is fixed, but the next design pass should decide whether this remains log-first-with-pairing or becomes more explicitly span-shaped.

### D. Prefer honest domain families over coarse catch-all families

The current Stage 2 model biased toward fewer rich families.
That reduced code churn, but it may have collapsed domain boundaries too aggressively.

The next design pass should optimize for:

1. honest runtime semantics
2. clear operator meaning
3. low reconstruction ambiguity

That may justify more families than Stage 2 used.

## Concrete Questions For The Next Authoritative Pass

These should be answered in authoritative docs before the next large refactor.

1. What is the native observability carrier for Beluna: OpenTelemetry semantic context plus structured log body, or a private event algebra flattened into OTLP?
2. Which AI events belong to generic gateway infrastructure, and which belong to chat capability or future non-chat capabilities?
3. Does `cortex.tick` survive, narrow, or disappear?
4. Should Stem observability be split into `stem.afferent` and `stem.efferent`?
5. Should dispatch be represented as entity lifecycle, operation span, or both?
6. Which correlation anchors are true telemetry context, and which are domain payload?
7. Which interval surfaces must Moira pair into Gantt bars without requiring new Core events?

## Immediate Non-Goals

This note does not yet decide:

1. the final family catalog
2. the final OpenTelemetry encoding strategy
3. the exact Loom screen decomposition
4. whether all current Stage 2 fields are kept, renamed, or removed

## Proposed Next Step

Do not start by patching event names one by one.

Instead:

1. update authoritative Product TDD and Core Unit TDD around the three-layer model:
   - telemetry context
   - event-body ownership
   - Moira projection rules
2. define the corrected family lattice for:
   - tick
   - chat capability vs gateway transport
   - Stem afferent/efferent
   - Spine endpoint/adapter/dispatch
3. only then refactor Core emission helpers and Moira normalization together

## Promotion Candidates

If the next discussion stabilizes, promote into authoritative docs:

1. the separation between telemetry context and event-body semantics
2. the corrected event-family lattice
3. the rule for interval pairing into Loom chronology
4. the decision on whether `cortex.tick` remains canonical

## Decisions Already Made

The following decisions are now stable enough to guide the next discussion round:

1. Beluna's native telemetry carrier is OpenTelemetry semantic context plus structured log body.
2. `cortex.tick` should disappear.
3. Stem observability should split at least into:
   - `stem.afferent`
   - `stem.efferent`
   - `stem.ns-catalog`
   - `stem.proprioception`
4. `wake` / `run_id` and `tick` are the strongest currently accepted global telemetry anchors.
5. `cortex.<organ-id>.start/end` should already be paired by Moira into one Gantt bar.

The following remain intentionally open:

1. the final AI gateway family lattice
2. which AI events belong to generic gateway transport versus chat capability
3. whether dispatch should finally be modeled purely as entity lifecycle, operation span, or a hybrid
4. which non-global identifiers should become trace/span context versus remain payload fields

## Round-2 Corrections

The following corrections were raised after the first Product TDD and Core Unit TDD draft pass.
They should be treated as active design corrections, not as settled implementation details.

1. `cortex.organ` may be too coarse.
   A likely target is one family per stable organ such as:
   - `cortex.primary`
   - `cortex.sense-helper`
   - `cortex.goal-forest-helper`
   This improves operator readability, but it increases family-count churn and fixture churn.
2. `spine.endpoint` should continue to own endpoint attachment and lifecycle semantics such as new / register / drop.
   `spine.sense` should mean Spine has received one sense from a body endpoint, not "an endpoint became available".
3. `ai-gateway.request` should either disappear or become strictly capability-neutral.
   It should not carry chat-specific semantics if it remains a generic gateway family.
4. The current "projection-relevant identity exposure" draft likely mixed up telemetry context with Moira grouping policy.
   The stronger user direction is:
   - `tick` is the human trace anchor
   - `organ_id`, `endpoint_id`, `thread_id`, `turn_id`, and similar identities are closer to span identity than to ad hoc lane-key fallback
5. This still needs one careful distinction:
   domain identity and literal OpenTelemetry `span_id` may need to remain separate if one domain actor can produce multiple span instances over time or within one tick.
