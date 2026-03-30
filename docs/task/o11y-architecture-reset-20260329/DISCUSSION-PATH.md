# Observability Discussion Path

This file proposes how to conduct the next discussion rounds so we can prepare authoritative updates to Product TDD, Core Unit TDD, and Moira Unit TDD without collapsing into premature implementation detail.

It assumes the current issue note in [README.md](/Users/lanzhijiang/Development/Beluna/docs/task/o11y-architecture-reset-20260329/README.md) is the problem statement and this file is the meeting order.

## Goal

Produce enough stable design to update:

1. [observability-contract.md](/Users/lanzhijiang/Development/Beluna/docs/20-product-tdd/observability-contract.md)
2. [observability.md](/Users/lanzhijiang/Development/Beluna/docs/30-unit-tdd/core/observability.md)
3. the relevant Moira Unit TDD files under [docs/30-unit-tdd/moira](/Users/lanzhijiang/Development/Beluna/docs/30-unit-tdd/moira)

without yet forcing the full AI-gateway observability redesign.

## Discussion Principle

Do not discuss "the whole event catalog" at once.

Instead, discuss in descending order of stability:

1. telemetry model principles
2. cross-unit reconstruction guarantees
3. Core family ownership and lattice
4. Moira projection rules
5. deferred AI-gateway specialization

That order matters because later layers should inherit from earlier ones.

## Pass 1: Product TDD First

This pass should settle only cross-unit and long-lived rules.

Target outcome:

1. a corrected product-level observability model that does not depend on Stage 2 naming accidents
2. a clear boundary between telemetry context and structured event body
3. stable consumer guarantees that Moira may rely on

Questions to settle in this pass:

1. Which identifiers are globally meaningful telemetry anchors?
2. What does it mean for one event to belong to one `tick`?
3. Which work must be representable as interval work in Loom?
4. Which surfaces are guaranteed from raw logs versus projected by Moira?
5. Which concepts are family names, and which are merely lane or UI grouping concepts?

Questions to avoid in this pass:

1. exact field spelling in Core structs
2. exact AI gateway event names
3. backend adapter peculiarities
4. exact SQL projections or frontend component layout

Expected Product TDD outputs:

1. native carrier statement:
   - OpenTelemetry semantic context plus structured log body
2. anchor statement:
   - `wake` and `tick` are the primary observability anchors
3. interval statement:
   - organ execution and other admitted interval work must be pairable into Loom task bars
4. family-vs-lane statement:
   - family ownership and UI lane grouping are separate concerns
5. removal statement:
   - `cortex.tick` is not part of the target contract

### Pass 1, Simplified

Pass 1 does not need full event catalog design.
It only needs the minimum answers that later layers can build on.

Those minimum answers are:

1. Which identifiers are globally meaningful telemetry anchors?
2. What does it mean, at contract level, for an event to belong to one `tick`?
3. Which kinds of runtime activity must be reconstructable as interval work?
4. Which operator information slices must be reconstructable from raw Core logs?
5. Where do we need to distinguish family ownership from Loom lane grouping, and where may they simply coincide?

### Pass 1, Clarified Meanings

To avoid discussing the wrong thing:

1. "`event belongs to one tick`" should not mean a universal causal guarantee.
   It should mean the event is attributed to one analysis frame.
   If stronger causality is needed, Core must expose it explicitly through structured body fields or telemetry parentage.
2. "`interval work`" does not mean a specific widget.
   It means Core logs must let Moira reconstruct a bounded duration activity when that activity matters to operator reasoning.
3. "`surface`" means a reconstructable operator information slice, not necessarily one UI panel.
   Examples: wake scoping, tick chronology, organ execution, goal-forest comparison, topology, raw drilldown.
4. "`guarantee`" means Moira may rely on raw Core logs containing enough structured data to reconstruct that slice without parsing free-form prose.
5. "`family` versus `lane`" only matters when ownership and operator grouping diverge.
   If they happen to align for some concepts, that is fine.

### Current Identifier Inventory

This is the current Stage 2 identifier inventory gathered from Core observability docs and contract structs.
It is an inventory, not a target design.

Global anchors:

1. `run_id`
2. `tick`

Telemetry-causality anchors:

1. `span_id`
2. `parent_span_id`

Runtime entity or operation identifiers:

