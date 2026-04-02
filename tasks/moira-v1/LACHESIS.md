# Lachesis Collection And Visualization

All stages in this file are Lachesis sub-stages inside the same Moira task. They are not separate roadmap phases.

## Design Goal

Lachesis should make Beluna explainable during early development. The collection and browsing model therefore optimize for:

- reconstructing what happened at one selected tick
- following AI-gateway conversation flow without replay heuristics
- following Stem and Spine movement as a lane-based chronology instead of one flat log list
- preserving enough raw evidence that Loom does not become a lossy summary layer
- keeping the Stage 2 event catalog owner-centric and coarse-grained enough that it matches the actual Core runtime boundaries

This note originally used an older Stage 2 draft vocabulary.
Current code truth now lives in the Core contract and Moira projection, so names such as `ai-gateway.turn`, `cortex.organ`, `stem.signal`, `stem.dispatch`, `stem.descriptor.catalog`, and `spine.dispatch` should be treated as historical task-buffer language only.
Older fixture ids may still preserve that vocabulary as scenario labels.

## Collection Principles

### 1. Collect raw first, project second

- Persist every accepted OTLP log event in a raw table.
- Build derived tables or indexes only for views that are repeatedly expensive or awkward to reconstruct.
- Do not store precomputed interpretations such as goal-forest diff when they can be derived from two snapshots.

### 2. Collect Beluna-native structure, not just text

- The primary contract must be structured event fields.
- Free-form text can remain as debugging payload, but must not be the only way to recover a tick, thread, signal, or dispatch story.

### 3. Prefer lossless payload retention during early development

- Do not fear large payloads in raw events while Beluna is still observability-heavy.
- Preserve full request, response, sense, act, thinking, and topology payloads in the raw store by default.
- Keep derived tables selective so the contract stays readable even when raw events stay large.

### 4. Tick is the trace anchor

- Every event belongs to one `tick`.
- Within a tick, chronology is reconstructed by timestamp plus span or lane identity.
- Loom should be able to render a per-tick Gantt-like chronology where the vertical axis is span or stable resource lane.
- Events that happen before the first live tick grant should remain inspectable under startup `tick = 0` rather than falling outside the model.

## What Lachesis Should Collect In This Task

## A. Wake And Process Lifecycle

Collect:

- Moira-supervised wake id
- Core version or source-build identity
- profile/config identity
- wake time, stop time, termination reason
- OTLP receiver health and ingest lag indicators

Why:

- every later view needs to scope itself to a wake
- failures during wake/stop must be visible as first-class events

## B. AI-Gateway Conversation Surface

Collect:

- `ai-gateway.request`
- `ai-gateway.chat.turn`
- `ai-gateway.chat.thread`

Each conversation record should preserve:

- `tick`
- request id, thread id, or turn id as appropriate
- provider
- model
- tool usage
- token consumption
- thinking payload when present
- full request or response payload
- originating `organ_id`
- committed turn messages array when a turn is actually persisted
- authoritative thread snapshot after each completed turn or thread rewrite

Why:

- human-friendly browsing needs committed conversation state, not isolated Cortex summaries pretending to be conversation history
- backend retry or failure detail lives at the AI-gateway request boundary, not at the committed-turn boundary
- primary-thread replacement and reset-context flows are naturally explained by authoritative thread snapshots

## C. Stem State And Pathways

Collect:

- `stem.tick`
- `stem.afferent`
- `stem.efferent`
- `stem.proprioception`
- `stem.ns-catalog`
- `stem.afferent.rule`

Each pathway record should preserve:

- direction or kind
- descriptor identity
- endpoint identity when present
- sense or act identity when present
- payload
- queue or deferral state when present
- rule ids or rule selectors when relevant
- terminal reason or reference when relevant

Why:

- this is the minimum needed so Stem is not visually empty
- the real runtime model is transition-based, not a speculative push/pop taxonomy
- descriptor catalog and proprioception are owned by the Stem physical-state store, not by Loom heuristics

## D. Cortex Organ And Goal-Forest Surface

Collect:

- `cortex.primary`
- `cortex.sense-helper`
- `cortex.goal-forest-helper`
- `cortex.acts-helper`
- `cortex.goal-forest`

Why:

