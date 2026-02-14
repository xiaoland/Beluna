# Spine PRD

## Purpose

Spine is the sole execution channel between Stem and Body Endpoints.
It is transport-agnostic and performs mechanical route dispatch only.

## Core Invariants

1. Transport ignorance in Spine core.
2. Universal endpoint abstraction for local/native and remote endpoints.
3. Mechanical routing by route key (`endpoint_id`, `capability_id`).

## Requirements

1. Spine accepts `ActDispatchRequest` only.
2. Spine dispatches one act per call and returns one `SpineEvent`.
3. Missing route maps to deterministic rejection.
4. Endpoint invocation failure maps to deterministic rejection.
5. Every event includes settlement linkage fields (`reserve_entry_id`, `cost_attribution_id`).
6. Spine owns capability catalog snapshot used in physical state composition.

## MVP Scope

1. Spine core routing kernel and in-memory registry.
2. UnixSocket adapter shell for sense + capability ingress and body endpoint lifecycle.
3. No WebSocket/HTTP adapter in this MVP iteration.
