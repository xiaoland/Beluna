# CLI Interfaces

## Command Interface

- `cargo run -- --socket-path <path>`
- optional: `--endpoint-id <id>`

## Protocol Surface

1. Auth/register handshake over Unix socket NDJSON.
2. Emits user text senses.
3. Consumes present-text act payloads.

## Contract Notes

1. Capability registration must match descriptor identity constraints expected by Core.
2. Endpoint connectivity and protocol behavior must remain deterministic.
