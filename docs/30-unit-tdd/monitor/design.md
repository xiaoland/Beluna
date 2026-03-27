# Monitor Design

## Responsibility

Monitor provides a local web UX for reading, filtering, searching, and inspecting core runtime logs.

## Non-Responsibility

1. Monitor does not own runtime authority or dispatch semantics.
2. Monitor does not modify core logs or redefine cross-unit observability ownership.
3. Monitor does not act as remote observability control plane in this MVP.

## Local Design Invariants

1. Log consumption is read-only.
2. Parsing failures are bounded to row-level errors and never crash rendering.
3. Refresh behavior is explicit (`observer` or `polling`) and user-visible.
4. Filter semantics are deterministic for a given in-memory dataset.
