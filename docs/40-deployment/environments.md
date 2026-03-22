# Environments

## Primary Runtime Environment

- Core process runs as foreground binary with JSONC config.
- Body endpoints may be inline (same process) or external (Unix socket).

## Configuration Model

1. One unified config file input (`beluna.jsonc`).
2. Typed config structs define shape/defaults/validation.
3. Schema generation is code-driven and explicit.

Decision Source: ADR-002.
