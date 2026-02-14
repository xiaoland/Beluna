# Cortex PRD

## Purpose

Cortex is Beluna's deliberative cognition boundary.

It consumes a drained `Sense[]` batch plus current physical+cognition state and emits:
1. non-binding `Act[]`
2. next `CognitionState`

## Requirements

- Cortex behaves as a pure boundary function (no direct side effects).
- Cortex does not durably persist cognition state itself.
- Primary LLM outputs prose IR; sub-LLM stages compile IR to structured drafts.
- Deterministic clamp is final authority before acts leave Cortex.
- Every non-noop act includes `based_on` sense ids.
- Cortex can intend broadly; execution is constrained downstream in Stem pipeline.
- Per-cycle hard bounds:
  - exactly 1 primary call,
  - at most N subcalls,
  - at most 1 repair call,
  - strict max attempts/payload/time/token limits,
  - noop fallback on irreparable generation.

## Out of Scope

- Direct execution access.
- Runtime queue ownership.
- Settlement or resource reservation decisions.