- helper-scoped execution intervals carry the current Cortex request and AI-correlation story
- the current runtime owns goal-forest snapshot plus patch/persist semantics, not a fixed biological verb taxonomy
- the older `cortex.organ` and `cortex.tick` draft names are not the current canonical family names in code

## E. Spine Topology And Dispatch

Collect:

- `spine.adapter`
- `spine.endpoint`
- `spine.sense`
- `spine.act`

Why:

- Spine should answer which adapters and endpoints were active and how acts were actually routed

## What Lachesis Should Not Collect As First-Class Scope

- full metrics ingestion
- full trace ingestion outside the tick-anchored log contract
- precomputed cross-wake analytics
- curated health scores that hide the underlying evidence

Instead:

- Loom surfaces metrics exporter status
- Loom surfaces traces exporter status
- Loom provides handoff links or destinations for external metrics/traces inspection

## Recommended Storage Shape

### Raw Table

- append-only OTLP log event rows
- full structured attributes
- raw body payload

### Derived Tables And Indexes

- baseline:
  - `runs`
  - `ticks`
- next justified additions:
  - tick-lane index for Gantt rendering
  - thread index for conversation browsing
  - goal-forest snapshots only if raw-first retrieval proves awkward
  - focused signal or topology projections only if the dedicated views need them

If a derived table or index does not clearly pay for itself in UI clarity or query speed, keep the information raw-first.

## Working Defaults For This Task

- store full raw OTLP log events in DuckDB for the whole task
- materialize `runs` first
- materialize `ticks` first
- add only the extra tick-lane or thread indexes needed to make browsing humane
- treat raw payload preservation as default
- keep goal-forest comparison derived from selected ticks
- do not make retention or compaction a blocker for this task; manual reset is acceptable during early development

## Visualization Model

## 1. Wake Overview

- current wake state
- ingest state
- recent active ticks
- shortcuts into the latest inspectable tick and active conversation thread

## 2. Tick Workspace

- left: tick list with summary columns
- center: per-tick Gantt-like chronology grouped by span or stable resource lane
- right: selected event, thread, or subsystem detail

Selected tick browsing must not depend on raw JSON as the first thing a human sees.

## 3. Conversation Browser

- authoritative thread snapshots
- committed turn list and turn details
- linked AI-gateway request lifecycle for retries or failures
- provider, model, tool usage, token consumption, thinking payload, and full request/response payloads
- correlation back to the originating `organ_id`

## 4. Cortex / Stem / Spine Detail

- Cortex:
  - organ boundaries
  - goal-forest snapshots and patch history
- Stem:
  - tick, proprioception, descriptor catalog, signal transitions, dispatch transitions, and afferent rules
- Spine:
  - adapters
  - endpoints
  - dispatch binding and outcome

## 5. Raw Event Inspector

- raw OTLP event JSON
- full body and attributes
- only after the operator already has a humane browsing path

## Lachesis Sub-Stages Inside This Task

### Stage 1

- raw OTLP ingest
- `runs` projection
- `ticks` projection
- Loom wake list
- Loom tick list
- selected tick workspace
- raw-event inspector

### Stage 2

- realign the Core event contract around owner-centric logical families:
  - `ai-gateway.request`
  - `ai-gateway.chat.turn`
  - `ai-gateway.chat.thread`
  - `cortex.primary`
  - `cortex.sense-helper`
  - `cortex.goal-forest-helper`
  - `cortex.acts-helper`
  - `cortex.goal-forest`
  - `stem.tick`
  - `stem.afferent`
  - `stem.efferent`
  - `stem.proprioception`
  - `stem.ns-catalog`
  - `stem.afferent.rule`
  - `spine.adapter`
  - `spine.endpoint`
  - `spine.sense`
  - `spine.act`
- preserve full payloads in raw events
- stop relying on fuzzy keyword grouping for Stem and Spine
- keep richer semantics inside `kind`, `status`, and `transition_kind` fields instead of exploding family count

### Stage 3

- tick-lane and thread indexes only where the humane browsing model needs them
- per-tick Gantt-like chronology
- conversation browser backed by authoritative thread snapshots and committed turns

### Stage 4

- dedicated Goal-forest compare
- dedicated Stem pathway view if the tick workspace still feels too compressed
- dedicated Spine topology and dispatch view if the tick workspace still feels too compressed

### Stage 5

- storage and retention hardening
- migration policy
- query-performance tuning
