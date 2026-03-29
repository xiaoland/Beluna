# Lachesis Collection And Visualization

All stages in this file are Lachesis sub-stages inside the same Moira task. They are not separate roadmap phases.

## Design Goal

Lachesis should make Beluna explainable during early development. The collection model should therefore optimize for:

- reconstructing what happened at a given tick
- following signal flow through Stem and Spine
- understanding which runtime topology was active
- preserving enough raw evidence that the UI does not become a lossy summary layer

## Collection Principles

### 1. Collect raw first, project second

- Persist every accepted OTLP log event in a raw table.
- Build derived tables only for views that are repeatedly expensive or awkward to reconstruct.
- Do not store precomputed interpretations such as “goal-forest diff” when they can be derived from two snapshots.

### 2. Collect Beluna-native structure, not just text

- The primary contract must be structured event fields.
- Free-form text can remain as debugging payload, but must not be the only way to recover a tick, signal, or dispatch story.

### 3. Prefer snapshots at semantic boundaries

- At tick boundaries, store enough data to reconstruct the tick in isolation.
- At descriptor/topology changes, store snapshot-like records so later queries do not require replaying the entire run from the beginning.

## What Lachesis Should Collect In V1

## A. Run And Process Lifecycle

- Moira-supervised run id
- Core version or source-build identity
- profile/config identity
- wake time, stop time, termination reason
- OTLP receiver health and ingest lag indicators

Why:
- every later view needs to scope itself to a run
- failures during wake/stop must be visible as first-class events

## B. Cortex Tick Records

For each tick:

- `tick`
- run id
- trigger summary
- pending sense count
- selected or consumed senses
- proprioception snapshot
- primary request metadata
- primary response metadata
- primary tool list
- emitted acts
- goal-forest snapshot reference

Why:
- this is the core unit of reasoning inspection the user asked for
- a single tick page should answer “what did Beluna know, decide, and emit here?”

## C. Goal-Forest Snapshots

Collect:

- full goal-forest snapshot at each tick for now
- snapshot handle keyed by run + tick
- summary fields for quick browsing

Do not collect:

- precomputed goal-forest diff rows as the canonical source of truth

Visualization:

- let Loom compare any two selected ticks side by side
- derive added, removed, and changed nodes in query/UI code

## D. Stem Signal Flow

Collect afferent and efferent transitions with:

- run id
- tick if known at the moment
- signal direction
- endpoint id
- neural-signal descriptor id
- sense or act instance id
- transition kind: enqueue, defer, release, drop, dispatch, result
- terminal outcome where relevant

Why:
- this powers the requested live neural-signal understanding
- it makes backpressure and queue behavior visible

## E. Neural-Signal Descriptor Catalog

Collect:

- full catalog snapshot on version change
- patch/drop events
- catalog version references from signal events when feasible

Visualization:

- current catalog view
- catalog history by run
- click from a signal to the descriptor active at that time

## F. Spine Topology And Dispatch

Collect:

- adapter lifecycle
- endpoint lifecycle
- route registration/drop
- dispatch binding and terminal outcome
- dispatch latency when available

Visualization:

- which adapters are enabled
- which body endpoints are currently connected
- which dispatches went where and how they ended

## What Lachesis Should Not Collect As First-Class V1 Scope

- full metrics ingestion
- full trace ingestion
- precomputed cross-run analytics
- heavily curated “health scores” that hide the underlying evidence

Instead:

- Loom surfaces metrics exporter status
- Loom surfaces traces exporter status
- Loom provides handoff links or destinations for external metrics/traces inspection

## Recommended Storage Shape

### Raw Table

- append-only OTLP log event rows
- full structured attributes
- raw body payload

### Derived Tables

- First landable slice:
  - `runs`
  - `ticks`
- Later only if query pressure proves the need:
  - `goal_forest_snapshots`
  - `signals`
  - `descriptor_catalog_snapshots`
  - `topology_events`
  - `dispatch_outcomes`

This is the current recommendation, not yet a durable contract. If a derived table does not clearly pay for itself in UI simplicity or query speed, keep the information raw-first.

## Working Defaults For This Task

- store full raw OTLP log events in DuckDB for the whole task
- materialize `runs` first
- materialize `ticks` first
- reconstruct selected tick detail from raw events before adding subsystem-specific tables
- snapshot proprioception per tick
- snapshot goal-forest per tick
- keep large bodies raw-first, with summary and preview columns in derived tables rather than duplicating full payloads widely
- treat any event family not explicitly promoted in `L2.md` as debug-only for now
- do not make retention or compaction a blocker for this task; manual reset is acceptable during early development

## Visualization Model

## 1. Overview

- current run state
- ingest state
- event-rate timeline
- recent warnings/errors
- shortcuts into active tick, active dispatch failures, and topology changes

The first landable slice may compress this into:

- run list
- active run state
- ingest state
- shortcuts into the latest inspectable tick

## 2. Cortex Tick Explorer

- left: tick list with summary columns
- center: selected tick narrative
- right: raw event inspector

Selected tick detail should show:

- senses
- proprioception
- primary messages
- primary tools
- acts
- linked goal-forest snapshot

This view is the minimum required Loom detail surface for the first landable slice, even before dedicated Stem and Spine pages exist.

## 3. Goal-Forest Compare

- choose tick A and tick B
- render trees side by side
- derive node changes in UI/query layer

This directly matches the clarified requirement and avoids baking one diff algorithm into storage too early.

## 4. Stem Signal Timeline

- lane by afferent/efferent
- filter by endpoint, descriptor, act id, sense id
- live mode during active runs
- click-through to raw event detail

## 5. Spine Topology View

- active adapters
- active endpoints
- recent connect/disconnect/register/drop events
- dispatch outcome stream

## Lachesis Sub-Stages Inside This Task

### Stage 1

- raw OTLP ingest
- `runs` projection
- `ticks` projection
- Loom run list
- Loom tick timeline
- Loom selected tick detail with Cortex / Stem / Spine tabs
- raw-event inspector

### Stage 2

- close the Core structured-log gaps exposed by Stage 1 surfaces
- goal-forest snapshot storage only if raw-first tick detail proves insufficient
- tighten the selected tick narrative

### Stage 3

- `signals`, `topology_events`, and `dispatch_outcomes` projections
- Stem and Spine dedicated views

### Stage 4

- storage/retention hardening
- migration policy
- query-performance tuning
