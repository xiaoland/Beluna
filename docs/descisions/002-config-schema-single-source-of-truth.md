# 002 - Config Schema Single Source of Truth

Date: 2026-03-22

## Status

Accepted

## Context

Core configuration had drift between multiple declarations and checks:

- Rust config structs and defaults in `core/src/config.rs`
- Hand-authored JSON Schema in `core/beluna.schema.json`
- Additional runtime fallback/repair logic (for example `.max(1)` guards)

This made configuration changes costly and error-prone. A field rename or invariant update required synchronized edits across schema, deserialization, runtime checks, and tests, with no guaranteed consistency.

## Decision

1. Rust config structs become the only source of truth for configuration shape and defaults.
2. Validation is performed through derive-based validation annotations on typed config structs.
3. `core/beluna.schema.json` is generated from Rust config types.
4. Core runtime no longer performs schema-file-based validation during startup.
5. Config ownership is modularized by subsystem while keeping one unified config file input (`beluna.jsonc`).
6. CLI adds `beluna config schema [--output <path>]` to generate schema from code.

## Consequences

- Configuration evolution now has one canonical contract, reducing drift risk.
- Unknown keys and invalid values fail fast at config boundary.
- Runtime paths rely on boundary invariants instead of local fallback repairs.
- Schema updates are explicit generation actions from the same typed source.
- Existing stale keys that are not present in typed config are rejected (no backward compatibility shims).
