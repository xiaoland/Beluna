# Spine PRD

## Purpose

Spine is the sole channel between Mind (Cortex + Non-Cortex) and Body (Endpoints).
It is a transport-ignorant execution kernel that only performs mechanical routing and dispatch of admitted actions.

## Core Invariants

1. Total transport ignorance in Spine core.
2. Universal endpoint abstraction: local/native and remote endpoints are invoked the same way.
3. Mechanical routing by (`affordance_key`, `capability_handle`) table lookup.

## Requirements

1. Spine accepts `AdmittedActionBatch` only.
2. Spine executes asynchronously and returns ordered `SpineExecutionReport`.
3. Missing route must map to deterministic per-action rejection.
4. Endpoint invoke failure must map to deterministic per-action rejection.
5. Settlement events must include reservation and attribution linkage fields.
6. Spine owns capability catalog snapshot used by Cortex.

## MVP Scope

1. Spine core routing kernel and in-memory registry.
2. UnixSocket adapter shell for ingress.
3. No WebSocket/HTTP adapter in this MVP iteration.