1. `organ_id`
2. `request_id`
3. `thread_id`
4. `turn_id`
5. `sense_id`
6. `act_id`
7. `descriptor_id`
8. `endpoint_id`
9. `adapter_id`
10. `channel_id`
11. `rule_id`

Other identifiers or version/reference-like fields currently present:

1. `backend_id`
2. `resource_id`
3. `reference_id`
4. `catalog_version`

Pragmatic reading for Pass 1:

1. `run_id` and `tick` are already the strongest accepted global anchors.
2. `span_id` and `parent_span_id` remain the generic within-tick causality anchors.
3. most other ids should remain local until we have a concrete reason to promote them into broader telemetry context.

### Current Minimum Interval-Work Set

Only one interval-work guarantee is currently strong enough to treat as settled at Pass 1:

1. `cortex.organ.start/end` must be pairable by Moira into one interval bar

Everything else may remain provisional until later passes unless we discover a strong cross-unit reason to lock it earlier.

## Pass 2: Core Unit TDD Lattice Correction

Only after Pass 1 is stable should we define the corrected Core family lattice.

This pass should answer:

1. Which runtime owner emits which family?
2. Which families are entities, and which are operation lifecycles?
3. Which fields are required in event bodies?
4. Which identifiers should be lifted into telemetry context only when they cross module boundaries?

Suggested order inside this pass:

1. tick and top-level anchors
2. Stem family split
3. Cortex family reduction after removing `cortex.tick`
4. Spine family cleanup
5. AI-gateway placeholder boundary, without locking the final capability lattice

Likely near-term target lattice to discuss:

1. `stem.tick`
2. `stem.afferent`
3. `stem.efferent`
4. `stem.ns-catalog`
5. `stem.proprioception`
6. `stem.afferent.rule` if that lifecycle still deserves a separate family
7. `cortex.organ`
8. `cortex.goal-forest`
9. `spine.adapter`
10. `spine.endpoint`
11. provisional `spine.dispatch` or a renamed replacement
12. provisional `ai-gateway.*` families pending deeper exploration

What should remain explicitly provisional here:

1. the final AI gateway split between transport and capability events
2. whether dispatch is represented purely as lifecycle or as lifecycle plus explicit operation spans

## Pass 3: Moira Unit TDD Projection Rules

Once Product TDD and the draft Core lattice are stable, define what Moira reconstructs from them.

This pass should answer:

1. which events Moira pairs into one chronology interval
2. which events remain point events
3. how raw-first query results map into humane chronology
4. which lane keys are used when multiple identities exist

Immediate stable rule to document:

1. `cortex.organ.start/end` are paired by Moira into one Gantt bar

Other likely projection rules to settle:

1. `stem.tick` becomes the canonical tick anchor row
2. afferent and efferent events project into different lane families
3. dispatch and endpoint activity should not force Moira to invent semantics absent from Core logs

Questions to avoid in this pass:

1. CSS
2. final visual style
3. query optimization details unless they change reconstruction truth

## Pass 4: AI-Gateway-Specific Exploration

This pass should happen only after the general observability model is stable enough that AI-gateway work is exploring one subsystem, not redefining the whole architecture.

Reason to defer:

1. the gateway currently mixes generic transport and chat-capability semantics
2. the right split depends on how chat runtime, future non-chat capabilities, and provider adapters actually own their state transitions

Questions for that later pass:

1. Which events are truly capability-neutral gateway transport events?
2. Which events are chat-thread and chat-turn events?
3. Where do tool call, tool result, thinking, token usage, and provider payloads belong?
4. Should thread and turn be modeled under a chat capability namespace instead of under `ai-gateway`?

## Proposed Meeting Cadence

Keep each round narrow.

Round 1:

1. finalize Product TDD principles and non-negotiable guarantees

Round 2:

1. finalize non-AI Core family lattice

Round 3:

1. finalize Moira pairing and lane-projection rules

Round 4:

1. enter AI-gateway observability redesign with the rest of the model already stable

## What I Should Draft Next

The most useful next authored artifact is not code.
It is a proposed authoritative diff for Product TDD first.

Specifically, I should next draft:

1. the updated structure of [observability-contract.md](/Users/lanzhijiang/Development/Beluna/docs/20-product-tdd/observability-contract.md)
2. the exact clauses to change, remove, or add
3. a provisional non-AI family lattice table for Core Unit TDD, marked as draft where needed

That would let us review the architecture at the right layer before touching implementation.
